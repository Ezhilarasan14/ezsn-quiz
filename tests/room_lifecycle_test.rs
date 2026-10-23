use ezsn_quiz::db::{self, models::*};
use ezsn_quiz::game::manager::RoomManager;
use ezsn_quiz::game::room::GameRoom;
use ezsn_quiz::game::types::*;
use ezsn_quiz::ws::messages::ServerMessage;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_room_lifecycle_and_multiplayer_gameplay() {
    let pool = db::init_db("sqlite::memory:").await.expect("Failed to init in-memory db");
    let manager = Arc::new(RoomManager::new(pool.clone()));

    // Create a quiz with 2 questions
    let quiz_id = db::create_quiz(
        &pool,
        CreateQuizRequest {
            title: "Multiplayer Unit Test Quiz".to_string(),
            description: Some("Automated test quiz".to_string()),
            questions: vec![
                CreateQuestionRequest {
                    question_text: "What is 2 + 2?".to_string(),
                    time_limit_sec: Some(10),
                    points: Some(1000),
                    options: vec![
                        CreateOptionRequest {
                            option_text: "4".to_string(),
                            is_correct: true,
                        },
                        CreateOptionRequest {
                            option_text: "5".to_string(),
                            is_correct: false,
                        },
                    ],
                },
                CreateQuestionRequest {
                    question_text: "Is Rust memory-safe?".to_string(),
                    time_limit_sec: Some(10),
                    points: Some(1000),
                    options: vec![
                        CreateOptionRequest {
                            option_text: "Yes".to_string(),
                            is_correct: true,
                        },
                        CreateOptionRequest {
                            option_text: "No".to_string(),
                            is_correct: false,
                        },
                    ],
                },
            ],
        },
    )
    .await
    .expect("Failed to create quiz");

    let quiz = db::get_quiz_by_id(&pool, &quiz_id).await.unwrap().unwrap();

    // 1. Create Room
    let (pin, host_token, _game_id) = manager.create_room(quiz.clone()).await.expect("Failed to create room");
    assert_eq!(pin.len(), 6);
    assert!(!host_token.is_empty());
    assert!(manager.room_exists(&pin).await);

    let room_arc = manager.get_room(&pin).await.expect("Room should exist");

    // 2. Register Host
    let (host_tx, mut host_rx) = mpsc::unbounded_channel::<ServerMessage>();
    {
        let mut room = room_arc.write().await;
        room.register_host(host_tx);
    }
    let host_msg = host_rx.recv().await.expect("Host should receive connected message");
    match host_msg {
        ServerMessage::Connected { role, is_reconnect, .. } => {
            assert_eq!(role, "host");
            assert!(!is_reconnect);
        }
        _ => panic!("Expected Connected message for host"),
    }

    // 3. Register Player 1 ("Alice")
    let (p1_tx, _p1_rx) = mpsc::unbounded_channel::<ServerMessage>();
    let p1_id = {
        let mut room = room_arc.write().await;
        room.register_player(None, "Alice".to_string(), p1_tx).expect("Alice should join")
    };
    assert!(!p1_id.is_empty());

    // 4. Register Player 2 ("Bob")
    let (p2_tx, _p2_rx) = mpsc::unbounded_channel::<ServerMessage>();
    let p2_id = {
        let mut room = room_arc.write().await;
        room.register_player(None, "Bob".to_string(), p2_tx).expect("Bob should join")
    };
    assert!(!p2_id.is_empty());

    // Verify duplicate nickname rejection
    {
        let (dup_tx, _) = mpsc::unbounded_channel::<ServerMessage>();
        let mut room = room_arc.write().await;
        let dup_res = room.register_player(None, "alice".to_string(), dup_tx);
        assert!(dup_res.is_err(), "Duplicate nickname should be rejected");
    }

    // 5. Test Player Disconnection and Reconnection
    {
        let mut room = room_arc.write().await;
        room.disconnect_player(&p1_id);
        assert_eq!(room.active_player_count(), 1);

        // Reconnect Alice
        let (reconnect_tx, mut reconnect_rx) = mpsc::unbounded_channel::<ServerMessage>();
        let reconnected_id = room
            .register_player(Some(p1_id.clone()), "Alice".to_string(), reconnect_tx)
            .expect("Alice should reconnect");
        assert_eq!(reconnected_id, p1_id);
        assert_eq!(room.active_player_count(), 2);

        let rec_msg = reconnect_rx.recv().await.unwrap();
        match rec_msg {
            ServerMessage::Connected { is_reconnect, .. } => assert!(is_reconnect),
            _ => panic!("Expected is_reconnect=true"),
        }
    }

    // 6. Test Start Game and Active Question
    let q0_opt_correct = quiz.questions[0].options[0].id.clone();
    let q0_opt_wrong = quiz.questions[0].options[1].id.clone();

    // Directly set active question for immediate deterministic testing
    {
        let mut room = room_arc.write().await;
        room.state = RoomState::QuestionActive {
            question_index: 0,
            started_at: std::time::Instant::now(),
            duration_secs: 10,
            player_answers: std::collections::HashMap::new(),
        };
    }

    // Alice answers correctly
    let ans_res1 = GameRoom::submit_answer(room_arc.clone(), &p1_id, "", &q0_opt_correct).await;
    assert!(ans_res1.is_ok());

    // Duplicate answer rejected
    let ans_dup = GameRoom::submit_answer(room_arc.clone(), &p1_id, "", &q0_opt_correct).await;
    assert!(ans_dup.is_err(), "Duplicate answer submission must be rejected");

    // Bob answers incorrectly
    let ans_res2 = GameRoom::submit_answer(room_arc.clone(), &p2_id, "", &q0_opt_wrong).await;
    assert!(ans_res2.is_ok());

    // 7. Verify Scores & Leaderboard
    {
        let room = room_arc.read().await;
        let alice = room.players.get(&p1_id).unwrap();
        let bob = room.players.get(&p2_id).unwrap();

        assert!(alice.score > 0, "Alice should have earned points");
        assert_eq!(alice.streak, 1);
        assert_eq!(bob.score, 0, "Bob should have 0 points for incorrect answer");
        assert_eq!(bob.streak, 0);
    }
}
