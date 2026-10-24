use ezsn_quiz::db::{self, models::*};
use ezsn_quiz::game::manager::RoomManager;
use ezsn_quiz::game::room::GameRoom;
use ezsn_quiz::game::types::*;
use ezsn_quiz::ws::messages::ServerMessage;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_gameplay_flow_no_immediate_feedback_and_final_review() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be configured in .env or environment to run tests");
    let pool = db::init_db(&db_url).await.expect("Failed to init postgres db");
    let manager = Arc::new(RoomManager::new(pool.clone()));

    // Create a 2-question quiz
    let quiz_id = db::create_quiz(
        &pool,
        CreateQuizRequest {
            title: "Review Flow Integration Quiz".to_string(),
            description: Some("Testing review and timing".to_string()),
            questions: vec![
                CreateQuestionRequest {
                    question_text: "Capital of France?".to_string(),
                    time_limit_sec: Some(10),
                    points: Some(1000),
                    options: vec![
                        CreateOptionRequest {
                            option_text: "Paris".to_string(),
                            is_correct: true,
                        },
                        CreateOptionRequest {
                            option_text: "Rome".to_string(),
                            is_correct: false,
                        },
                    ],
                },
                CreateQuestionRequest {
                    question_text: "What is 10 x 10?".to_string(),
                    time_limit_sec: Some(10),
                    points: Some(1000),
                    options: vec![
                        CreateOptionRequest {
                            option_text: "100".to_string(),
                            is_correct: true,
                        },
                        CreateOptionRequest {
                            option_text: "1000".to_string(),
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

    // 1. Create Room & Register Host and Player
    let (pin, _host_token, game_id) = manager.create_room(quiz.clone()).await.expect("Failed to create room");
    let room_arc = manager.get_room(&pin).await.expect("Room should exist");

    let (host_tx, mut _host_rx) = mpsc::unbounded_channel::<ServerMessage>();
    {
        let mut room = room_arc.write().await;
        room.register_host(host_tx);
    }

    let (player_tx, mut player_rx) = mpsc::unbounded_channel::<ServerMessage>();
    let player_id = {
        let mut room = room_arc.write().await;
        room.register_player(None, "Dan".to_string(), Some("dan@test.com".to_string()), player_tx)
            .expect("Player should join")
    };

    // Drain initial messages (connected, theme, lobby_state)
    while let Ok(_) = player_rx.try_recv() {}

    // 2. Question 1 Active
    let q0_correct = quiz.questions[0].options[0].id.clone();
    {
        let mut room = room_arc.write().await;
        room.state = RoomState::QuestionActive {
            question_index: 0,
            started_at: std::time::Instant::now(),
            duration_secs: 10,
            player_answers: std::collections::HashMap::new(),
        };
    }

    // Submit correct answer for Q1
    let ans1 = GameRoom::submit_answer(room_arc.clone(), &player_id, &quiz.questions[0].id, &q0_correct).await;
    assert!(ans1.is_ok());

    // Verify AnswerAccepted received
    let msg1 = player_rx.recv().await.unwrap();
    match msg1 {
        ServerMessage::AnswerAccepted { question_index, response_time_ms } => {
            assert_eq!(question_index, 1);
            assert!(response_time_ms <= 10_000);
        }
        _ => panic!("Expected AnswerAccepted"),
    }

    // 3. Question 2 Active
    let q1_wrong = quiz.questions[1].options[1].id.clone();
    {
        let mut room = room_arc.write().await;
        room.state = RoomState::QuestionActive {
            question_index: 1,
            started_at: std::time::Instant::now(),
            duration_secs: 10,
            player_answers: std::collections::HashMap::new(),
        };
    }

    // Submit incorrect answer for Q2
    let ans2 = GameRoom::submit_answer(room_arc.clone(), &player_id, &quiz.questions[1].id, &q1_wrong).await;
    assert!(ans2.is_ok());

    // 4. Finalize Game
    GameRoom::finalize_game(room_arc.clone(), 99).await;

    // Verify Player receives Podium message with full review and accurate server timing
    let mut received_podium = false;
    while let Ok(msg) = player_rx.try_recv() {
        if let ServerMessage::Podium { podium, full_results, my_rank, player_summary } = msg {
            received_podium = true;
            assert_eq!(podium.len(), 1);
            assert_eq!(full_results.len(), 1);
            assert_eq!(my_rank, Some(1));

            let summary = player_summary.expect("Player should receive summary review");
            assert_eq!(summary.total_questions, 2);
            assert_eq!(summary.correct_count, 1);
            assert_eq!(summary.incorrect_count, 1);
            assert_eq!(summary.accuracy_percentage, 50.0);
            assert_eq!(summary.final_rank, 1);
            assert_eq!(summary.reviews.len(), 2);

            // Review for Q1 (Correct)
            assert!(summary.reviews[0].is_correct);
            assert_eq!(summary.reviews[0].selected_option_id, Some(q0_correct.clone()));
            assert_eq!(summary.reviews[0].correct_option_id, q0_correct.clone());
            assert!(summary.reviews[0].points_awarded > 0);

            // Review for Q2 (Incorrect)
            assert!(!summary.reviews[1].is_correct);
            assert_eq!(summary.reviews[1].selected_option_id, Some(q1_wrong.clone()));
            assert_eq!(summary.reviews[1].points_awarded, 0);

            break;
        }
    }
    assert!(received_podium, "Player must receive Podium message with comprehensive review");

    // 5. Test Persistence via get_player_game_review
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let persisted_review = db::get_player_game_review(&pool, &game_id, &player_id)
        .await
        .expect("Database query should succeed")
        .expect("Persisted review should be found");

    assert_eq!(persisted_review.total_questions, 2);
    assert_eq!(persisted_review.correct_count, 1);
    assert_eq!(persisted_review.accuracy_percentage, 50.0);
    assert_eq!(persisted_review.reviews.len(), 2);
    assert!(persisted_review.reviews[0].is_correct);
    assert!(!persisted_review.reviews[1].is_correct);

    // Clean up test quiz
    let _ = db::delete_quiz(&pool, &quiz_id).await;
}
