use crate::db::models::QuizDetail;
use crate::db::{self, DbPool, FinalPlayerResult};
use crate::game::scoring::ScoreCalculator;
use crate::game::types::*;
use crate::ws::messages::{LobbyPlayer, ServerMessage};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

const SHAPES: [&str; 4] = ["triangle", "diamond", "circle", "square"];

pub struct GameRoom {
    pub pin: String,
    pub game_id: String,
    pub host_token: String,
    pub quiz: QuizDetail,
    pub state: RoomState,
    pub host_tx: Option<mpsc::UnboundedSender<ServerMessage>>,
    pub players: HashMap<String, PlayerSession>,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub timer_token: u64,
}

impl GameRoom {
    pub fn new(pin: String, game_id: String, host_token: String, quiz: QuizDetail) -> Self {
        Self {
            pin,
            game_id,
            host_token,
            quiz,
            state: RoomState::Lobby,
            host_tx: None,
            players: HashMap::new(),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            timer_token: 0,
        }
    }

    pub fn is_abandoned(&self) -> bool {
        let inactive_duration = self.last_activity.elapsed();
        if self.host_tx.is_none() && inactive_duration > Duration::from_secs(900) {
            return true;
        }
        if inactive_duration > Duration::from_secs(7200) {
            return true;
        }
        false
    }

    pub fn register_host(&mut self, tx: mpsc::UnboundedSender<ServerMessage>) {
        self.last_activity = Instant::now();
        self.host_tx = Some(tx.clone());

        // Send initial confirmation
        let _ = tx.send(ServerMessage::Connected {
            role: "host".to_string(),
            pin: self.pin.clone(),
            player_id: None,
            nickname: None,
            is_reconnect: false,
        });

        // Send current state
        self.send_host_current_state();
    }

    pub fn register_player(
        &mut self,
        player_id_opt: Option<String>,
        nickname: String,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<String, String> {
        self.last_activity = Instant::now();

        // Check if this is a reconnect
        if let Some(pid) = player_id_opt {
            if let Some(player) = self.players.get_mut(&pid) {
                player.is_connected = true;
                player.tx = Some(tx.clone());
                player.last_seen = Instant::now();

                let nick = player.nickname.clone();
                let _ = tx.send(ServerMessage::Connected {
                    role: "player".to_string(),
                    pin: self.pin.clone(),
                    player_id: Some(pid.clone()),
                    nickname: Some(nick),
                    is_reconnect: true,
                });

                self.send_player_current_state(&pid);
                self.broadcast_lobby_update();
                return Ok(pid);
            }
        }

        // If not reconnect, check if game has already started
        match &self.state {
            RoomState::Lobby => {}
            _ => {
                return Err("Game has already started. New players cannot join right now.".to_string());
            }
        }

        let trimmed_name = nickname.trim();
        if trimmed_name.is_empty() {
            return Err("Nickname cannot be empty.".to_string());
        }
        if trimmed_name.len() > 20 {
            return Err("Nickname must be 20 characters or less.".to_string());
        }

        // Check duplicate nickname (case-insensitive)
        let name_lower = trimmed_name.to_lowercase();
        for p in self.players.values() {
            if p.nickname.to_lowercase() == name_lower && p.is_connected {
                return Err("Nickname is already taken. Please choose another name.".to_string());
            }
        }

        let new_id = Uuid::new_v4().to_string();
        let session = PlayerSession {
            id: new_id.clone(),
            nickname: trimmed_name.to_string(),
            score: 0,
            streak: 0,
            correct_count: 0,
            total_answered: 0,
            rank_history: Vec::new(),
            is_connected: true,
            last_seen: Instant::now(),
            tx: Some(tx.clone()),
        };

        self.players.insert(new_id.clone(), session);

        let _ = tx.send(ServerMessage::Connected {
            role: "player".to_string(),
            pin: self.pin.clone(),
            player_id: Some(new_id.clone()),
            nickname: Some(trimmed_name.to_string()),
            is_reconnect: false,
        });

        // Notify host and other players
        let player_count = self.active_player_count();
        if let Some(ref h_tx) = self.host_tx {
            let _ = h_tx.send(ServerMessage::PlayerJoined {
                player_id: new_id.clone(),
                nickname: trimmed_name.to_string(),
                player_count,
            });
        }

        self.broadcast_lobby_update();
        Ok(new_id)
    }

    pub fn disconnect_host(&mut self) {
        self.host_tx = None;
        self.last_activity = Instant::now();
    }

    pub fn disconnect_player(&mut self, player_id: &str) {
        let nickname_opt = if let Some(player) = self.players.get_mut(player_id) {
            player.is_connected = false;
            player.tx = None;
            player.last_seen = Instant::now();
            Some(player.nickname.clone())
        } else {
            None
        };

        if let Some(nickname) = nickname_opt {
            let p_count = self.active_player_count();
            if let Some(ref h_tx) = self.host_tx {
                let _ = h_tx.send(ServerMessage::PlayerLeft {
                    player_id: player_id.to_string(),
                    nickname,
                    player_count: p_count,
                });
            }

            self.broadcast_lobby_update();
        }
    }

    pub fn kick_player(&mut self, player_id: &str) -> bool {
        if let Some(player) = self.players.remove(player_id) {
            if let Some(tx) = player.tx {
                let _ = tx.send(ServerMessage::PlayerKicked {
                    reason: "You were removed from the room by the host.".to_string(),
                });
            }
            self.broadcast_lobby_update();
            true
        } else {
            false
        }
    }

    pub fn active_player_count(&self) -> usize {
        self.players.values().filter(|p| p.is_connected).count()
    }

    pub fn total_registered_players(&self) -> usize {
        self.players.len()
    }

    pub fn get_lobby_players(&self) -> Vec<LobbyPlayer> {
        self.players
            .values()
            .map(|p| LobbyPlayer {
                player_id: p.id.clone(),
                nickname: p.nickname.clone(),
                is_connected: p.is_connected,
            })
            .collect()
    }

    pub fn broadcast_lobby_update(&self) {
        if !matches!(self.state, RoomState::Lobby) {
            return;
        }

        let players = self.get_lobby_players();
        let count = players.iter().filter(|p| p.is_connected).count();

        let msg = ServerMessage::LobbyState {
            pin: self.pin.clone(),
            quiz_title: self.quiz.title.clone(),
            player_count: count,
            players,
        };

        self.broadcast(&msg);
    }

    pub fn start_game(room_arc: Arc<RwLock<GameRoom>>) {
        tokio::spawn(async move {
            let mut room = room_arc.write().await;
            if !matches!(room.state, RoomState::Lobby) {
                return;
            }
            if room.quiz.questions.is_empty() {
                if let Some(ref h_tx) = room.host_tx {
                    let _ = h_tx.send(ServerMessage::Error {
                        message: "Cannot start game: quiz has no questions.".to_string(),
                    });
                }
                return;
            }

            room.last_activity = Instant::now();
            room.timer_token += 1;
            let current_token = room.timer_token;

            // Broadcast game starting (3s countdown)
            room.broadcast(&ServerMessage::GameStarting { countdown_secs: 3 });
            drop(room);

            tokio::time::sleep(Duration::from_secs(3)).await;

            // Advance to Question 0 Intro
            GameRoom::start_question_intro(room_arc.clone(), 0, current_token);
        });
    }

    pub fn start_question_intro(room_arc: Arc<RwLock<GameRoom>>, q_idx: usize, token: u64) {
        tokio::spawn(async move {
            let mut room = room_arc.write().await;
            if room.timer_token != token {
                return;
            }
            if q_idx >= room.quiz.questions.len() {
                return;
            }

            room.timer_token += 1;
            let next_token = room.timer_token;

            let question_text = room.quiz.questions[q_idx].question_text.clone();
            let total_questions = room.quiz.questions.len();
            let intro_secs = 4;
            room.state = RoomState::QuestionIntro {
                question_index: q_idx,
                ends_at: Instant::now() + Duration::from_secs(intro_secs),
                total_seconds: intro_secs,
            };

            let intro_msg = ServerMessage::QuestionIntro {
                question_index: q_idx + 1,
                total_questions,
                question_text,
                countdown_secs: intro_secs,
            };
            room.broadcast(&intro_msg);
            drop(room);

            tokio::time::sleep(Duration::from_secs(intro_secs)).await;

            GameRoom::start_question_active(room_arc.clone(), q_idx, next_token);
        });
    }

    pub fn start_question_active(room_arc: Arc<RwLock<GameRoom>>, q_idx: usize, token: u64) {
        tokio::spawn(async move {
            let mut room = room_arc.write().await;
            if room.timer_token != token {
                return;
            }
            if q_idx >= room.quiz.questions.len() {
                return;
            }

            room.timer_token += 1;
            let current_token = room.timer_token;

            let (duration_secs, question_text, total_questions, options_view) = {
                let q = &room.quiz.questions[q_idx];
                let dur = (q.time_limit_sec as u64).max(5);
                let text = q.question_text.clone();
                let tot = room.quiz.questions.len();
                let opts: Vec<OptionView> = q
                    .options
                    .iter()
                    .enumerate()
                    .map(|(idx, opt)| OptionView {
                        id: opt.id.clone(),
                        text: opt.option_text.clone(),
                        color_idx: idx % 4,
                        shape: SHAPES[idx % 4].to_string(),
                    })
                    .collect();
                (dur, text, tot, opts)
            };

            room.state = RoomState::QuestionActive {
                question_index: q_idx,
                started_at: Instant::now(),
                duration_secs,
                player_answers: HashMap::new(),
            };

            let active_msg = ServerMessage::QuestionActive {
                question_index: q_idx + 1,
                total_questions,
                question_text,
                time_limit_sec: duration_secs,
                options: options_view,
                already_answered: false,
            };

            room.broadcast(&active_msg);

            // Send initial 0/N answer count to host
            let active_players = room.active_player_count();
            if let Some(ref h_tx) = room.host_tx {
                let _ = h_tx.send(ServerMessage::HostAnswerCountUpdate {
                    answered_count: 0,
                    total_players: active_players,
                });
            }

            drop(room);

            // Countdown timer task
            tokio::time::sleep(Duration::from_secs(duration_secs)).await;

            // Timer expired: trigger end of question
            GameRoom::end_question_if_active(room_arc.clone(), q_idx, current_token);
        });
    }

    pub async fn submit_answer(
        room_arc: Arc<RwLock<GameRoom>>,
        player_id: &str,
        question_id: &str,
        option_id: &str,
    ) -> Result<(), String> {
        let mut room = room_arc.write().await;
        let (q_idx, started_at, duration_secs) = match &room.state {
            RoomState::QuestionActive {
                question_index,
                started_at,
                duration_secs,
                player_answers,
            } => {
                if player_answers.contains_key(player_id) {
                    return Err("You have already submitted an answer for this question.".to_string());
                }
                (*question_index, *started_at, *duration_secs)
            }
            _ => {
                return Err("No active question currently accepting answers.".to_string());
            }
        };

        if q_idx >= room.quiz.questions.len() {
            return Err("Invalid question index.".to_string());
        }

        let question = &room.quiz.questions[q_idx];
        if !question_id.is_empty() && question.id != question_id {
            return Err("Answer submitted for incorrect question.".to_string());
        }

        let elapsed_ms = started_at.elapsed().as_millis() as u64;
        let total_duration_ms = duration_secs * 1000;
        if elapsed_ms > total_duration_ms + 1000 {
            return Err("Time is up! Your answer was submitted too late.".to_string());
        }

        // Validate option & check correctness
        let selected_option = question.options.iter().find(|o| o.id == option_id);
        let is_correct = selected_option.map(|o| o.is_correct).unwrap_or(false);

        let current_streak = room
            .players
            .get(player_id)
            .map(|p| p.streak)
            .unwrap_or(0);

        let (points_awarded, new_streak) = ScoreCalculator::calculate(
            is_correct,
            question.points,
            elapsed_ms,
            total_duration_ms,
            current_streak,
        );

        // Update player score & stats
        if let Some(player) = room.players.get_mut(player_id) {
            player.score += points_awarded;
            player.streak = new_streak;
            player.total_answered += 1;
            if is_correct {
                player.correct_count += 1;
            }

            if let Some(ref tx) = player.tx {
                let _ = tx.send(ServerMessage::AnswerAccepted {
                    question_index: q_idx + 1,
                });
            }
        }

        // Store answer in room state
        let player_answer = PlayerAnswer {
            option_id: option_id.to_string(),
            response_time_ms: elapsed_ms,
            is_correct,
            points_awarded,
            submitted_at: Instant::now(),
        };

        let mut all_answered = false;
        let mut answered_count = 0;
        let active_players = room.active_player_count();

        if let RoomState::QuestionActive {
            ref mut player_answers,
            ..
        } = room.state
        {
            player_answers.insert(player_id.to_string(), player_answer);
            answered_count = player_answers.len();
            if active_players > 0 && answered_count >= active_players {
                all_answered = true;
            }
        }

        // Send update to host
        if let Some(ref h_tx) = room.host_tx {
            let _ = h_tx.send(ServerMessage::HostAnswerCountUpdate {
                answered_count,
                total_players: active_players,
            });
        }

        let current_token = room.timer_token;
        drop(room);

        // If everyone answered, finish question early without waiting for the full timer!
        if all_answered {
            GameRoom::end_question_if_active(room_arc, q_idx, current_token);
        }

        Ok(())
    }

    pub fn end_question_if_active(room_arc: Arc<RwLock<GameRoom>>, q_idx: usize, token: u64) {
        tokio::spawn(async move {
            let mut room = room_arc.write().await;
            if room.timer_token != token {
                return;
            }

            let (answers, question_index) = match &room.state {
                RoomState::QuestionActive {
                    question_index,
                    player_answers,
                    ..
                } if *question_index == q_idx => (player_answers.clone(), *question_index),
                _ => return,
            };

            room.timer_token += 1;

            if question_index >= room.quiz.questions.len() {
                return;
            }

            let question = &room.quiz.questions[question_index];
            let correct_option_ids: Vec<String> = question
                .options
                .iter()
                .filter(|o| o.is_correct)
                .map(|o| o.id.clone())
                .collect();

            // Count options chosen
            let mut option_counts: HashMap<String, u32> = HashMap::new();
            for opt in &question.options {
                option_counts.insert(opt.id.clone(), 0);
            }
            for ans in answers.values() {
                *option_counts.entry(ans.option_id.clone()).or_insert(0) += 1;
            }

            room.state = RoomState::QuestionResult {
                question_index,
                correct_option_ids: correct_option_ids.clone(),
                option_counts: option_counts.clone(),
                player_answers: answers.clone(),
            };

            // Send host results
            if let Some(ref h_tx) = room.host_tx {
                let _ = h_tx.send(ServerMessage::QuestionResult {
                    question_index: question_index + 1,
                    correct_option_ids: correct_option_ids.clone(),
                    option_counts: option_counts.clone(),
                    player_correct: None,
                    points_awarded: None,
                    total_score: None,
                    streak: None,
                });
            }

            // Send each player their individual result
            for (pid, player) in &room.players {
                if let Some(ref tx) = player.tx {
                    let ans = answers.get(pid);
                    let (correct, points) = match ans {
                        Some(a) => (Some(a.is_correct), Some(a.points_awarded)),
                        None => (Some(false), Some(0)),
                    };

                    let _ = tx.send(ServerMessage::QuestionResult {
                        question_index: question_index + 1,
                        correct_option_ids: correct_option_ids.clone(),
                        option_counts: option_counts.clone(),
                        player_correct: correct,
                        points_awarded: points,
                        total_score: Some(player.score),
                        streak: Some(player.streak),
                    });
                }
            }
        });
    }

    pub fn host_advance_step(room_arc: Arc<RwLock<GameRoom>>, db_pool: DbPool) {
        tokio::spawn(async move {
            let mut room = room_arc.write().await;
            room.last_activity = Instant::now();

            match &room.state {
                RoomState::Lobby => {
                    drop(room);
                    GameRoom::start_game(room_arc);
                }
                RoomState::QuestionActive { question_index, .. } => {
                    let q_idx = *question_index;
                    let token = room.timer_token;
                    drop(room);
                    GameRoom::end_question_if_active(room_arc, q_idx, token);
                }
                RoomState::QuestionResult { question_index, .. } => {
                    let q_idx = *question_index;
                    room.timer_token += 1;

                    let mut player_list: Vec<_> = room.players.values_mut().collect();
                    player_list.sort_by(|a, b| b.score.cmp(&a.score));

                    let mut rankings = Vec::new();
                    for (rank_idx, player) in player_list.into_iter().enumerate() {
                        let rank = rank_idx + 1;
                        let prev_rank = player.rank_history.last().copied().unwrap_or(rank);
                        let rank_change = (prev_rank as i32) - (rank as i32);
                        player.rank_history.push(rank);

                        rankings.push(LeaderboardEntry {
                            player_id: player.id.clone(),
                            nickname: player.nickname.clone(),
                            score: player.score,
                            rank,
                            rank_change,
                            streak: player.streak,
                            is_connected: player.is_connected,
                        });
                    }

                    room.state = RoomState::Leaderboard {
                        question_index: q_idx,
                        rankings: rankings.clone(),
                    };

                    if let Some(ref h_tx) = room.host_tx {
                        let _ = h_tx.send(ServerMessage::LeaderboardUpdate {
                            rankings: rankings.clone(),
                            my_rank: None,
                            my_score: None,
                        });
                    }

                    for (pid, player) in &room.players {
                        if let Some(ref tx) = player.tx {
                            let my_entry = rankings.iter().find(|r| r.player_id == *pid);
                            let _ = tx.send(ServerMessage::LeaderboardUpdate {
                                rankings: rankings.clone(),
                                my_rank: my_entry.map(|e| e.rank),
                                my_score: Some(player.score),
                            });
                        }
                    }
                }
                RoomState::Leaderboard { question_index, .. } => {
                    let next_q = *question_index + 1;
                    if next_q < room.quiz.questions.len() {
                        let token = room.timer_token;
                        drop(room);
                        GameRoom::start_question_intro(room_arc, next_q, token);
                    } else {
                        room.timer_token += 1;
                        let mut sorted_players: Vec<_> = room.players.values().collect();
                        sorted_players.sort_by(|a, b| b.score.cmp(&a.score));

                        let podium: Vec<PodiumEntry> = sorted_players
                            .iter()
                            .take(3)
                            .enumerate()
                            .map(|(idx, p)| PodiumEntry {
                                player_id: p.id.clone(),
                                nickname: p.nickname.clone(),
                                score: p.score,
                                rank: idx + 1,
                                correct_count: p.correct_count,
                                total_answered: p.total_answered,
                            })
                            .collect();

                        let full_results: Vec<LeaderboardEntry> = sorted_players
                            .iter()
                            .enumerate()
                            .map(|(idx, p)| LeaderboardEntry {
                                player_id: p.id.clone(),
                                nickname: p.nickname.clone(),
                                score: p.score,
                                rank: idx + 1,
                                rank_change: 0,
                                streak: p.streak,
                                is_connected: p.is_connected,
                            })
                            .collect();

                        room.state = RoomState::Podium {
                            podium: podium.clone(),
                            full_results: full_results.clone(),
                        };

                        if let Some(ref h_tx) = room.host_tx {
                            let _ = h_tx.send(ServerMessage::Podium {
                                podium: podium.clone(),
                                full_results: full_results.clone(),
                                my_rank: None,
                            });
                        }

                        for (pid, player) in &room.players {
                            if let Some(ref tx) = player.tx {
                                let my_rank = full_results.iter().position(|r| r.player_id == *pid).map(|i| i + 1);
                                let _ = tx.send(ServerMessage::Podium {
                                    podium: podium.clone(),
                                    full_results: full_results.clone(),
                                    my_rank,
                                });
                            }
                        }

                        let game_id = room.game_id.clone();
                        let total_players = room.players.len() as i64;
                        let db_results: Vec<FinalPlayerResult> = full_results
                            .iter()
                            .map(|r| {
                                let p = room.players.get(&r.player_id);
                                FinalPlayerResult {
                                    player_id: r.player_id.clone(),
                                    nickname: r.nickname.clone(),
                                    total_score: r.score,
                                    final_rank: r.rank as i64,
                                    correct_count: p.map(|pl| pl.correct_count as i64).unwrap_or(0),
                                    total_answered: p.map(|pl| pl.total_answered as i64).unwrap_or(0),
                                }
                            })
                            .collect();

                        tokio::spawn(async move {
                            let _ = db::update_game_status(&db_pool, &game_id, "completed", total_players).await;
                            let _ = db::save_game_results(&db_pool, &game_id, &db_results).await;
                            log::info!("Game {} results saved to SQLite successfully.", game_id);
                        });
                    }
                }
                _ => {}
            }
        });
    }

    pub fn send_host_current_state(&self) {
        let Some(ref tx) = self.host_tx else { return };

        match &self.state {
            RoomState::Lobby => {
                let players = self.get_lobby_players();
                let count = players.iter().filter(|p| p.is_connected).count();
                let _ = tx.send(ServerMessage::LobbyState {
                    pin: self.pin.clone(),
                    quiz_title: self.quiz.title.clone(),
                    player_count: count,
                    players,
                });
            }
            RoomState::QuestionIntro {
                question_index,
                ends_at,
                ..
            } => {
                let remaining = ends_at.saturating_duration_since(Instant::now()).as_secs();
                let q = &self.quiz.questions[*question_index];
                let _ = tx.send(ServerMessage::QuestionIntro {
                    question_index: *question_index + 1,
                    total_questions: self.quiz.questions.len(),
                    question_text: q.question_text.clone(),
                    countdown_secs: remaining,
                });
            }
            RoomState::QuestionActive {
                question_index,
                duration_secs,
                started_at,
                player_answers,
            } => {
                let elapsed = started_at.elapsed().as_secs();
                let remaining = duration_secs.saturating_sub(elapsed);
                let q = &self.quiz.questions[*question_index];
                let options_view: Vec<OptionView> = q
                    .options
                    .iter()
                    .enumerate()
                    .map(|(idx, opt)| OptionView {
                        id: opt.id.clone(),
                        text: opt.option_text.clone(),
                        color_idx: idx % 4,
                        shape: SHAPES[idx % 4].to_string(),
                    })
                    .collect();

                let _ = tx.send(ServerMessage::QuestionActive {
                    question_index: *question_index + 1,
                    total_questions: self.quiz.questions.len(),
                    question_text: q.question_text.clone(),
                    time_limit_sec: remaining,
                    options: options_view,
                    already_answered: false,
                });

                let _ = tx.send(ServerMessage::HostAnswerCountUpdate {
                    answered_count: player_answers.len(),
                    total_players: self.active_player_count(),
                });
            }
            RoomState::QuestionResult {
                question_index,
                correct_option_ids,
                option_counts,
                ..
            } => {
                let _ = tx.send(ServerMessage::QuestionResult {
                    question_index: *question_index + 1,
                    correct_option_ids: correct_option_ids.clone(),
                    option_counts: option_counts.clone(),
                    player_correct: None,
                    points_awarded: None,
                    total_score: None,
                    streak: None,
                });
            }
            RoomState::Leaderboard { rankings, .. } => {
                let _ = tx.send(ServerMessage::LeaderboardUpdate {
                    rankings: rankings.clone(),
                    my_rank: None,
                    my_score: None,
                });
            }
            RoomState::Podium {
                podium,
                full_results,
            } => {
                let _ = tx.send(ServerMessage::Podium {
                    podium: podium.clone(),
                    full_results: full_results.clone(),
                    my_rank: None,
                });
            }
        }
    }

    pub fn send_player_current_state(&self, player_id: &str) {
        let Some(player) = self.players.get(player_id) else { return };
        let Some(ref tx) = player.tx else { return };

        match &self.state {
            RoomState::Lobby => {
                let players = self.get_lobby_players();
                let count = players.iter().filter(|p| p.is_connected).count();
                let _ = tx.send(ServerMessage::LobbyState {
                    pin: self.pin.clone(),
                    quiz_title: self.quiz.title.clone(),
                    player_count: count,
                    players,
                });
            }
            RoomState::QuestionIntro {
                question_index,
                ends_at,
                ..
            } => {
                let remaining = ends_at.saturating_duration_since(Instant::now()).as_secs();
                let q = &self.quiz.questions[*question_index];
                let _ = tx.send(ServerMessage::QuestionIntro {
                    question_index: *question_index + 1,
                    total_questions: self.quiz.questions.len(),
                    question_text: q.question_text.clone(),
                    countdown_secs: remaining,
                });
            }
            RoomState::QuestionActive {
                question_index,
                duration_secs,
                started_at,
                player_answers,
            } => {
                let elapsed = started_at.elapsed().as_secs();
                let remaining = duration_secs.saturating_sub(elapsed);
                let q = &self.quiz.questions[*question_index];
                let already_answered = player_answers.contains_key(player_id);

                let options_view: Vec<OptionView> = q
                    .options
                    .iter()
                    .enumerate()
                    .map(|(idx, opt)| OptionView {
                        id: opt.id.clone(),
                        text: opt.option_text.clone(),
                        color_idx: idx % 4,
                        shape: SHAPES[idx % 4].to_string(),
                    })
                    .collect();

                let _ = tx.send(ServerMessage::QuestionActive {
                    question_index: *question_index + 1,
                    total_questions: self.quiz.questions.len(),
                    question_text: q.question_text.clone(),
                    time_limit_sec: remaining,
                    options: options_view,
                    already_answered,
                });
            }
            RoomState::QuestionResult {
                question_index,
                correct_option_ids,
                option_counts,
                player_answers,
            } => {
                let ans = player_answers.get(player_id);
                let (correct, points) = match ans {
                    Some(a) => (Some(a.is_correct), Some(a.points_awarded)),
                    None => (Some(false), Some(0)),
                };

                let _ = tx.send(ServerMessage::QuestionResult {
                    question_index: *question_index + 1,
                    correct_option_ids: correct_option_ids.clone(),
                    option_counts: option_counts.clone(),
                    player_correct: correct,
                    points_awarded: points,
                    total_score: Some(player.score),
                    streak: Some(player.streak),
                });
            }
            RoomState::Leaderboard { rankings, .. } => {
                let my_entry = rankings.iter().find(|r| r.player_id == player_id);
                let _ = tx.send(ServerMessage::LeaderboardUpdate {
                    rankings: rankings.clone(),
                    my_rank: my_entry.map(|e| e.rank),
                    my_score: Some(player.score),
                });
            }
            RoomState::Podium {
                podium,
                full_results,
            } => {
                let my_rank = full_results.iter().position(|r| r.player_id == player_id).map(|i| i + 1);
                let _ = tx.send(ServerMessage::Podium {
                    podium: podium.clone(),
                    full_results: full_results.clone(),
                    my_rank,
                });
            }
        }
    }

    pub fn broadcast(&self, message: &ServerMessage) {
        if let Some(ref h_tx) = self.host_tx {
            let _ = h_tx.send(message.clone());
        }

        for player in self.players.values() {
            if let Some(ref tx) = player.tx {
                let _ = tx.send(message.clone());
            }
        }
    }
}
