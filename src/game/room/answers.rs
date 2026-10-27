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

        // Flexible question_id matching: accept empty, UUID, or 1-based / 0-based question index representations
        let q_idx_1_based = format!("q_{}", q_idx + 1);
        let q_idx_0_based = format!("q_{}", q_idx);
        let num_1_based = (q_idx + 1).to_string();
        let num_0_based = q_idx.to_string();

        let is_valid_qid = question_id.is_empty()
            || question.id == question_id
            || question_id == q_idx_1_based
            || question_id == q_idx_0_based
            || question_id == num_1_based
            || question_id == num_0_based;

        if !is_valid_qid {
            log::warn!(
                "Answer rejected for player {}: question_id '{}' did not match active question id '{}' or index '{}'",
                player_id,
                question_id,
                question.id,
                q_idx + 1
            );
            return Err("Answer submitted for incorrect question.".to_string());
        }

        let elapsed_ms = started_at.elapsed().as_millis() as u64;
        let total_duration_ms = duration_secs * 1000;
        if elapsed_ms > total_duration_ms {
            return Err("Time is up! Answers are locked for this question.".to_string());
        }
        let clamped_elapsed_ms = elapsed_ms.min(total_duration_ms);

        // Validate option & check correctness
        // Match by exact option ID (UUID), option text, or 0/1-based index fallback
        let selected_option = question
            .options
            .iter()
            .find(|o| o.id == option_id || o.option_text.trim().eq_ignore_ascii_case(option_id.trim()))
            .or_else(|| {
                let idx_opt = option_id
                    .strip_prefix("opt_")
                    .or_else(|| option_id.strip_prefix("opt-"))
                    .unwrap_or(option_id)
                    .parse::<usize>()
                    .ok();
                idx_opt.and_then(|idx| {
                    if idx < question.options.len() {
                        Some(&question.options[idx])
                    } else if idx > 0 && (idx - 1) < question.options.len() {
                        Some(&question.options[idx - 1])
                    } else {
                        None
                    }
                })
            });

        let Some(option) = selected_option else {
            log::warn!(
                "Selected option '{}' not found in question '{}'. Options: {:?}",
                option_id,
                question.id,
                question.options.iter().map(|o| (&o.id, &o.option_text)).collect::<Vec<_>>()
            );
            return Err("Invalid option selected for this question.".to_string());
        };

        let canonical_option_id = option.id.clone();
        let is_correct = option.is_correct;

        let current_streak = room
            .players
            .get(player_id)
            .map(|p| p.streak)
            .unwrap_or(0);

        let (points_awarded, new_streak) = ScoreCalculator::calculate(
            is_correct,
            question.points,
            clamped_elapsed_ms,
            total_duration_ms,
            current_streak,
        );

        // Update player score & stats
        if let Some(player) = room.players.get_mut(player_id) {
            player.score += points_awarded;
            player.streak = new_streak;
            player.total_answered += 1;
            player.total_time_ms += clamped_elapsed_ms;
            if is_correct {
                player.correct_count += 1;
            }

            if let Some(ref tx) = player.tx {
                let _ = tx.send(ServerMessage::AnswerAccepted {
                    question_index: q_idx + 1,
                    response_time_ms: clamped_elapsed_ms,
                });
            }
        }

        // Store answer in room state
        let player_answer = PlayerAnswer {
            option_id: canonical_option_id,
            response_time_ms: clamped_elapsed_ms,
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
