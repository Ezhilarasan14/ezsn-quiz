use ezsn_quiz::db::{self, models::*};
use ezsn_quiz::game::manager::RoomManager;
use ezsn_quiz::game::room::GameRoom;
use ezsn_quiz::game::types::*;
use ezsn_quiz::ws::messages::ServerMessage;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_high_concurrency_100_players() {
    let pool = db::init_db("sqlite::memory:").await.expect("Failed to init in-memory db");
    let manager = Arc::new(RoomManager::new(pool.clone()));

    // Create a 3-question quiz
    let quiz_id = db::create_quiz(
        &pool,
        CreateQuizRequest {
            title: "Concurrency Load Test Quiz".to_string(),
            description: Some("100 concurrent players test".to_string()),
            questions: vec![
                CreateQuestionRequest {
                    question_text: "Load Question 1".to_string(),
                    time_limit_sec: Some(20),
                    points: Some(1000),
                    options: vec![
                        CreateOptionRequest { option_text: "Option A".to_string(), is_correct: true },
                        CreateOptionRequest { option_text: "Option B".to_string(), is_correct: false },
                        CreateOptionRequest { option_text: "Option C".to_string(), is_correct: false },
                        CreateOptionRequest { option_text: "Option D".to_string(), is_correct: false },
                    ],
                },
            ],
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let quiz = db::get_quiz_by_id(&pool, &quiz_id).await.unwrap().unwrap();
    let (pin, _token, _game_id) = manager.create_room(quiz.clone()).await.unwrap();
    let room_arc = manager.get_room(&pin).await.unwrap();

    // Spawn 100 concurrent player registrations
    let mut player_ids = Vec::new();
    let num_players = 100;

    for i in 0..num_players {
        let (tx, _rx) = mpsc::unbounded_channel::<ServerMessage>();
        let mut room = room_arc.write().await;
        let pid = room
            .register_player(None, format!("Player_{:03}", i), tx)
            .expect("Registration should succeed");
        player_ids.push(pid);
    }

    assert_eq!(room_arc.read().await.active_player_count(), num_players);

    // Set question active
    let opt_correct = quiz.questions[0].options[0].id.clone();
    let opt_wrong = quiz.questions[0].options[1].id.clone();
    {
        let mut room = room_arc.write().await;
        room.state = RoomState::QuestionActive {
            question_index: 0,
            started_at: std::time::Instant::now(),
            duration_secs: 20,
            player_answers: std::collections::HashMap::new(),
        };
    }

    // Submit 100 answers concurrently via Tokio tasks
    let mut tasks = Vec::new();
    for (i, pid) in player_ids.iter().enumerate() {
        let room_clone = room_arc.clone();
        let pid_clone = pid.clone();
        let opt_id = if i % 2 == 0 { opt_correct.clone() } else { opt_wrong.clone() };

        tasks.push(tokio::spawn(async move {
            GameRoom::submit_answer(room_clone, &pid_clone, "", &opt_id).await
        }));
    }

    for task in tasks {
        let res = task.await.expect("Task should join cleanly");
        assert!(res.is_ok(), "Answer submission should succeed");
    }

    // Verify all 100 answers were processed
    {
        let room = room_arc.read().await;
        let mut correct_count = 0;
        let mut incorrect_count = 0;

        for p in room.players.values() {
            if p.score > 0 {
                correct_count += 1;
            } else {
                incorrect_count += 1;
            }
        }

        assert_eq!(correct_count, 50);
        assert_eq!(incorrect_count, 50);
    }
}
