use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::errors::AppError;
use crate::types::MemberOut;

pub struct MemberRepo {
    db: PgPool,
}

impl MemberRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list(&self, company_id: Uuid) -> Result<Vec<MemberOut>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT cm.id, cm.company_id, cm.user_id, cm.role, cm.joined_at,
                   u.email, u.name
            FROM company_members cm
            JOIN users u ON u.id = cm.user_id
            WHERE cm.company_id = $1
            ORDER BY cm.joined_at DESC
            "#,
        )
        .bind(company_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|r| MemberOut {
                id: r.get("id"),
                company_id: r.get("company_id"),
                user_id: r.get("user_id"),
                role: r.get("role"),
                email: r.get("email"),
                name: r.get("name"),
                joined_at: r.get("joined_at"),
            })
            .collect())
    }

    pub async fn remove(&self, user_id: Uuid, company_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"DELETE FROM company_members WHERE user_id = $1 AND company_id = $2"#,
        )
        .bind(user_id)
        .bind(company_id)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn update_role(&self, user_id: Uuid, company_id: Uuid, role: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"UPDATE company_members SET role = $1 WHERE user_id = $2 AND company_id = $3"#,
        )
        .bind(role)
        .bind(user_id)
        .bind(company_id)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }
}
