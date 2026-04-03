use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::errors::AppError;
use crate::types::{ApiKeyOut, MemberApiKeySet};

pub struct ApiKeyRepo {
    db: PgPool,
}

impl ApiKeyRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn upsert(
        &self,
        company_id: Uuid,
        req: MemberApiKeySet,
        encrypted_key: String,
        now: i64,
    ) -> Result<ApiKeyOut, AppError> {
        let id = Uuid::new_v4();
        let provider = req.provider.clone();
        let ollama_url = req.ollama_url.clone();

        sqlx::query(
            r#"
            INSERT INTO member_api_keys (id, company_id, user_id, provider, key_encrypted, ollama_url, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (company_id, user_id, provider) DO UPDATE SET
                key_encrypted = EXCLUDED.key_encrypted,
                ollama_url = EXCLUDED.ollama_url,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(id)
        .bind(company_id)
        .bind(req.user_id)
        .bind(&req.provider)
        .bind(encrypted_key)
        .bind(req.ollama_url)
        .bind(now)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(ApiKeyOut {
            id,
            company_id,
            user_id: req.user_id,
            provider,
            key_masked: mask_key(&req.plain_key),
            ollama_url,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn list(&self, user_id: Uuid, company_id: Uuid) -> Result<Vec<ApiKeyOut>, AppError> {
        let rows = sqlx::query(
            r#"SELECT id, company_id, user_id, provider, ollama_url, created_at, updated_at
               FROM member_api_keys WHERE user_id = $1 AND company_id = $2"#,
        )
        .bind(user_id)
        .bind(company_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|r| ApiKeyOut {
                id: r.get("id"),
                company_id: r.get("company_id"),
                user_id: r.get("user_id"),
                provider: r.get("provider"),
                key_masked: "••••••••".to_string(),
                ollama_url: r.get("ollama_url"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    pub async fn delete(&self, key_id: Uuid, company_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"DELETE FROM member_api_keys WHERE id = $1 AND company_id = $2"#,
        )
        .bind(key_id)
        .bind(company_id)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }
}

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        "••••••••".into()
    } else {
        format!("{}••••••••{}", &key[..4], &key[key.len() - 4..])
    }
}
