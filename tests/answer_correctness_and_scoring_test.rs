use ezsn_quiz::db::{self, models::*};
use ezsn_quiz::game::manager::RoomManager;
use ezsn_quiz::game::room::GameRoom;
use ezsn_quiz::game::types::*;
use ezsn_quiz::ws::messages::ServerMessage;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_correct_answer_calculation_and_flexible_question_id() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be configured in .env or environment to run tests");
    let pool = db::init_db(&db_url).await.expect("Failed to init postgres db");
    let manager = Arc::new(RoomManager::new(pool.clone()));

    // 1. Create Quiz: "Which planet is known as the Red Planet?"
    let quiz_id = db::create_quiz(
        &pool,
        CreateQuizRequest {
            title: "Planetary Science Quiz".to_string(),
            description: Some("Solar system trivia".to_string()),
            questions: vec![
                CreateQuestionRequest {
                    question_text: "Which planet is known as the Red Planet?".to_string(),
                    time_limit_sec: Some(60),
                    points: Some(1000),
                    options: vec![
                        CreateOptionRequest {
                            option_text: "Venus".to_string(),
                            is_correct: false,
                        },
                        CreateOptionRequest {
                            option_text: "Mars".to_string(),
                            is_correct: true,
                        },
                        CreateOptionRequest {
                            option_text: "Jupiter".to_string(),
                            is_correct: false,
                        },
                        CreateOptionRequest {
                            option_text: "Saturn".to_string(),
                            is_correct: false,
                        },
                    ],
                },
            ],
            ..Default::default()
        },
    )
    .await
    .expect("Failed to create quiz");

    let quiz = db::get_quiz_by_id(&pool, &quiz_id).await.unwrap().unwrap();
    let mars_opt = quiz.questions[0].options.iter().find(|o| o.option_text == "Mars").unwrap();
    let mars_opt_id = mars_opt.id.clone();
    assert!(mars_opt.is_correct, "Mars must be marked is_correct=true");

    // 2. Create Room & Host
    let (pin, _host_token, _game_id) = manager.create_room(quiz.clone()).await.expect("Room creation failed");
    let room_arc = manager.get_room(&pin).await.expect("Room should exist");

    let (host_tx, mut host_rx) = mpsc::unbounded_channel::<ServerMessage>();
    {
        let mut room = room_arc.write().await;
        room.register_host(host_tx);
    }
    let _ = host_rx.recv().await; // Connected
    let _ = host_rx.recv().await; // ThemeConfig
    let _ = host_rx.recv().await; // LobbyState

    // 3. Register Player "Ezhil"
    let (p_tx, mut p_rx) = mpsc::unbounded_channel::<ServerMessage>();
    let player_id = {
        let mut room = room_arc.write().await;
        room.register_player(None, "Ezhil".to_string(), None, p_tx).expect("Join failed")
    };
    let _ = p_rx.recv().await; // Connected
    let _ = p_rx.recv().await; // ThemeConfig
    let _ = p_rx.recv().await; // LobbyState

    // 4. Set question active
    {
        let mut room = room_arc.write().await;
        room.state = RoomState::QuestionActive {
            question_index: 0,
            started_at: Instant::now() - Duration::from_millis(500),
            duration_secs: 60,
            player_answers: std::collections::HashMap::new(),
        };
    }

    // 5. Player submits answer "Mars" using frontend format `question_id = "q_1"`
    let ans_result = GameRoom::submit_answer(room_arc.clone(), &player_id, "q_1", &mars_opt_id).await;
    assert!(ans_result.is_ok(), "Answer submission with question_id='q_1' must succeed, but got error: {:?}", ans_result.err());

    // Verify player received AnswerAccepted
    let acc_msg = p_rx.recv().await.expect("Should receive AnswerAccepted");
    match acc_msg {
        ServerMessage::AnswerAccepted { question_index, .. } => {
            assert_eq!(question_index, 1);
        }
        other => panic!("Expected AnswerAccepted, got {:?}", other),
    }

    // 6. Verify player internal score before reveal
    {
        let room = room_arc.read().await;
        let player = room.players.get(&player_id).unwrap();
        assert!(player.score > 0, "Player score must be > 0 for correct answer");
        assert_eq!(player.streak, 1, "Streak must be 1");
        assert_eq!(player.correct_count, 1, "Correct count must be 1");
    }

    // 7. End question and broadcast QuestionResult
    let token = {
        let room = room_arc.read().await;
        room.timer_token
    };
    GameRoom::end_question_if_active(room_arc.clone(), 0, token);

    // Give background async task time to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 8. Verify QuestionResult message sent to Player
    let res_msg = p_rx.recv().await.expect("Should receive QuestionResult");
    match res_msg {
        ServerMessage::QuestionResult {
            question_index,
            correct_option_ids,
            option_counts,
            player_correct,
            points_awarded,
            total_score,
            streak,
        } => {
            assert_eq!(question_index, 1);
            assert!(correct_option_ids.contains(&mars_opt_id), "Correct options must contain Mars");
            assert_eq!(option_counts.get(&mars_opt_id), Some(&1), "Mars must have 1 vote!");
            assert_eq!(player_correct, Some(true), "Player must be evaluated as CORRECT (not false/Not Quite Right)");
            assert!(points_awarded.unwrap() > 0, "Points awarded must be > 0");
            assert!(total_score.unwrap() > 0, "Total score must be > 0");
            assert_eq!(streak, Some(1), "Streak must be 1");
        }
        other => panic!("Expected QuestionResult, got {:?}", other),
    }

    // 9. Clean up quiz
    let _ = db::delete_quiz(&pool, &quiz_id).await;
}
