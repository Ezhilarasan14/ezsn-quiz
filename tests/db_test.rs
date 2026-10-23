use ezsn_quiz::db::{self, models::*, FinalPlayerResult};

#[tokio::test]
async fn test_database_crud_and_history() {
    // Use an in-memory SQLite database for isolated testing
    let pool = db::init_db("sqlite::memory:")
        .await
        .expect("Failed to init in-memory db");

    // 1. Initial seed should have 2 default quizzes
    let initial_quizzes = db::list_quizzes(&pool).await.expect("Failed to list quizzes");
    assert!(initial_quizzes.len() >= 2);

    // 2. Create a new custom quiz
    let create_req = CreateQuizRequest {
        title: "Test Rust Concurrency Quiz".to_string(),
        description: Some("Testing async channels and locks".to_string()),
        questions: vec![
            CreateQuestionRequest {
                question_text: "What is an Arc in Rust?".to_string(),
                time_limit_sec: Some(15),
                points: Some(1000),
                options: vec![
                    CreateOptionRequest {
                        option_text: "Atomic Reference Counted pointer".to_string(),
                        is_correct: true,
                    },
                    CreateOptionRequest {
                        option_text: "Automatic Garbage Collector".to_string(),
                        is_correct: false,
                    },
                ],
            },
        ],
    };

    let quiz_id = db::create_quiz(&pool, create_req).await.expect("Failed to create quiz");
    assert!(!quiz_id.is_empty());

    // 3. Fetch the created quiz
    let fetched = db::get_quiz_by_id(&pool, &quiz_id).await.expect("Failed to get quiz");
    assert!(fetched.is_some());
    let quiz_detail = fetched.unwrap();
    assert_eq!(quiz_detail.title, "Test Rust Concurrency Quiz");
    assert_eq!(quiz_detail.questions.len(), 1);
    assert_eq!(quiz_detail.questions[0].options.len(), 2);
    assert!(quiz_detail.questions[0].options[0].is_correct);

    // 4. Duplicate the quiz
    let duplicated_id = db::duplicate_quiz(&pool, &quiz_id).await.expect("Failed to duplicate");
    assert!(duplicated_id.is_some());
    let dup_id = duplicated_id.unwrap();
    let dup_quiz = db::get_quiz_by_id(&pool, &dup_id).await.unwrap().unwrap();
    assert_eq!(dup_quiz.title, "Test Rust Concurrency Quiz (Copy)");

    // 5. Update original quiz
    let update_req = UpdateQuizRequest {
        title: "Updated Rust Concurrency Quiz".to_string(),
        description: Some("Updated description".to_string()),
        questions: vec![
            CreateQuestionRequest {
                question_text: "Updated question?".to_string(),
                time_limit_sec: Some(25),
                points: Some(2000),
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
    };
    let updated = db::update_quiz(&pool, &quiz_id, update_req).await.expect("Failed to update");
    assert!(updated);

    let verify_updated = db::get_quiz_by_id(&pool, &quiz_id).await.unwrap().unwrap();
    assert_eq!(verify_updated.title, "Updated Rust Concurrency Quiz");
    assert_eq!(verify_updated.questions[0].points, 2000);

    // 6. Test game recording and results persistence
    let game_id = uuid::Uuid::new_v4().to_string();
    let room_pin = "654321";
    let host_token = "token-xyz";

    db::create_game_record(&pool, &game_id, &quiz_id, room_pin, host_token)
        .await
        .expect("Failed to create game record");

    let results = vec![
        FinalPlayerResult {
            player_id: "p1".to_string(),
            nickname: "FerrisChampion".to_string(),
            total_score: 3800,
            final_rank: 1,
            correct_count: 4,
            total_answered: 4,
        },
        FinalPlayerResult {
            player_id: "p2".to_string(),
            nickname: "CrabbyPlayer".to_string(),
            total_score: 2900,
            final_rank: 2,
            correct_count: 3,
            total_answered: 4,
        },
    ];

    db::update_game_status(&pool, &game_id, "completed", 2).await.expect("Failed to update status");
    db::save_game_results(&pool, &game_id, &results).await.expect("Failed to save results");

    let history = db::list_game_history(&pool).await.expect("Failed to list history");
    assert!(!history.is_empty());
    let latest_game = &history[0];
    assert_eq!(latest_game.room_pin, "654321");
    assert_eq!(latest_game.winner_nickname.as_deref(), Some("FerrisChampion"));
    assert_eq!(latest_game.winner_score, Some(3800));

    let player_results = db::get_game_results(&pool, &game_id).await.expect("Failed to get player results");
    assert_eq!(player_results.len(), 2);
    assert_eq!(player_results[0].nickname, "FerrisChampion");
    assert_eq!(player_results[0].final_rank, 1);

    // 7. Delete quiz
    let deleted = db::delete_quiz(&pool, &quiz_id).await.expect("Failed to delete quiz");
    assert!(deleted);
    let check_deleted = db::get_quiz_by_id(&pool, &quiz_id).await.expect("Failed to fetch deleted");
    assert!(check_deleted.is_none());
}
