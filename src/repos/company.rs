use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::errors::AppError;
use crate::types::{CompanyConfigOut, CompanyConfigPatch};

pub struct CompanyRepo {
    db: PgPool,
}

impl CompanyRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_config(&self, company_id: Uuid) -> Result<Option<CompanyConfigOut>, AppError> {
        let row = sqlx::query(
            r#"SELECT company_id, allowed_models, default_provider, default_model, hivemind_enabled, updated_at
               FROM company_config WHERE company_id = $1"#,
        )
        .bind(company_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::from)?;

        match row {
            Some(row) => {
                let allowed_models = parse_string_array(&row.get("allowed_models"));
                Ok(Some(CompanyConfigOut {
                    company_id: row.get("company_id"),
                    allowed_models,
                    default_provider: row.get("default_provider"),
                    default_model: row.get("default_model"),
                    hivemind_enabled: row.get("hivemind_enabled"),
                    updated_at: row.get("updated_at"),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn update_config(
        &self,
        company_id: Uuid,
        patch: CompanyConfigPatch,
        now: i64,
    ) -> Result<CompanyConfigOut, AppError> {
        let allowed_models_json = patch.allowed_models.as_ref().map(|v| serde_json::to_value(v).unwrap());
        sqlx::query(
            r#"
            INSERT INTO company_config (company_id, allowed_models, default_provider, default_model, hivemind_enabled, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (company_id) DO UPDATE SET
                allowed_models = COALESCE(EXCLUDED.allowed_models, company_config.allowed_models),
                default_provider = COALESCE(EXCLUDED.default_provider, company_config.default_provider),
                default_model = COALESCE(EXCLUDED.default_model, company_config.default_model),
                hivemind_enabled = COALESCE(EXCLUDED.hivemind_enabled, company_config.hivemind_enabled),
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(company_id)
        .bind(allowed_models_json)
        .bind(patch.default_provider)
        .bind(patch.default_model)
        .bind(patch.hivemind_enabled)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;

        self.get_config(company_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Company config not found".into()))
    }
}

fn parse_string_array(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}
