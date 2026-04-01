use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct CompanyMember {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct CompanyInvite {
    pub id: Uuid,
    pub company_id: Uuid,
    pub email: String,
    pub role: String,
    pub created_at: i64,
    pub used_at: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct MemberApiKey {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub key_encrypted: String,
    pub ollama_url: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct CompanyConfig {
    pub company_id: Uuid,
    pub allowed_models: serde_json::Value,
    pub default_provider: String,
    pub default_model: String,
    pub hivemind_enabled: bool,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct Meeting {
    pub id: Uuid,
    pub company_id: Uuid,
    pub title: String,
    pub date: i64,
    pub duration_seconds: i32,
    pub participants: serde_json::Value,
    pub summary: Option<String>,
    pub created_at: i64,
    pub vexa_meeting_id: Option<i32>,
    pub vexa_platform: Option<String>,
    pub vexa_native_meeting_id: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct KnowledgeChunk {
    pub id: Uuid,
    pub meeting_id: Uuid,
    pub segment_id: Option<Uuid>,
    pub text: String,
    pub speaker: Option<String>,
    pub timestamp: Option<i64>,
    pub chunk_type: String,
    pub embedding: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct TokenUsage {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub session_id: String,
    pub model: String,
    pub provider: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_cents: i32,
    pub recorded_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct AuthToken {
    pub token: String,
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub created_at: i64,
    pub expires_at: i64,
}
