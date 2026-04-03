use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Auth ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub role: String,
    pub exp: i64,
}

#[derive(Debug, Deserialize)]
pub struct RegisterAdminRequest {
    pub company_name: String,
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterMemberRequest {
    pub email: String,
    pub name: String,
    pub password: String,
    pub company_slug: String,
}

#[derive(Debug, Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthSession {
    pub user_id: Uuid,
    pub email: String,
    pub name: String,
    pub company_id: Uuid,
    pub company_name: String,
    pub company_slug: String,
    pub role: String,
    pub token: String,
}

// ─── Company Config ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CompanyConfigOut {
    pub company_id: Uuid,
    pub allowed_models: Vec<String>,
    pub default_provider: String,
    pub default_model: String,
    pub hivemind_enabled: bool,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CompanyConfigPatch {
    pub allowed_models: Option<Vec<String>>,
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
    pub hivemind_enabled: Option<bool>,
}

// ─── Members ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MemberOut {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub email: String,
    pub name: String,
    pub joined_at: i64,
}

// ─── Invites ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct InviteOut {
    pub id: Uuid,
    pub company_id: Uuid,
    pub email: String,
    pub role: String,
    pub created_at: i64,
    pub used_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct InviteCreate {
    pub email: String,
    #[serde(default = "default_member_role")]
    pub role: String,
}

fn default_member_role() -> String {
    "member".into()
}

// ─── API Keys ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ApiKeyOut {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub key_masked: String,
    pub ollama_url: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct MemberApiKeySet {
    pub user_id: Uuid,
    pub provider: String,
    pub plain_key: String,
    pub ollama_url: Option<String>,
}

// ─── Meetings ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct TranscriptSegment {
    pub speaker: String,
    pub text: String,
    #[serde(default)]
    pub start_time: f64,
    #[serde(default)]
    pub end_time: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MeetingIngestRequest {
    pub title: String,
    pub date: i64,
    #[serde(default)]
    pub duration_seconds: i32,
    #[serde(default)]
    pub participants: Vec<String>,
    pub transcript: Vec<TranscriptSegment>,
    pub vexa_meeting_id: Option<i32>,
    pub vexa_platform: Option<String>,
    pub vexa_native_meeting_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MeetingOut {
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

// ─── Knowledge ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct KnowledgeSearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    6
}

#[derive(Debug, Serialize)]
pub struct KnowledgeSearchResult {
    pub chunk: serde_json::Value,
    pub meeting: serde_json::Value,
    pub score: f64,
}

// ─── Usage ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UsageRecord {
    pub user_id: Option<Uuid>,
    pub session_id: String,
    pub model: String,
    pub provider: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[derive(Debug, Serialize)]
pub struct UsageSummary {
    pub user_id: Uuid,
    pub email: String,
    pub name: String,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_cents: i64,
    pub session_count: i64,
    pub last_active_at: Option<i64>,
}

// ─── Sessions ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SessionCreateRequest {
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SessionPatchRequest {
    pub title: Option<String>,
    pub status: Option<String>,
    pub model: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SessionOut {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub status: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub mode: String,
    pub created_at: i64,
    pub updated_at: i64,
}

// ─── Messages ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct MessageCreateRequest {
    pub id: Uuid,
    pub role: String,
    pub content: serde_json::Value,
    pub timestamp: i64,
    pub token_usage: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct MessageOut {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: String,
    pub content: serde_json::Value,
    pub timestamp: i64,
    pub token_usage: Option<serde_json::Value>,
}

// ─── Trace Steps ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TraceStepCreateRequest {
    pub id: Uuid,
    pub r#type: String,
    pub status: String,
    pub title: String,
    pub content: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_output: Option<String>,
    pub is_error: Option<bool>,
    pub timestamp: i64,
    pub duration: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct TraceStepPatchRequest {
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_output: Option<String>,
    pub is_error: Option<bool>,
    pub timestamp: Option<i64>,
    pub duration: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TraceStepOut {
    pub id: Uuid,
    pub session_id: Uuid,
    pub r#type: String,
    pub status: String,
    pub title: String,
    pub content: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_output: Option<String>,
    pub is_error: Option<bool>,
    pub timestamp: i64,
    pub duration: Option<i64>,
}
