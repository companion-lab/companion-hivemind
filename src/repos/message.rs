use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::types::{MessageCreateRequest, MessageOut};

pub struct MessageRepo {
    db: PgPool,
}

impl MessageRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list_by_session(&self, session_id: Uuid) -> Result<Vec<MessageOut>, AppError> {
        let rows = sqlx::query(
            r#"SELECT id, session_id, role, content, timestamp, token_usage
               FROM messages WHERE session_id = $1 ORDER BY timestamp ASC"#,
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(message_from_row).collect())
    }

    pub async fn create(
        &self,
        session_id: Uuid,
        req: MessageCreateRequest,
    ) -> Result<MessageOut, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO messages (id, session_id, role, content, timestamp, token_usage)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, session_id, role, content, timestamp, token_usage
            "#,
        )
        .bind(req.id)
        .bind(session_id)
        .bind(req.role)
        .bind(req.content)
        .bind(req.timestamp)
        .bind(req.token_usage)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(message_from_row(row))
    }
}

pub fn message_from_row(row: sqlx::postgres::PgRow) -> MessageOut {
    use sqlx::Row;
    MessageOut {
        id: row.get("id"),
        session_id: row.get("session_id"),
        role: row.get("role"),
        content: row.get("content"),
        timestamp: row.get("timestamp"),
        token_usage: row.get("token_usage"),
    }
}
