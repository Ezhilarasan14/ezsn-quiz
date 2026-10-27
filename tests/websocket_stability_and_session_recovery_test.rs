use ezsn_quiz::db::{self, models::*};
use ezsn_quiz::game::manager::RoomManager;
use ezsn_quiz::game::room::GameRoom;
use ezsn_quiz::game::types::*;
use ezsn_quiz::ws::messages::ServerMessage;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_websocket_stability_3hr_session_and_reconnection_recovery() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be configured in .env or environment to run tests");
    let pool = db::init_db(&db_url).await.expect("Failed to init postgres db");
    let manager = Arc::new(RoomManager::new(pool.clone()));

    // 1. Create a quiz with 60-second questions
    let quiz_id = db::create_quiz(
        &pool,
        CreateQuizRequest {
            title: "3-Hour Session Stability Test".to_string(),
            description: Some("Automated WebSocket stability test".to_string()),
            questions: vec![
                CreateQuestionRequest {
                    question_text: "What is the capital of Tamil Nadu?".to_string(),
                    time_limit_sec: Some(60),
                    points: Some(1000),
                    options: vec![
                        CreateOptionRequest {
                            option_text: "Chennai".to_string(),
                            is_correct: true,
                        },
                        CreateOptionRequest {
                            option_text: "Madurai".to_string(),
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

    // 2. Create room
    let (pin, _host_token, _game_id) = manager.create_room(quiz.clone()).await.expect("Room creation failed");
    let room_arc = manager.get_room(&pin).await.expect("Room should exist");

    // 3. Verify Room Lifetime configuration supports 3-hour sessions
    {
        let room = room_arc.read().await;
        // Default max lifetime must be at least 4 hours (14400s), here 5 hours (18000s)
        assert!(room.max_room_lifetime_secs >= 14400, "Max room lifetime must exceed 3 hours");
        assert!(room.room_abandoned_timeout_secs >= 1800, "Abandoned timeout should be at least 30 minutes");
        assert!(!room.is_abandoned(), "New room must not be abandoned");
    }

    // 4. Verify 3-Hour Simulation: An active room with periodic heartbeats/touch is NOT abandoned
    {
        let mut room = room_arc.write().await;
        // Simulate room created 3 hours ago (10800 seconds)
        room.created_at = Instant::now() - Duration::from_secs(10800);
        // But heartbeat / activity was touched recently (e.g. 5 seconds ago)
        room.touch();
        assert!(!room.is_abandoned(), "Active room at 3 hours must NOT be abandoned");
    }

    // 5. Test Host Reconnection Concurrency Protection
    let host_conn_1 = "conn-host-001".to_string();
    let host_conn_2 = "conn-host-002".to_string();
    let (host_tx_1, _host_rx_1) = mpsc::unbounded_channel::<ServerMessage>();
    let (host_tx_2, _host_rx_2) = mpsc::unbounded_channel::<ServerMessage>();

    {
        let mut room = room_arc.write().await;
        // Host connects with conn_1
        room.register_host_with_conn(host_tx_1, Some(host_conn_1.clone()));
        assert!(room.host_tx.is_some());
        assert_eq!(room.host_connection_id.as_deref(), Some(host_conn_1.as_str()));

        // Host reconnects with conn_2
        room.register_host_with_conn(host_tx_2, Some(host_conn_2.clone()));
        assert_eq!(room.host_connection_id.as_deref(), Some(host_conn_2.as_str()));

        // Stale conn_1 cleanup task finally fires
        room.disconnect_host_conn(Some(&host_conn_1));
        // Host must STILL be connected because conn_2 is active!
        assert!(room.host_tx.is_some(), "Host should remain connected after stale connection cleanup");
        assert_eq!(room.host_connection_id.as_deref(), Some(host_conn_2.as_str()));
    }

    // 6. Test Player Reconnection Concurrency Protection
    let p_conn_1 = "conn-p1-001".to_string();
    let p_conn_2 = "conn-p1-002".to_string();
    let (p1_tx_1, _p1_rx_1) = mpsc::unbounded_channel::<ServerMessage>();
    let (p1_tx_2, mut p1_rx_2) = mpsc::unbounded_channel::<ServerMessage>();

    let player_id = {
        let mut room = room_arc.write().await;
        let pid = room
            .register_player_with_conn(None, "Ramesh".to_string(), Some("ramesh@test.com".to_string()), p1_tx_1, Some(p_conn_1.clone()))
            .expect("Player should join");
        assert_eq!(room.active_player_count(), 1);

        // Player reconnects with conn_2
        let rec_id = room
            .register_player_with_conn(Some(pid.clone()), "Ramesh".to_string(), Some("ramesh@test.com".to_string()), p1_tx_2, Some(p_conn_2.clone()))
            .expect("Player should reconnect");
        assert_eq!(rec_id, pid);

        // Stale p_conn_1 cleanup fires
        room.disconnect_player_conn(&pid, Some(&p_conn_1));
        // Player must STILL be connected with conn_2!
        assert_eq!(room.active_player_count(), 1, "Player must remain connected after stale connection drops");

        let p_session = room.players.get(&pid).unwrap();
        assert!(p_session.is_connected);
        assert!(p_session.tx.is_some());
        assert_eq!(p_session.connection_id.as_deref(), Some(p_conn_2.as_str()));
        pid
    };

    // Drain connected message on conn_2
    let _ = p1_rx_2.recv().await; // Connected
    let _ = p1_rx_2.recv().await; // ThemeConfig

    // 7. Verify State Recovery during Active Question with `already_answered` flag
    let correct_opt_id = quiz.questions[0].options[0].id.clone();
    {
        let mut room = room_arc.write().await;
        room.state = RoomState::QuestionActive {
            question_index: 0,
            started_at: Instant::now() - Duration::from_secs(15),
            duration_secs: 60,
            player_answers: std::collections::HashMap::new(),
        };
    }

    // Player submits answer
    let ans_res = GameRoom::submit_answer(room_arc.clone(), &player_id, "", &correct_opt_id).await;
    assert!(ans_res.is_ok(), "Answer submission should succeed");

    // Player reconnects on conn_3
    let p_conn_3 = "conn-p1-003".to_string();
    let (p1_tx_3, mut p1_rx_3) = mpsc::unbounded_channel::<ServerMessage>();
    {
        let mut room = room_arc.write().await;
        let _ = room
            .register_player_with_conn(Some(player_id.clone()), "Ramesh".to_string(), None, p1_tx_3, Some(p_conn_3))
            .expect("Reconnection should succeed");
    }

    // Conn 3 receives Connected -> ThemeConfig -> QuestionActive
    let msg1 = p1_rx_3.recv().await.unwrap();
    match msg1 {
        ServerMessage::Connected { is_reconnect, .. } => assert!(is_reconnect),
        _ => panic!("Expected Connected message"),
    }
    let _ = p1_rx_3.recv().await.unwrap(); // ThemeConfig
    let msg3 = p1_rx_3.recv().await.unwrap(); // QuestionActive
    match msg3 {
        ServerMessage::QuestionActive {
            question_index,
            time_limit_sec,
            already_answered,
            ..
        } => {
            assert_eq!(question_index, 1);
            assert!(already_answered, "Player already answered, state sync must report already_answered=true");
            assert!(time_limit_sec <= 45, "Remaining time must be accurately calculated");
        }
        other => panic!("Expected QuestionActive, got: {:?}", other),
    }

    // Clean up
    let _ = db::delete_quiz(&pool, &quiz_id).await;
}
