use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::types::{AuthSession, Claims, RegisterAdminRequest, RegisterMemberRequest, SignInRequest};

pub struct AuthRepo {
    db: PgPool,
}

impl AuthRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<UserRecord>, AppError> {
        sqlx::query_as::<_, UserRecord>(
            r#"SELECT id, email, name, password_hash FROM users WHERE email = $1"#,
        )
        .bind(email)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::from)
    }

    pub async fn find_company_by_slug(&self, slug: &str) -> Result<Option<CompanyRecord>, AppError> {
        sqlx::query_as::<_, CompanyRecord>(
            r#"SELECT id, name, slug FROM companies WHERE slug = $1"#,
        )
        .bind(slug)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::from)
    }

    pub async fn find_company_by_id(&self, id: Uuid) -> Result<CompanyRecord, AppError> {
        sqlx::query_as::<_, CompanyRecord>(
            r#"SELECT id, name, slug FROM companies WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::from)
    }

    pub async fn find_invite(&self, email: &str, company_id: Uuid) -> Result<Option<InviteRecord>, AppError> {
        sqlx::query_as::<_, InviteRecord>(
            r#"SELECT id, role FROM company_invites WHERE email = $1 AND company_id = $2 AND used_at IS NULL"#,
        )
        .bind(email)
        .bind(company_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::from)
    }

    pub async fn find_membership(&self, user_id: Uuid) -> Result<Option<MembershipRecord>, AppError> {
        sqlx::query_as::<_, MembershipRecord>(
            r#"SELECT company_id, role FROM company_members WHERE user_id = $1 LIMIT 1"#,
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::from)
    }

    pub async fn find_user_context(
        &self,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<UserContextRecord>, AppError> {
        sqlx::query_as::<_, UserContextRecord>(
            r#"
            SELECT u.email, u.name, c.name AS company_name, c.slug AS company_slug, cm.role
            FROM users u
            JOIN companies c ON c.id = $1
            JOIN company_members cm ON cm.user_id = $2 AND cm.company_id = $1
            WHERE u.id = $2
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::from)
    }

    pub async fn create_company(&self, id: Uuid, name: &str, slug: &str, now: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"INSERT INTO companies (id, name, slug, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(now)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn create_user(&self, id: Uuid, email: &str, name: &str, password_hash: &str, now: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"INSERT INTO users (id, email, name, password_hash, created_at) VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(email)
        .bind(name)
        .bind(password_hash)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn create_membership(&self, id: Uuid, company_id: Uuid, user_id: Uuid, role: &str, now: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"INSERT INTO company_members (id, company_id, user_id, role, joined_at) VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(user_id)
        .bind(role)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn create_default_config(&self, company_id: Uuid, now: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO company_config (company_id, allowed_models, default_provider, default_model, hivemind_enabled, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(company_id)
        .bind(serde_json::json!(["claude-sonnet-4-5", "gpt-4o", "gpt-4o-mini"]))
        .bind("anthropic")
        .bind("claude-sonnet-4-5")
        .bind(true)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn create_auth_token(&self, token: &str, user_id: Uuid, company_id: Uuid, now: i64, expires_at: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"INSERT INTO auth_tokens (token, user_id, company_id, created_at, expires_at) VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(token)
        .bind(user_id)
        .bind(company_id)
        .bind(now)
        .bind(expires_at)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn delete_auth_tokens(&self, user_id: Uuid, company_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"DELETE FROM auth_tokens WHERE user_id = $1 AND company_id = $2"#,
        )
        .bind(user_id)
        .bind(company_id)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn mark_invite_used(&self, invite_id: Uuid, now: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"UPDATE company_invites SET used_at = $1 WHERE id = $2"#,
        )
        .bind(now)
        .bind(invite_id)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CompanyRecord {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct InviteRecord {
    pub id: Uuid,
    pub role: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct MembershipRecord {
    pub company_id: Uuid,
    pub role: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserContextRecord {
    pub email: String,
    pub name: String,
    pub company_name: String,
    pub company_slug: String,
    pub role: String,
}

// ─── JWT helpers ─────────────────────────────────────────────────────────────

pub fn create_token(
    secret: &str,
    user_id: Uuid,
    company_id: Uuid,
    role: &str,
    ttl_seconds: i64,
) -> Result<String, AppError> {
    use jsonwebtoken::{EncodingKey, Header, encode};
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(e.into()))?
        .as_secs() as i64;

    let claims = Claims {
        user_id,
        company_id,
        role: role.to_string(),
        exp: now + ttl_seconds,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(e.into()))
}

pub fn validate_token(secret: &str, token: &str) -> Result<Claims, AppError> {
    use jsonwebtoken::{DecodingKey, Validation, decode};

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

    Ok(token_data.claims)
}

// ─── Password helpers ────────────────────────────────────────────────────────

pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.into()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    bcrypt::verify(password, hash)
        .map_err(|e| AppError::Internal(e.into()))
}

// ─── Slugify ─────────────────────────────────────────────────────────────────

pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(64)
        .collect()
}

// ─── AuthSession builder ────────────────────────────────────────────────────

impl AuthSession {
    pub fn new(
        user_id: Uuid,
        email: String,
        name: String,
        company_id: Uuid,
        company_name: String,
        company_slug: String,
        role: String,
        token: String,
    ) -> Self {
        Self {
            user_id,
            email,
            name,
            company_id,
            company_name,
            company_slug,
            role,
            token,
        }
    }
}
