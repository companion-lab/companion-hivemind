use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::types::{SessionCreateRequest, SessionPatchRequest, SessionOut};

pub struct SessionRepo {
    db: PgPool,
}

impl SessionRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        company_id: Uuid,
        user_id: Uuid,
        req: SessionCreateRequest,
        now: i64,
    ) -> Result<SessionOut, AppError> {
        let id = Uuid::new_v4();
        let title = req.title.unwrap_or_else(|| "New conversation".into());
        let mode = req.mode.unwrap_or_else(|| "research".into());

        sqlx::query(
            r#"
            INSERT INTO sessions (id, company_id, user_id, title, status, cwd, model, mode, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 'idle', $5, $6, $7, $8, $8)
            "#,
        )
        .bind(id)
        .bind(company_id)
        .bind(user_id)
        .bind(&title)
        .bind(req.cwd.as_deref())
        .bind(&req.model)
        .bind(&mode)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(SessionOut {
            id,
            company_id,
            user_id,
            title,
            status: "idle".into(),
            cwd: req.cwd,
            model: req.model,
            mode,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn list(&self, company_id: Uuid, user_id: Uuid) -> Result<Vec<SessionOut>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, company_id, user_id, title, status, cwd, model, mode, created_at, updated_at
            FROM sessions
            WHERE company_id = $1 AND user_id = $2
            ORDER BY updated_at DESC
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(session_from_row).collect())
    }

    pub async fn get_by_id(
        &self,
        session_id: Uuid,
        company_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<SessionOut>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, company_id, user_id, title, status, cwd, model, mode, created_at, updated_at
            FROM sessions
            WHERE id = $1 AND company_id = $2 AND user_id = $3
            "#,
        )
        .bind(session_id)
        .bind(company_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(row.map(session_from_row))
    }

    pub async fn update(
        &self,
        session_id: Uuid,
        company_id: Uuid,
        user_id: Uuid,
        req: SessionPatchRequest,
        now: i64,
    ) -> Result<Option<SessionOut>, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE sessions
            SET title = COALESCE($4, title),
                status = COALESCE($5, status),
                model = COALESCE($6, model),
                mode = COALESCE($7, mode),
                updated_at = $8
            WHERE id = $1 AND company_id = $2 AND user_id = $3
            RETURNING id, company_id, user_id, title, status, cwd, model, mode, created_at, updated_at
            "#,
        )
        .bind(session_id)
        .bind(company_id)
        .bind(user_id)
        .bind(req.title.as_deref())
        .bind(req.status.as_deref())
        .bind(req.model.as_deref())
        .bind(req.mode.as_deref())
        .bind(now)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(row.map(session_from_row))
    }

    pub async fn delete(
        &self,
        session_id: Uuid,
        company_id: Uuid,
        user_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"DELETE FROM sessions WHERE id = $1 AND company_id = $2 AND user_id = $3"#,
        )
        .bind(session_id)
        .bind(company_id)
        .bind(user_id)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(result.rows_affected())
    }

    pub async fn is_accessible(
        &self,
        session_id: Uuid,
        company_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, AppError> {
        let row = sqlx::query(
            r#"SELECT 1 FROM sessions WHERE id = $1 AND company_id = $2 AND user_id = $3"#,
        )
        .bind(session_id)
        .bind(company_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(row.is_some())
    }
}

pub fn session_from_row(row: sqlx::postgres::PgRow) -> SessionOut {
    use sqlx::Row;
    SessionOut {
        id: row.get("id"),
        company_id: row.get("company_id"),
        user_id: row.get("user_id"),
        title: row.get("title"),
        status: row.get("status"),
        cwd: row.get("cwd"),
        model: row.get("model"),
        mode: row.get("mode"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
