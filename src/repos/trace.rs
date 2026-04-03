use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::types::{TraceStepCreateRequest, TraceStepPatchRequest, TraceStepOut};

pub struct TraceRepo {
    db: PgPool,
}

impl TraceRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list_by_session(&self, session_id: Uuid) -> Result<Vec<TraceStepOut>, AppError> {
        let rows = sqlx::query(
            r#"SELECT id, session_id, type, status, title, content, tool_name, tool_input, tool_output, is_error, timestamp, duration
               FROM trace_steps WHERE session_id = $1 ORDER BY timestamp ASC"#,
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(trace_step_from_row).collect())
    }

    pub async fn upsert(
        &self,
        session_id: Uuid,
        req: TraceStepCreateRequest,
    ) -> Result<TraceStepOut, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO trace_steps (id, session_id, type, status, title, content, tool_name, tool_input, tool_output, is_error, timestamp, duration)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id) DO UPDATE SET
                type = EXCLUDED.type,
                status = EXCLUDED.status,
                title = EXCLUDED.title,
                content = EXCLUDED.content,
                tool_name = EXCLUDED.tool_name,
                tool_input = EXCLUDED.tool_input,
                tool_output = EXCLUDED.tool_output,
                is_error = EXCLUDED.is_error,
                timestamp = EXCLUDED.timestamp,
                duration = EXCLUDED.duration
            RETURNING id, session_id, type, status, title, content, tool_name, tool_input, tool_output, is_error, timestamp, duration
            "#,
        )
        .bind(req.id)
        .bind(session_id)
        .bind(&req.r#type)
        .bind(&req.status)
        .bind(&req.title)
        .bind(&req.content)
        .bind(&req.tool_name)
        .bind(&req.tool_input)
        .bind(&req.tool_output)
        .bind(req.is_error)
        .bind(req.timestamp)
        .bind(req.duration)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(trace_step_from_row(row))
    }

    pub async fn update(
        &self,
        trace_id: Uuid,
        session_id: Uuid,
        req: TraceStepPatchRequest,
    ) -> Result<Option<TraceStepOut>, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE trace_steps
            SET type = COALESCE($3, type),
                status = COALESCE($4, status),
                title = COALESCE($5, title),
                content = COALESCE($6, content),
                tool_name = COALESCE($7, tool_name),
                tool_input = COALESCE($8, tool_input),
                tool_output = COALESCE($9, tool_output),
                is_error = COALESCE($10, is_error),
                timestamp = COALESCE($11, timestamp),
                duration = COALESCE($12, duration)
            WHERE id = $1 AND session_id = $2
            RETURNING id, session_id, type, status, title, content, tool_name, tool_input, tool_output, is_error, timestamp, duration
            "#,
        )
        .bind(trace_id)
        .bind(session_id)
        .bind(req.r#type.as_deref())
        .bind(req.status.as_deref())
        .bind(req.title.as_deref())
        .bind(req.content.as_deref())
        .bind(req.tool_name.as_deref())
        .bind(req.tool_input.as_ref())
        .bind(req.tool_output.as_deref())
        .bind(req.is_error)
        .bind(req.timestamp)
        .bind(req.duration)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(row.map(trace_step_from_row))
    }
}

pub fn trace_step_from_row(row: sqlx::postgres::PgRow) -> TraceStepOut {
    use sqlx::Row;
    TraceStepOut {
        id: row.get("id"),
        session_id: row.get("session_id"),
        r#type: row.get("type"),
        status: row.get("status"),
        title: row.get("title"),
        content: row.get("content"),
        tool_name: row.get("tool_name"),
        tool_input: row.get("tool_input"),
        tool_output: row.get("tool_output"),
        is_error: row.get("is_error"),
        timestamp: row.get("timestamp"),
        duration: row.get("duration"),
    }
}
