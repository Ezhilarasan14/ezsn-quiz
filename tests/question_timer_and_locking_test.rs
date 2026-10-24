use ezsn_quiz::config::Config;
use ezsn_quiz::db::{self, models::*};
use ezsn_quiz::game::manager::RoomManager;
use ezsn_quiz::game::room::GameRoom;
use ezsn_quiz::game::types::*;
use ezsn_quiz::ws::messages::ServerMessage;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_configured_question_answer_time_and_lock() {
    dotenvy::dotenv().ok();
    // 1. Validate Config loading of QUIZ_QUESTION_TIME_SECONDS=5
    std::env::set_var("QUIZ_QUESTION_TIME_SECONDS", "5");
    let config = Config::from_env();
    assert_eq!(config.quiz_question_time_seconds, 5);
    assert!(config.validate().is_ok());

    // Validation fails on 0 or excessive value
    let invalid_config = Config {
        quiz_question_time_seconds: 0,
        ..config.clone()
    };
    assert!(invalid_config.validate().is_err());

    let excessive_config = Config {
        quiz_question_time_seconds: 301,
        ..config.clone()
    };
    assert!(excessive_config.validate().is_err());

    // 2. Database & RoomManager setup
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be configured in .env or environment to run tests");
    let pool = db::init_db(&db_url).await.expect("Failed to init postgres db");

    let manager = Arc::new(RoomManager::with_question_time(pool.clone(), 5));
    assert_eq!(manager.question_time_secs, 5);

    // Create a 2-question quiz
    let quiz_id = db::create_quiz(
        &pool,
        CreateQuizRequest {
            title: "5s Answer Time Test Quiz".to_string(),
            description: Some("Testing 5-second question timeout & locking".to_string()),
            questions: vec![
                CreateQuestionRequest {
                    question_text: "What is 5 x 5?".to_string(),
                    time_limit_sec: Some(20), // configured time (5s) will be applied
                    points: Some(1000),
                    options: vec![
                        CreateOptionRequest { option_text: "25".to_string(), is_correct: true },
                        CreateOptionRequest { option_text: "50".to_string(), is_correct: false },
                    ],
                },
                CreateQuestionRequest {
                    question_text: "What is 3 + 2?".to_string(),
                    time_limit_sec: Some(20),
                    points: Some(1000),
                    options: vec![
                        CreateOptionRequest { option_text: "5".to_string(), is_correct: true },
                        CreateOptionRequest { option_text: "6".to_string(), is_correct: false },
                    ],
                },
            ],
            ..Default::default()
        },
    )
    .await
    .expect("Failed to create test quiz");

    let quiz = db::get_quiz_by_id(&pool, &quiz_id).await.unwrap().unwrap();
    let (pin, _host_token, _game_id) = manager.create_room(quiz.clone()).await.expect("Failed to create room");
    let room_arc = manager.get_room(&pin).await.expect("Room should exist");

    // Verify room has 5-second question answer time configured
    {
        let room = room_arc.read().await;
        assert_eq!(room.question_time_secs, 5);
        for q in &room.quiz.questions {
            assert_eq!(q.time_limit_sec, 5);
        }
    }

    let (player_tx, mut player_rx) = mpsc::unbounded_channel::<ServerMessage>();
    let player_id = {
        let mut room = room_arc.write().await;
        room.register_player(None, "Ravi".to_string(), None, player_tx)
            .expect("Player should register")
    };

    // Drain initial messages
    while let Ok(_) = player_rx.try_recv() {}

    // 3. Question 1 Active: Answer submitted within time limit (e.g. 500ms)
    let q0_opt_correct = quiz.questions[0].options[0].id.clone();
    {
        let mut room = room_arc.write().await;
        room.state = RoomState::QuestionActive {
            question_index: 0,
            started_at: std::time::Instant::now(),
            duration_secs: 5,
            player_answers: std::collections::HashMap::new(),
        };
    }

    // Submit answer in time
    let res1 = GameRoom::submit_answer(room_arc.clone(), &player_id, &quiz.questions[0].id, &q0_opt_correct).await;
    assert!(res1.is_ok());

    // Verify answer is accepted and response time is accurately recorded on server
    {
        let room = room_arc.read().await;
        let p_ans = room.all_answers.get(&0).unwrap().get(&player_id).unwrap();
        assert!(p_ans.is_correct);
        assert!(p_ans.response_time_ms < 5000);
        assert!(p_ans.points_awarded > 0);
    }

    // 4. Question 2 Active: Answer submitted AFTER time expires (> 5s)
    let q1_opt_correct = quiz.questions[1].options[0].id.clone();
    {
        let mut room = room_arc.write().await;
        // Simulate question started 5.2 seconds ago (timer expired)
        room.state = RoomState::QuestionActive {
            question_index: 1,
            started_at: std::time::Instant::now() - std::time::Duration::from_millis(5200),
            duration_secs: 5,
            player_answers: std::collections::HashMap::new(),
        };
    }

    // Attempt to submit after timer expired
    let res2 = GameRoom::submit_answer(room_arc.clone(), &player_id, &quiz.questions[1].id, &q1_opt_correct).await;
    assert!(res2.is_err(), "Late answer submission must be rejected");
    assert_eq!(res2.unwrap_err(), "Time is up! Answers are locked for this question.");

    // 5. Finalize and verify final review statistics
    GameRoom::finalize_game(room_arc.clone(), 123).await;

    while let Ok(msg) = player_rx.try_recv() {
        if let ServerMessage::Podium { player_summary, .. } = msg {
            let summary = player_summary.expect("Summary should be present");
            assert_eq!(summary.total_questions, 2);
            assert_eq!(summary.correct_count, 1);
            assert_eq!(summary.incorrect_count, 1);

            // Review for Q1 (answered in time)
            assert!(summary.reviews[0].is_correct);
            assert_eq!(summary.reviews[0].time_limit_sec, 5);
            assert!(summary.reviews[0].response_time_ms < 5000);

            // Review for Q2 (timed out, unanswered)
            assert!(!summary.reviews[1].is_correct);
            assert_eq!(summary.reviews[1].time_limit_sec, 5);
            assert_eq!(summary.reviews[1].response_time_ms, 5000); // capped at duration
            assert_eq!(summary.reviews[1].points_awarded, 0);
            break;
        }
    }

    // Clean up test quiz
    let _ = db::delete_quiz(&pool, &quiz_id).await;
}
