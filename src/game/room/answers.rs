use crate::game::room::GameRoom;
use crate::game::scoring::ScoreCalculator;
use crate::game::types::*;
use crate::ws::messages::ServerMessage;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

impl GameRoom {
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
            player.total_time_ms += elapsed_ms;
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

        // Also record in cumulative all_answers map
        room.all_answers
            .entry(q_idx)
            .or_default()
            .insert(player_id.to_string(), player_answer.clone());

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

        // If everyone answered, finish question early without waiting for the full timer
        if all_answered {
            GameRoom::end_question_if_active(room_arc, q_idx, current_token);
        }

        Ok(())
    }
}
