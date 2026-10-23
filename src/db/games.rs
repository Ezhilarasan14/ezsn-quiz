use crate::db::models::*;
use crate::db::DbPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FinalPlayerResult {
    pub player_id: String,
    pub nickname: String,
    pub total_score: i64,
    pub final_rank: i64,
    pub correct_count: i64,
    pub total_answered: i64,
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
