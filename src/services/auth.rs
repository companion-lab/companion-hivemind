use anyhow::Result;
use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation, encode, decode};
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;
use sqlx::{PgPool, Row};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Settings;
use crate::api::{Claims, RegisterAdminRequest, RegisterMemberRequest, SignInRequest, AuthSession};

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

pub struct AuthService;

impl AuthService {
    pub fn hash_password(password: &str) -> Result<String> {
        Ok(hash(password, DEFAULT_COST)?)
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        Ok(verify(password, hash)?)
    }

    pub fn create_token(secret: &str, user_id: Uuid, company_id: Uuid, role: &str, ttl_seconds: i64) -> Result<String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let claims = Claims {
            user_id,
            company_id,
            role: role.to_string(),
            exp: now + ttl_seconds,
        };
        Ok(encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))?)
    }

    pub fn validate_token(secret: &str, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    pub async fn register_admin(
        db: &PgPool,
        settings: &Settings,
        req: RegisterAdminRequest,
    ) -> Result<AuthSession> {
        if req.company_name.trim().is_empty() {
            anyhow::bail!("Company name is required");
        }
        if !req.email.contains('@') {
            anyhow::bail!("Valid email is required");
        }
        if req.password.len() < 8 {
            anyhow::bail!("Password must be at least 8 characters");
        }

        let existing = sqlx::query("SELECT id FROM users WHERE email = $1")
            .bind(&req.email)
            .fetch_optional(db).await?;
        if existing.is_some() {
            anyhow::bail!("Email already registered");
        }

        let now = now_ms();
        let company_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let slug = slugify(&req.company_name);
        let password_hash = Self::hash_password(&req.password)?;

        sqlx::query(
            "INSERT INTO companies (id, name, slug, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
        ).bind(company_id).bind(&req.company_name).bind(&slug).bind(now).bind(now)
        .execute(db).await?;

        sqlx::query(
            "INSERT INTO users (id, email, name, password_hash, created_at) VALUES ($1, $2, $3, $4, $5)",
        ).bind(user_id).bind(&req.email).bind(&req.name).bind(password_hash).bind(now)
        .execute(db).await?;

        sqlx::query(
            "INSERT INTO company_members (id, company_id, user_id, role, joined_at) VALUES ($1, $2, $3, $4, $5)",
        ).bind(Uuid::new_v4()).bind(company_id).bind(user_id).bind("admin").bind(now)
        .execute(db).await?;

        sqlx::query(
            "INSERT INTO company_config (company_id, allowed_models, default_provider, default_model, hivemind_enabled, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
        ).bind(company_id)
         .bind(serde_json::json!(["claude-sonnet-4-5", "gpt-4o", "gpt-4o-mini"]))
         .bind("anthropic").bind("claude-sonnet-4-5").bind(true).bind(now)
        .execute(db).await?;

        let token = Self::create_token(&settings.jwt_secret, user_id, company_id, "admin", settings.jwt_ttl_seconds)?;
        sqlx::query(
            "INSERT INTO auth_tokens (token, user_id, company_id, created_at, expires_at) VALUES ($1, $2, $3, $4, $5)",
        ).bind(&token).bind(user_id).bind(company_id).bind(now).bind(now + (settings.jwt_ttl_seconds * 1000))
        .execute(db).await?;

        Ok(AuthSession {
            user_id, email: req.email, name: req.name,
            company_id, company_name: req.company_name, company_slug: slug,
            role: "admin".into(), token,
        })
    }

    pub async fn register_member(
        db: &PgPool,
        settings: &Settings,
        req: RegisterMemberRequest,
    ) -> Result<AuthSession> {
        if !req.email.contains('@') {
            anyhow::bail!("Valid email is required");
        }
        if req.password.len() < 8 {
            anyhow::bail!("Password must be at least 8 characters");
        }

        let company = sqlx::query(
            "SELECT id, name, slug FROM companies WHERE slug = $1",
        ).bind(&req.company_slug)
        .fetch_optional(db).await?
            .ok_or_else(|| anyhow::anyhow!("Company not found"))?;

        let company_id: Uuid = company.get("id");
        let company_name: String = company.get("name");
        let company_slug: String = company.get("slug");

        let invite = sqlx::query(
            "SELECT id, role FROM company_invites WHERE email = $1 AND company_id = $2 AND used_at IS NULL",
        ).bind(&req.email).bind(company_id)
        .fetch_optional(db).await?
            .ok_or_else(|| anyhow::anyhow!("No invitation found"))?;

        let invite_role: String = invite.get("role");
        let invite_id: Uuid = invite.get("id");

        let existing = sqlx::query("SELECT id FROM users WHERE email = $1")
            .bind(&req.email)
            .fetch_optional(db).await?;
        if existing.is_some() {
            anyhow::bail!("Email already registered");
        }

        let now = now_ms();
        let user_id = Uuid::new_v4();
        let password_hash = Self::hash_password(&req.password)?;

        sqlx::query(
            "INSERT INTO users (id, email, name, password_hash, created_at) VALUES ($1, $2, $3, $4, $5)",
        ).bind(user_id).bind(&req.email).bind(&req.name).bind(password_hash).bind(now)
        .execute(db).await?;

        sqlx::query(
            "INSERT INTO company_members (id, company_id, user_id, role, joined_at) VALUES ($1, $2, $3, $4, $5)",
        ).bind(Uuid::new_v4()).bind(company_id).bind(user_id).bind(&invite_role).bind(now)
        .execute(db).await?;

        sqlx::query(
            "UPDATE company_invites SET used_at = $1 WHERE id = $2",
        ).bind(now).bind(invite_id)
        .execute(db).await?;

        let token = Self::create_token(&settings.jwt_secret, user_id, company_id, &invite_role, settings.jwt_ttl_seconds)?;
        sqlx::query(
            "INSERT INTO auth_tokens (token, user_id, company_id, created_at, expires_at) VALUES ($1, $2, $3, $4, $5)",
        ).bind(&token).bind(user_id).bind(company_id).bind(now).bind(now + (settings.jwt_ttl_seconds * 1000))
        .execute(db).await?;

        Ok(AuthSession {
            user_id, email: req.email, name: req.name,
            company_id, company_name, company_slug,
            role: invite_role, token,
        })
    }

    pub async fn sign_in(
        db: &PgPool,
        settings: &Settings,
        req: SignInRequest,
    ) -> Result<AuthSession> {
        let user = sqlx::query(
            "SELECT id, email, name, password_hash FROM users WHERE email = $1",
        ).bind(&req.email)
        .fetch_optional(db).await?
            .ok_or_else(|| anyhow::anyhow!("No account found"))?;

        let user_id: Uuid = user.get("id");
        let email: String = user.get("email");
        let name: String = user.get("name");
        let password_hash: String = user.get("password_hash");

        if !Self::verify_password(&req.password, &password_hash)? {
            anyhow::bail!("Incorrect password");
        }

        let membership = sqlx::query(
            "SELECT company_id, role FROM company_members WHERE user_id = $1 LIMIT 1",
        ).bind(user_id)
        .fetch_optional(db).await?
            .ok_or_else(|| anyhow::anyhow!("No company membership found"))?;

        let company_id: Uuid = membership.get("company_id");
        let role: String = membership.get("role");

        let company = sqlx::query(
            "SELECT id, name, slug FROM companies WHERE id = $1",
        ).bind(company_id)
        .fetch_one(db).await?;

        let company_name: String = company.get("name");
        let company_slug: String = company.get("slug");

        let now = now_ms();
        let token = Self::create_token(&settings.jwt_secret, user_id, company_id, &role, settings.jwt_ttl_seconds)?;

        sqlx::query(
            "INSERT INTO auth_tokens (token, user_id, company_id, created_at, expires_at) VALUES ($1, $2, $3, $4, $5)",
        ).bind(&token).bind(user_id).bind(company_id).bind(now).bind(now + (settings.jwt_ttl_seconds * 1000))
        .execute(db).await?;

        Ok(AuthSession {
            user_id, email, name,
            company_id, company_name, company_slug,
            role, token,
        })
    }
}

fn slugify(name: &str) -> String {
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
