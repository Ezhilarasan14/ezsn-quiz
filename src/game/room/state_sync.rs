use crate::game::room::{GameRoom, SHAPES};
use crate::game::types::*;
use crate::ws::messages::ServerMessage;
use std::time::Instant;

impl GameRoom {
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
}
