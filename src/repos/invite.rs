use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::errors::AppError;
use crate::types::{InviteCreate, InviteOut};

pub struct InviteRepo {
    db: PgPool,
}

impl InviteRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(&self, company_id: Uuid, req: InviteCreate, now: i64) -> Result<InviteOut, AppError> {
        let id = Uuid::new_v4();
        let email = req.email.clone();
        let role = req.role.clone();
        sqlx::query(
            r#"INSERT INTO company_invites (id, company_id, email, role, created_at) VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(req.email)
        .bind(req.role)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(InviteOut {
            id,
            company_id,
            email,
            role,
            created_at: now,
            used_at: None,
        })
    }

    pub async fn list(&self, company_id: Uuid) -> Result<Vec<InviteOut>, AppError> {
        let rows = sqlx::query(
            r#"SELECT id, company_id, email, role, created_at, used_at
               FROM company_invites WHERE company_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(company_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|r| InviteOut {
                id: r.get("id"),
                company_id: r.get("company_id"),
                email: r.get("email"),
                role: r.get("role"),
                created_at: r.get("created_at"),
                used_at: r.get("used_at"),
            })
            .collect())
    }

    pub async fn remove(&self, invite_id: Uuid, company_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"DELETE FROM company_invites WHERE id = $1 AND company_id = $2"#,
        )
        .bind(invite_id)
        .bind(company_id)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }
}
