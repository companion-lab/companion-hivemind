use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::types::MeetingOut;

pub struct MeetingRepo {
    db: PgPool,
}

impl MeetingRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        company_id: Uuid,
        req: crate::types::MeetingIngestRequest,
        now: i64,
    ) -> Result<MeetingOut, AppError> {
        let id = Uuid::new_v4();
        let participants_json = serde_json::to_value(&req.participants)
            .map_err(|e| AppError::Internal(e.into()))?;
        let vexa_meeting_id = req.vexa_meeting_id;
        let vexa_platform = req.vexa_platform.clone();
        let vexa_native_meeting_id = req.vexa_native_meeting_id.clone();

        sqlx::query(
            r#"
            INSERT INTO meetings (id, company_id, title, date, duration_seconds, participants, summary, created_at,
                                  vexa_meeting_id, vexa_platform, vexa_native_meeting_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&req.title)
        .bind(req.date)
        .bind(req.duration_seconds)
        .bind(&participants_json)
        .bind(Option::<String>::None)
        .bind(now)
        .bind(vexa_meeting_id)
        .bind(&vexa_platform)
        .bind(&vexa_native_meeting_id)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(MeetingOut {
            id,
            company_id,
            title: req.title,
            date: req.date,
            duration_seconds: req.duration_seconds,
            participants: participants_json,
            summary: None,
            created_at: now,
            vexa_meeting_id,
            vexa_platform,
            vexa_native_meeting_id,
        })
    }

    pub async fn list(&self, company_id: Uuid) -> Result<Vec<MeetingOut>, AppError> {
        let rows = sqlx::query(
            r#"SELECT id, company_id, title, date, duration_seconds, participants, summary, created_at,
                      vexa_meeting_id, vexa_platform, vexa_native_meeting_id
               FROM meetings WHERE company_id = $1 ORDER BY date DESC"#,
        )
        .bind(company_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(row_to_meeting_out).collect())
    }
}

pub fn row_to_meeting_out(row: sqlx::postgres::PgRow) -> MeetingOut {
    use sqlx::Row;
    MeetingOut {
        id: row.get("id"),
        company_id: row.get("company_id"),
        title: row.get("title"),
        date: row.get("date"),
        duration_seconds: row.get("duration_seconds"),
        participants: row.get("participants"),
        summary: row.get("summary"),
        created_at: row.get("created_at"),
        vexa_meeting_id: row.get("vexa_meeting_id"),
        vexa_platform: row.get("vexa_platform"),
        vexa_native_meeting_id: row.get("vexa_native_meeting_id"),
    }
}
