use crate::db::models::{Organization, UpdateOrganizationRequest};
use crate::db::DbPool;

pub async fn get_default_organization(pool: &DbPool) -> Result<Organization, sqlx::Error> {
    let org = sqlx::query_as::<_, Organization>(
        r#"
        SELECT id, name, logo_url, primary_color, secondary_color, background_color, button_color, created_at, updated_at
        FROM organizations
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    if let Some(o) = org {
        Ok(o)
    } else {
        // Fallback: create default if missing
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, logo_url, primary_color, secondary_color, background_color, button_color)
            VALUES ('org_default', 'EZSN Quiz Platform', '', '#6366f1', '#ec4899', '#0f172a', '#3b82f6')
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .execute(pool)
        .await?;

        let o = sqlx::query_as::<_, Organization>(
            r#"
            SELECT id, name, logo_url, primary_color, secondary_color, background_color, button_color, created_at, updated_at
            FROM organizations
            WHERE id = 'org_default'
            "#,
        )
        .fetch_one(pool)
        .await?;

        Ok(o)
    }
}

pub async fn update_organization(
    pool: &DbPool,
    org_id: &str,
    req: UpdateOrganizationRequest,
) -> Result<Organization, sqlx::Error> {
    let primary = req.primary_color.unwrap_or_else(|| "#6366f1".to_string());
    let secondary = req.secondary_color.unwrap_or_else(|| "#ec4899".to_string());
    let bg = req.background_color.unwrap_or_else(|| "#0f172a".to_string());
    let btn = req.button_color.unwrap_or_else(|| "#3b82f6".to_string());

    sqlx::query(
        r#"
        UPDATE organizations
        SET name = $1, logo_url = $2, primary_color = $3, secondary_color = $4, background_color = $5, button_color = $6, updated_at = CURRENT_TIMESTAMP
        WHERE id = $7
        "#,
    )
    .bind(&req.name)
    .bind(&req.logo_url)
    .bind(&primary)
    .bind(&secondary)
    .bind(&bg)
    .bind(&btn)
    .bind(org_id)
    .execute(pool)
    .await?;

    let updated = sqlx::query_as::<_, Organization>(
        r#"
        SELECT id, name, logo_url, primary_color, secondary_color, background_color, button_color, created_at, updated_at
        FROM organizations
        WHERE id = $1
        "#,
    )
    .bind(org_id)
    .fetch_one(pool)
    .await?;

    Ok(updated)
}
