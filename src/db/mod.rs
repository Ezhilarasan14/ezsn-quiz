pub mod models;

use models::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Row, Sqlite};
use std::str::FromStr;
use uuid::Uuid;

pub type DbPool = Pool<Sqlite>;

pub async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let connection_options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(connection_options)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

    log::info!("Database initialized and migrations applied successfully.");
    Ok(pool)
}

pub async fn list_quizzes(pool: &DbPool) -> Result<Vec<QuizSummary>, sqlx::Error> {
    let rows = sqlx::query_as::<_, QuizSummary>(
        r#"
        SELECT 
            q.id,
            q.title,
            q.description,
            COUNT(qu.id) as question_count,
            q.created_at,
            q.updated_at
        FROM quizzes q
        LEFT JOIN questions qu ON q.id = qu.quiz_id
        GROUP BY q.id
        ORDER BY q.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn get_quiz_by_id(pool: &DbPool, quiz_id: &str) -> Result<Option<QuizDetail>, sqlx::Error> {
    let quiz_row = sqlx::query(
        r#"
        SELECT id, title, description, created_at, updated_at
        FROM quizzes
        WHERE id = ?
        "#,
    )
    .bind(quiz_id)
    .fetch_optional(pool)
    .await?;

    let row = match quiz_row {
        Some(r) => r,
        None => return Ok(None),
    };

    let id: String = row.get("id");
    let title: String = row.get("title");
    let description: Option<String> = row.get("description");
    let created_at: chrono::NaiveDateTime = row.get("created_at");
    let updated_at: chrono::NaiveDateTime = row.get("updated_at");

    let questions = sqlx::query_as::<_, QuestionRow>(
        r#"
        SELECT id, quiz_id, question_text, time_limit_sec, points, order_index
        FROM questions
        WHERE quiz_id = ?
        ORDER BY order_index ASC
        "#,
    )
    .bind(quiz_id)
    .fetch_all(pool)
    .await?;

    let mut question_details = Vec::new();
    for q in questions {
        let options = sqlx::query_as::<_, OptionDetail>(
            r#"
            SELECT id, question_id, option_text, is_correct, order_index
            FROM options
            WHERE question_id = ?
            ORDER BY order_index ASC
            "#,
        )
        .bind(&q.id)
        .fetch_all(pool)
        .await?;

        question_details.push(QuestionDetail {
            id: q.id,
            quiz_id: q.quiz_id,
            question_text: q.question_text,
            time_limit_sec: q.time_limit_sec,
            points: q.points,
            order_index: q.order_index,
            options,
        });
    }

    Ok(Some(QuizDetail {
        id,
        title,
        description,
        questions: question_details,
        created_at,
        updated_at,
    }))
}

pub async fn create_quiz(pool: &DbPool, req: CreateQuizRequest) -> Result<String, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let quiz_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO quizzes (id, title, description, created_at, updated_at)
        VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&quiz_id)
    .bind(&req.title)
    .bind(&req.description)
    .execute(&mut *tx)
    .await?;

    for (q_idx, q) in req.questions.iter().enumerate() {
        let question_id = Uuid::new_v4().to_string();
        let time_limit = q.time_limit_sec.unwrap_or(20);
        let points = q.points.unwrap_or(1000);
        let q_idx_i64 = q_idx as i64;

        sqlx::query(
            r#"
            INSERT INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&question_id)
        .bind(&quiz_id)
        .bind(&q.question_text)
        .bind(time_limit)
        .bind(points)
        .bind(q_idx_i64)
        .execute(&mut *tx)
        .await?;

        for (opt_idx, opt) in q.options.iter().enumerate() {
            let opt_id = Uuid::new_v4().to_string();
            let opt_idx_i64 = opt_idx as i64;
            sqlx::query(
                r#"
                INSERT INTO options (id, question_id, option_text, is_correct, order_index)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(&opt_id)
            .bind(&question_id)
            .bind(&opt.option_text)
            .bind(opt.is_correct)
            .bind(opt_idx_i64)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(quiz_id)
}

pub async fn update_quiz(pool: &DbPool, quiz_id: &str, req: UpdateQuizRequest) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let res = sqlx::query(
        r#"
        UPDATE quizzes
        SET title = ?, description = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(&req.title)
    .bind(&req.description)
    .bind(quiz_id)
    .execute(&mut *tx)
    .await?;

    if res.rows_affected() == 0 {
        return Ok(false);
    }

    // Delete existing questions and options (cascades)
    sqlx::query("DELETE FROM questions WHERE quiz_id = ?")
        .bind(quiz_id)
        .execute(&mut *tx)
        .await?;

    for (q_idx, q) in req.questions.iter().enumerate() {
        let question_id = Uuid::new_v4().to_string();
        let time_limit = q.time_limit_sec.unwrap_or(20);
        let points = q.points.unwrap_or(1000);
        let q_idx_i64 = q_idx as i64;

        sqlx::query(
            r#"
            INSERT INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&question_id)
        .bind(quiz_id)
        .bind(&q.question_text)
        .bind(time_limit)
        .bind(points)
        .bind(q_idx_i64)
        .execute(&mut *tx)
        .await?;

        for (opt_idx, opt) in q.options.iter().enumerate() {
            let opt_id = Uuid::new_v4().to_string();
            let opt_idx_i64 = opt_idx as i64;
            sqlx::query(
                r#"
                INSERT INTO options (id, question_id, option_text, is_correct, order_index)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(&opt_id)
            .bind(&question_id)
            .bind(&opt.option_text)
            .bind(opt.is_correct)
            .bind(opt_idx_i64)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(true)
}

pub async fn delete_quiz(pool: &DbPool, quiz_id: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM quizzes WHERE id = ?")
        .bind(quiz_id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected() > 0)
}

pub async fn duplicate_quiz(pool: &DbPool, quiz_id: &str) -> Result<Option<String>, sqlx::Error> {
    let original = match get_quiz_by_id(pool, quiz_id).await? {
        Some(q) => q,
        None => return Ok(None),
    };

    let new_title = format!("{} (Copy)", original.title);
    let create_req = CreateQuizRequest {
        title: new_title,
        description: original.description,
        questions: original
            .questions
            .into_iter()
            .map(|q| CreateQuestionRequest {
                question_text: q.question_text,
                time_limit_sec: Some(q.time_limit_sec),
                points: Some(q.points),
                options: q
                    .options
                    .into_iter()
                    .map(|o| CreateOptionRequest {
                        option_text: o.option_text,
                        is_correct: o.is_correct,
                    })
                    .collect(),
            })
            .collect(),
    };

    let new_id = create_quiz(pool, create_req).await?;
    Ok(Some(new_id))
}

pub async fn create_game_record(
    pool: &DbPool,
    game_id: &str,
    quiz_id: &str,
    room_pin: &str,
    host_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO games (id, quiz_id, room_pin, host_token, status, created_at)
        VALUES (?, ?, ?, ?, 'lobby', CURRENT_TIMESTAMP)
        "#,
    )
    .bind(game_id)
    .bind(quiz_id)
    .bind(room_pin)
    .bind(host_token)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_game_status(
    pool: &DbPool,
    game_id: &str,
    status: &str,
    total_players: i64,
) -> Result<(), sqlx::Error> {
    if status == "completed" || status == "abandoned" {
        sqlx::query(
            r#"
            UPDATE games
            SET status = ?, total_players = ?, ended_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(total_players)
        .bind(game_id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE games
            SET status = ?, total_players = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(total_players)
        .bind(game_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct FinalPlayerResult {
    pub player_id: String,
    pub nickname: String,
    pub total_score: i64,
    pub final_rank: i64,
    pub correct_count: i64,
    pub total_answered: i64,
}

pub async fn save_game_results(
    pool: &DbPool,
    game_id: &str,
    results: &[FinalPlayerResult],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    for res in results {
        let result_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO game_player_results (id, game_id, player_id, nickname, total_score, final_rank, correct_count, total_answered)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&result_id)
        .bind(game_id)
        .bind(&res.player_id)
        .bind(&res.nickname)
        .bind(res.total_score)
        .bind(res.final_rank)
        .bind(res.correct_count)
        .bind(res.total_answered)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn list_game_history(pool: &DbPool) -> Result<Vec<GameHistoryItem>, sqlx::Error> {
    let rows = sqlx::query_as::<_, GameHistoryItem>(
        r#"
        SELECT 
            g.id,
            g.quiz_id,
            q.title as quiz_title,
            g.room_pin,
            g.total_players,
            g.created_at,
            g.ended_at,
            (SELECT nickname FROM game_player_results r WHERE r.game_id = g.id AND r.final_rank = 1 LIMIT 1) as winner_nickname,
            (SELECT total_score FROM game_player_results r WHERE r.game_id = g.id AND r.final_rank = 1 LIMIT 1) as winner_score
        FROM games g
        JOIN quizzes q ON g.quiz_id = q.id
        WHERE g.status = 'completed'
        ORDER BY g.ended_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn get_game_results(pool: &DbPool, game_id: &str) -> Result<Vec<GamePlayerResultRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, GamePlayerResultRow>(
        r#"
        SELECT id, game_id, player_id, nickname, total_score, final_rank, correct_count, total_answered
        FROM game_player_results
        WHERE game_id = ?
        ORDER BY final_rank ASC
        "#,
    )
    .bind(game_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
