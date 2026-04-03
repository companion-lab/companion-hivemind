use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::errors::AppError;
use crate::types::{UsageRecord, UsageSummary};

pub struct UsageRepo {
    db: PgPool,
}

impl UsageRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn record(
        &self,
        company_id: Uuid,
        req: UsageRecord,
        now: i64,
    ) -> Result<(), AppError> {
        let id = Uuid::new_v4();
        let cost_cents = estimate_cost_cents(&req.model, req.input_tokens, req.output_tokens);

        sqlx::query(
            r#"INSERT INTO token_usage (id, company_id, user_id, session_id, model, provider, input_tokens, output_tokens, cost_cents, recorded_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(req.user_id)
        .bind(req.session_id)
        .bind(req.model)
        .bind(req.provider)
        .bind(req.input_tokens)
        .bind(req.output_tokens)
        .bind(cost_cents)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }

    pub async fn get_summary(&self, company_id: Uuid) -> Result<Vec<UsageSummary>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT tu.user_id,
                   SUM(tu.input_tokens) as total_input_tokens,
                   SUM(tu.output_tokens) as total_output_tokens,
                   SUM(tu.cost_cents) as total_cost_cents,
                   COUNT(DISTINCT tu.session_id) as session_count,
                   MAX(tu.recorded_at) as last_active_at
            FROM token_usage tu
            WHERE tu.company_id = $1
            GROUP BY tu.user_id
            "#,
        )
        .bind(company_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::from)?;

        let user_ids: Vec<Uuid> = rows.iter().map(|r| r.get("user_id")).collect();
        let users = if user_ids.is_empty() {
            Vec::new()
        } else {
            sqlx::query(
                r#"SELECT id, email, name FROM users WHERE id = ANY($1)"#,
            )
            .bind(&user_ids)
            .fetch_all(&self.db)
            .await
            .unwrap_or_default()
        };

        let summaries = rows
            .into_iter()
            .map(|r| {
                let user_id: Uuid = r.get("user_id");
                let user = users.iter().find(|u| {
                    let uid: Uuid = u.get("id");
                    uid == user_id
                });
                UsageSummary {
                    user_id,
                    email: user.as_ref().map(|u| u.get("email")).unwrap_or_default(),
                    name: user.as_ref().map(|u| u.get("name")).unwrap_or_default(),
                    total_input_tokens: r.get::<Option<i64>, _>("total_input_tokens").unwrap_or(0),
                    total_output_tokens: r.get::<Option<i64>, _>("total_output_tokens").unwrap_or(0),
                    total_cost_cents: r.get::<Option<i64>, _>("total_cost_cents").unwrap_or(0),
                    session_count: r.get::<Option<i64>, _>("session_count").unwrap_or(0),
                    last_active_at: r.get::<Option<i64>, _>("last_active_at"),
                }
            })
            .collect();

        Ok(summaries)
    }
}

fn estimate_cost_cents(model: &str, input_tokens: i64, output_tokens: i64) -> i32 {
    let pricing: &[(&str, f64, f64)] = &[
        ("claude-sonnet-4-5", 0.3, 1.5),
        ("claude-opus-4", 1.5, 7.5),
        ("claude-haiku-3", 0.025, 0.125),
        ("gpt-4o", 0.5, 1.5),
        ("gpt-4o-mini", 0.015, 0.06),
        ("gpt-4-turbo", 1.0, 3.0),
    ];

    let (_, in_rate, out_rate) = pricing
        .iter()
        .find(|(m, _, _)| model.contains(m))
        .copied()
        .unwrap_or(("unknown", 0.1, 0.3));

    ((input_tokens as f64 / 1000.0) * in_rate + (output_tokens as f64 / 1000.0) * out_rate).round() as i32
}
