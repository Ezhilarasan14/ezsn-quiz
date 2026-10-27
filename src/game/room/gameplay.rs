use crate::db::{self, DbPool, FinalPlayerResult};
use crate::game::room::{GameRoom, SHAPES};
use crate::game::types::*;
use crate::ws::messages::ServerMessage;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

impl GameRoom {
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

            room.broadcast(&ServerMessage::GameStarting { countdown_secs: 3 });
            drop(room);

            tokio::time::sleep(Duration::from_secs(3)).await;
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
                let dur = if q.time_limit_sec > 0 {
                    q.time_limit_sec as u64
                } else if room.question_time_secs > 0 {
                    room.question_time_secs
                } else {
                    20
                };
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

            let q_id = room.quiz.questions[q_idx].id.clone();
            let active_msg = ServerMessage::QuestionActive {
                question_index: q_idx + 1,
                total_questions,
                question_text,
                time_limit_sec: duration_secs,
                options: options_view,
                already_answered: false,
                question_id: q_id,
            };

            room.broadcast(&active_msg);

            let active_players = room.active_player_count();
            if let Some(ref h_tx) = room.host_tx {
                let _ = h_tx.send(ServerMessage::HostAnswerCountUpdate {
                    answered_count: 0,
                    total_players: active_players,
                });
            }

            drop(room);

            tokio::time::sleep(Duration::from_secs(duration_secs)).await;
            GameRoom::end_question_if_active(room_arc.clone(), q_idx, current_token);
        });
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

            // Record answers in all_answers accumulator
            room.all_answers.entry(question_index).or_default().extend(answers.clone());

            // Update player rank histories
            let mut player_list: Vec<_> = room.players.values_mut().collect();
            player_list.sort_by(|a, b| b.score.cmp(&a.score));
            for (rank_idx, player) in player_list.into_iter().enumerate() {
                let rank = rank_idx + 1;
                player.rank_history.push(rank);
            }

            let question = &room.quiz.questions[question_index];
            let correct_option_ids: Vec<String> = question
                .options
                .iter()
                .filter(|o| o.is_correct)
                .map(|o| o.id.clone())
                .collect();

            let mut option_counts: HashMap<String, u32> = HashMap::new();
            for opt in &question.options {
                option_counts.insert(opt.id.clone(), 0);
            }
            for ans in answers.values() {
                *option_counts.entry(ans.option_id.clone()).or_insert(0) += 1;
            }

            // Broadcast QuestionResult to Host Screen
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

            // Broadcast QuestionResult to each player with personalized score/correctness
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

            // Set room state to QuestionResult and WAIT for host to click Next Question
            room.state = RoomState::QuestionResult {
                question_index,
                correct_option_ids,
                option_counts,
                player_answers: answers,
            };
        });
    }

    pub fn build_player_summary(&self, player_id: &str, final_rank: usize) -> Option<PlayerQuizSummary> {
        let player = self.players.get(player_id)?;
        let total_questions = self.quiz.questions.len();
        let total_players = self.players.len();

        let mut reviews = Vec::new();
        for (q_idx, question) in self.quiz.questions.iter().enumerate() {
            let correct_opt = question.options.iter().find(|o| o.is_correct);
            let correct_opt_id = correct_opt.map(|o| o.id.clone()).unwrap_or_default();
            let correct_opt_text = correct_opt.map(|o| o.option_text.clone()).unwrap_or_default();

            let player_ans = self.all_answers.get(&q_idx).and_then(|ans_map| ans_map.get(player_id));

            let q_time_limit = if question.time_limit_sec > 0 {
                question.time_limit_sec as u64
            } else if self.question_time_secs > 0 {
                self.question_time_secs
            } else {
                20
            };

            let (selected_opt_id, selected_opt_text, is_correct, points_awarded, response_time_ms) = match player_ans {
                Some(ans) => {
                    let opt_text = question.options.iter().find(|o| o.id == ans.option_id).map(|o| o.option_text.clone());
                    (Some(ans.option_id.clone()), opt_text, ans.is_correct, ans.points_awarded, ans.response_time_ms)
                }
                None => {
                    (None, None, false, 0, q_time_limit * 1000)
                }
            };

            reviews.push(QuestionReviewItem {
                question_id: question.id.clone(),
                question_index: q_idx + 1,
                question_text: question.question_text.clone(),
                selected_option_id: selected_opt_id,
                selected_option_text: selected_opt_text,
                correct_option_id: correct_opt_id,
                correct_option_text: correct_opt_text,
                is_correct,
                points_awarded,
                response_time_ms,
                time_limit_sec: q_time_limit,
            });
        }

        let correct_count = player.correct_count;
        let incorrect_count = total_questions.saturating_sub(correct_count);
        let accuracy_percentage = if total_questions > 0 {
            ((correct_count as f64 / total_questions as f64) * 1000.0).round() / 10.0
        } else {
            0.0
        };

        let total_time_ms: u64 = reviews.iter().map(|r| r.response_time_ms).sum();

        Some(PlayerQuizSummary {
            total_score: player.score,
            correct_count,
            incorrect_count,
            total_questions,
            accuracy_percentage,
            final_rank,
            total_players,
            total_time_ms,
            reviews,
        })
    }

    pub async fn finalize_game(room_arc: Arc<RwLock<GameRoom>>, _token: u64) {
        let mut room = room_arc.write().await;
        let mut sorted_players: Vec<_> = room.players.values().collect();
        sorted_players.sort_by(|a, b| b.score.cmp(&a.score));

        let total_players = room.players.len();

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
                total_time_ms: p.total_time_ms,
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
                total_time_ms: p.total_time_ms,
            })
            .collect();

        room.state = RoomState::Podium {
            podium: podium.clone(),
            full_results: full_results.clone(),
        };

        // Broadcast to Host Screen
        if let Some(ref h_tx) = room.host_tx {
            let _ = h_tx.send(ServerMessage::Podium {
                podium: podium.clone(),
                full_results: full_results.clone(),
                my_rank: None,
                player_summary: None,
            });
        }

        // Broadcast to each Player with their personal summary & answer review
        for (pid, player) in &room.players {
            if let Some(ref tx) = player.tx {
                let my_rank = full_results.iter().position(|r| r.player_id == *pid).map(|i| i + 1).unwrap_or(1);
                let summary = room.build_player_summary(pid, my_rank);

                let _ = tx.send(ServerMessage::Podium {
                    podium: podium.clone(),
                    full_results: full_results.clone(),
                    my_rank: Some(my_rank),
                    player_summary: summary,
                });
            }
        }

        // Database Persistence
        let game_id = room.game_id.clone();
        let total_players_i64 = total_players as i64;
        let db_results: Vec<FinalPlayerResult> = full_results
            .iter()
            .map(|r| {
                let p = room.players.get(&r.player_id);
                FinalPlayerResult {
                    player_id: r.player_id.clone(),
                    nickname: r.nickname.clone(),
                    email: p.and_then(|pl| pl.email.clone()),
                    total_score: r.score,
                    final_rank: r.rank as i64,
                    correct_count: p.map(|pl| pl.correct_count as i64).unwrap_or(0),
                    total_answered: p.map(|pl| pl.total_answered as i64).unwrap_or(0),
                }
            })
            .collect();

        let mut answer_inserts = Vec::new();
        for (q_idx, ans_map) in &room.all_answers {
            if let Some(question) = room.quiz.questions.get(*q_idx) {
                for (pid, ans) in ans_map {
                    answer_inserts.push(crate::db::GameAnswerInsert {
                        id: uuid::Uuid::new_v4().to_string(),
                        game_id: game_id.clone(),
                        question_id: question.id.clone(),
                        player_id: pid.clone(),
                        option_id: ans.option_id.clone(),
                        is_correct: ans.is_correct,
                        points_awarded: ans.points_awarded,
                        response_time_ms: ans.response_time_ms as i64,
                    });
                }
            }
        }

        let db_pool = room.db_pool.clone();
        tokio::spawn(async move {
            let _ = db::update_game_status(&db_pool, &game_id, "completed", total_players_i64).await;
            let _ = db::save_game_results(&db_pool, &game_id, &db_results).await;
            let _ = db::save_game_answers(&db_pool, &answer_inserts).await;
        });
    }

    pub fn host_advance_step(room_arc: Arc<RwLock<GameRoom>>, _db_pool: DbPool) {
        tokio::spawn(async move {
            let mut room = room_arc.write().await;
            room.last_activity = Instant::now();

            match &room.state {
                RoomState::Lobby => {
                    drop(room);
                    GameRoom::start_game(room_arc);
                }
                RoomState::QuestionIntro { question_index, .. } => {
                    let q_idx = *question_index;
                    room.timer_token += 1;
                    let token = room.timer_token;
                    drop(room);
                    GameRoom::start_question_active(room_arc, q_idx, token);
                }
                RoomState::QuestionActive { question_index, .. } => {
                    let q_idx = *question_index;
                    let token = room.timer_token;
                    drop(room);
                    GameRoom::end_question_if_active(room_arc, q_idx, token);
                }
                RoomState::QuestionResult { question_index, .. } => {
                    let q_idx = *question_index;
                    let next_q = q_idx + 1;
                    room.timer_token += 1;
                    let token = room.timer_token;
                    if next_q < room.quiz.questions.len() {
                        drop(room);
                        GameRoom::start_question_intro(room_arc, next_q, token);
                    } else {
                        drop(room);
                        GameRoom::finalize_game(room_arc, token).await;
                    }
                }
                RoomState::Leaderboard { question_index, .. } => {
                    let next_q = *question_index + 1;
                    room.timer_token += 1;
                    let token = room.timer_token;
                    if next_q < room.quiz.questions.len() {
                        drop(room);
                        GameRoom::start_question_intro(room_arc, next_q, token);
                    } else {
                        drop(room);
                        GameRoom::finalize_game(room_arc, token).await;
                    }
                }
                _ => {}
            }
        });
    }
}
