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

            room.state = RoomState::QuestionResult {
                question_index,
                correct_option_ids: correct_option_ids.clone(),
                option_counts: option_counts.clone(),
                player_answers: answers.clone(),
            };

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
}
