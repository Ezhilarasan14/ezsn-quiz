use crate::db::models::{Organization, UpdateOrganizationRequest};
use crate::db::DbPool;

pub async fn get_default_organization(pool: &DbPool) -> Result<Organization, sqlx::Error> {
    let org = sqlx::query_as::<_, Organization>(
        r#"
        SELECT id, name, branch_name, logo_url, primary_color, secondary_color, background_color, button_color, created_at, updated_at
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
            INSERT INTO organizations (id, name, branch_name, logo_url, primary_color, secondary_color, background_color, button_color)
            VALUES ('org_default', 'EZSN Quiz Platform', 'Main Campus', '', '#6366f1', '#ec4899', '#0f172a', '#3b82f6')
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .execute(pool)
        .await?;

        let o = sqlx::query_as::<_, Organization>(
            r#"
            SELECT id, name, branch_name, logo_url, primary_color, secondary_color, background_color, button_color, created_at, updated_at
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
    // 1. Resolve target organization row:
    // First try matching `org_id`, then fallback to the first created organization
    let target_org = if !org_id.trim().is_empty() {
        sqlx::query_as::<_, Organization>(
            r#"
            SELECT id, name, branch_name, logo_url, primary_color, secondary_color, background_color, button_color, created_at, updated_at
            FROM organizations
            WHERE id = $1
            "#,
        )
        .bind(org_id)
        .fetch_optional(pool)
        .await?
    } else {
        None
    };

    let target = match target_org {
        Some(o) => o,
        None => {
            // Check if any default organization exists in the table
            if let Some(existing) = sqlx::query_as::<_, Organization>(
                r#"
                SELECT id, name, branch_name, logo_url, primary_color, secondary_color, background_color, button_color, created_at, updated_at
                FROM organizations
                ORDER BY created_at ASC
                LIMIT 1
                "#,
            )
            .fetch_optional(pool)
            .await? {
                existing
            } else {
                // Table is empty -> create initial row
                let initial_id = if org_id.trim().is_empty() { "org_default" } else { org_id };
                sqlx::query(
                    r#"
                    INSERT INTO organizations (id, name, branch_name, logo_url, primary_color, secondary_color, background_color, button_color)
                    VALUES ($1, $2, $3, $4, '#6366f1', '#ec4899', '#0f172a', '#3b82f6')
                    ON CONFLICT (id) DO NOTHING
                    "#,
                )
                .bind(initial_id)
                .bind(&req.name)
                .bind(req.branch_name.as_deref().unwrap_or("Main Campus"))
                .bind(req.logo_url.as_deref().unwrap_or(""))
                .execute(pool)
                .await?;

                sqlx::query_as::<_, Organization>(
                    r#"
                    SELECT id, name, branch_name, logo_url, primary_color, secondary_color, background_color, button_color, created_at, updated_at
                    FROM organizations
                    WHERE id = $1
                    "#,
                )
                .bind(initial_id)
                .fetch_one(pool)
                .await?
            }
        }
    };

    let primary = req.primary_color.unwrap_or(target.primary_color);
    let secondary = req.secondary_color.unwrap_or(target.secondary_color);
    let bg = req.background_color.unwrap_or(target.background_color);
    let btn = req.button_color.unwrap_or(target.button_color);
    let branch = req.branch_name.or(target.branch_name).unwrap_or_else(|| "Main Campus".to_string());
    let logo = req.logo_url;

    sqlx::query(
        r#"
        UPDATE organizations
        SET name = $1, branch_name = $2, logo_url = $3, primary_color = $4, secondary_color = $5, background_color = $6, button_color = $7, updated_at = CURRENT_TIMESTAMP
        WHERE id = $8
        "#,
    )
    .bind(&req.name)
    .bind(&branch)
    .bind(&logo)
    .bind(&primary)
    .bind(&secondary)
    .bind(&bg)
    .bind(&btn)
    .bind(&target.id)
    .execute(pool)
    .await?;

    let updated = sqlx::query_as::<_, Organization>(
        r#"
        SELECT id, name, branch_name, logo_url, primary_color, secondary_color, background_color, button_color, created_at, updated_at
        FROM organizations
        WHERE id = $1
        "#,
    )
    .bind(&target.id)
    .fetch_one(pool)
    .await?;

    Ok(updated)
}
