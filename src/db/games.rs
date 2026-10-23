use crate::db::models::*;
use crate::db::DbPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FinalPlayerResult {
    pub player_id: String,
    pub nickname: String,
    pub email: Option<String>,
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
        VALUES ($1, $2, $3, $4, 'lobby', CURRENT_TIMESTAMP)
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
            SET status = $1, total_players = $2, ended_at = CURRENT_TIMESTAMP
            WHERE id = $3
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
            SET status = $1, total_players = $2
            WHERE id = $3
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
            INSERT INTO game_player_results (id, game_id, player_id, nickname, email, total_score, final_rank, correct_count, total_answered)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&result_id)
        .bind(game_id)
        .bind(&res.player_id)
        .bind(&res.nickname)
        .bind(&res.email)
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
        SELECT id, game_id, player_id, nickname, email, total_score, final_rank, correct_count, total_answered
        FROM game_player_results
        WHERE game_id = $1
        ORDER BY final_rank ASC
        "#,
    )
    .bind(game_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn get_player_stats(pool: &DbPool, email: &str) -> Result<Option<PlayerStats>, sqlx::Error> {
    let clean_email = email.trim().to_lowercase();
    let history = sqlx::query_as::<_, PlayerHistoryItem>(
        r#"
        SELECT 
            r.game_id,
            q.title as quiz_title,
            g.room_pin,
            r.total_score as score,
            r.final_rank as rank,
            g.total_players,
            r.correct_count,
            r.total_answered,
            g.created_at as played_at
        FROM game_player_results r
        JOIN games g ON r.game_id = g.id
        JOIN quizzes q ON g.quiz_id = q.id
        WHERE LOWER(r.email) = $1
        ORDER BY g.created_at DESC
        LIMIT 50
        "#,
    )
    .bind(&clean_email)
    .fetch_all(pool)
    .await?;

    if history.is_empty() {
        return Ok(None);
    }

    let most_recent_nick = sqlx::query_scalar::<_, String>(
        "SELECT nickname FROM game_player_results WHERE LOWER(email) = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(&clean_email)
    .fetch_optional(pool)
    .await?
    .unwrap_or_else(|| "Player".to_string());

    let quizzes_played = history.len() as i64;
    let mut total_score = 0;
    let mut wins_count = 0;
    let mut podium_count = 0;
    let mut total_correct = 0;
    let mut total_answered = 0;

    for h in &history {
        total_score += h.score;
        if h.rank == 1 { wins_count += 1; }
        if h.rank <= 3 { podium_count += 1; }
        total_correct += h.correct_count;
        total_answered += h.total_answered;
    }

    let avg_accuracy = if total_answered > 0 {
        ((total_correct as f64) / (total_answered as f64) * 100.0).round()
    } else {
        0.0
    };

    Ok(Some(PlayerStats {
        email: clean_email,
        nickname: most_recent_nick,
        quizzes_played,
        total_score,
        wins_count,
        podium_count,
        total_correct,
        total_answered,
        avg_accuracy,
        history,
    }))
}
