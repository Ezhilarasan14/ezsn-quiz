use crate::db::models::*;
use crate::db::DbPool;
use sqlx::Row;
use uuid::Uuid;

pub async fn list_quizzes(pool: &DbPool) -> Result<Vec<QuizSummary>, sqlx::Error> {
    let rows = sqlx::query_as::<_, QuizSummary>(
        r#"
        SELECT 
            q.id,
            q.title,
            q.description,
            COUNT(qu.id) as question_count,
            q.organization_id,
            q.theme_logo_url,
            q.theme_primary_color,
            q.theme_secondary_color,
            q.theme_bg_color,
            q.theme_btn_color,
            q.host_name,
            q.branch_name,
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
        SELECT id, title, description, organization_id, theme_logo_url, theme_primary_color, theme_secondary_color, theme_bg_color, theme_btn_color, host_name, branch_name, created_at, updated_at
        FROM quizzes
        WHERE id = $1
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
    let organization_id: Option<String> = row.get("organization_id");
    let theme_logo_url: Option<String> = row.get("theme_logo_url");
    let theme_primary_color: Option<String> = row.get("theme_primary_color");
    let theme_secondary_color: Option<String> = row.get("theme_secondary_color");
    let theme_bg_color: Option<String> = row.get("theme_bg_color");
    let theme_btn_color: Option<String> = row.get("theme_btn_color");
    let host_name: Option<String> = row.try_get("host_name").ok().flatten();
    let branch_name: Option<String> = row.try_get("branch_name").ok().flatten();
    let created_at: chrono::NaiveDateTime = row.get("created_at");
    let updated_at: chrono::NaiveDateTime = row.get("updated_at");

    let questions = sqlx::query_as::<_, QuestionRow>(
        r#"
        SELECT id, quiz_id, question_text, time_limit_sec, points, order_index
        FROM questions
        WHERE quiz_id = $1
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
            WHERE question_id = $1
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
        organization_id,
        theme_logo_url,
        theme_primary_color,
        theme_secondary_color,
        theme_bg_color,
        theme_btn_color,
        host_name,
        branch_name,
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
        INSERT INTO quizzes (
            id, title, description, organization_id, 
            theme_logo_url, theme_primary_color, theme_secondary_color, theme_bg_color, theme_btn_color,
            host_name, branch_name,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&quiz_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.organization_id)
    .bind(&req.theme_logo_url)
    .bind(&req.theme_primary_color)
    .bind(&req.theme_secondary_color)
    .bind(&req.theme_bg_color)
    .bind(&req.theme_btn_color)
    .bind(&req.host_name)
    .bind(&req.branch_name)
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
            VALUES ($1, $2, $3, $4, $5, $6)
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
                VALUES ($1, $2, $3, $4, $5)
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
        SET title = $1, description = $2, organization_id = $3, 
            theme_logo_url = $4, theme_primary_color = $5, theme_secondary_color = $6, theme_bg_color = $7, theme_btn_color = $8,
            host_name = $9, branch_name = $10,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $11
        "#,
    )
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.organization_id)
    .bind(&req.theme_logo_url)
    .bind(&req.theme_primary_color)
    .bind(&req.theme_secondary_color)
    .bind(&req.theme_bg_color)
    .bind(&req.theme_btn_color)
    .bind(&req.host_name)
    .bind(&req.branch_name)
    .bind(quiz_id)
    .execute(&mut *tx)
    .await?;

    if res.rows_affected() == 0 {
        return Ok(false);
    }

    sqlx::query("DELETE FROM questions WHERE quiz_id = $1")
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
            VALUES ($1, $2, $3, $4, $5, $6)
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
                VALUES ($1, $2, $3, $4, $5)
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
    let res = sqlx::query("DELETE FROM quizzes WHERE id = $1")
        .bind(quiz_id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected() > 0)
}

/// Check and delete duplicate quizzes, keeping only the earliest created record per title
pub async fn deduplicate_quizzes(pool: &DbPool) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"
        DELETE FROM quizzes
        WHERE id NOT IN (
            SELECT DISTINCT ON (title) id
            FROM quizzes
            ORDER BY title, created_at ASC
        )
        "#,
    )
    .execute(pool)
    .await?;

    let removed = res.rows_affected();
    if removed > 0 {
        log::info!("Cleaned up {} duplicate quiz records during database check/migration.", removed);
    }
    Ok(removed)
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
        organization_id: original.organization_id,
        theme_logo_url: original.theme_logo_url,
        theme_primary_color: original.theme_primary_color,
        theme_secondary_color: original.theme_secondary_color,
        theme_bg_color: original.theme_bg_color,
        theme_btn_color: original.theme_btn_color,
        host_name: original.host_name,
        branch_name: original.branch_name,
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
