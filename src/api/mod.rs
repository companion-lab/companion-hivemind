use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{delete, get, patch, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::AppState;
use crate::services::{auth::AuthService, crypto::CryptoService, knowledge::KnowledgeService};

// ─── Shared response types ───────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self { ok: true, data: Some(data), error: None }
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self { ok: false, data: None, error: Some(msg.into()) }
    }
}

// ─── Auth extraction middleware ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub role: String,
}

#[axum::async_trait]
impl axum::extract::FromRequestParts<AppState> for AuthContext {
    type Rejection = (StatusCode, Json<ApiResponse<()>>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::error("Missing Authorization header")),
            ))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::error("Invalid Authorization format")),
            ))?;

        let claims = AuthService::validate_token(&state.settings.jwt_secret, token)
            .map_err(|_| (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::error("Invalid or expired token")),
            ))?;

        Ok(AuthContext {
            user_id: claims.user_id,
            company_id: claims.company_id,
            role: claims.role,
        })
    }
}

// ─── JWT Claims ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub role: String,
    pub exp: i64,
}

// ─── Request/Response schemas ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RegisterAdminRequest {
    pub company_name: String,
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterMemberRequest {
    pub email: String,
    pub name: String,
    pub password: String,
    pub company_slug: String,
}

#[derive(Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
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

#[derive(Serialize, Deserialize)]
pub struct CompanyConfigOut {
    pub company_id: Uuid,
    pub allowed_models: Vec<String>,
    pub default_provider: String,
    pub default_model: String,
    pub hivemind_enabled: bool,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct CompanyConfigPatch {
    pub allowed_models: Option<Vec<String>>,
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
    pub hivemind_enabled: Option<bool>,
}

#[derive(Deserialize)]
pub struct MemberApiKeySet {
    pub user_id: Uuid,
    pub provider: String,
    pub plain_key: String,
    pub ollama_url: Option<String>,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct MemberOut {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub email: String,
    pub name: String,
    pub joined_at: i64,
}

#[derive(Serialize)]
pub struct InviteOut {
    pub id: Uuid,
    pub company_id: Uuid,
    pub email: String,
    pub role: String,
    pub created_at: i64,
    pub used_at: Option<i64>,
}

#[derive(Deserialize)]
pub struct InviteCreate {
    pub email: String,
    #[serde(default = "default_member_role")]
    pub role: String,
}

fn default_member_role() -> String { "member".into() }

#[derive(Deserialize)]
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

#[derive(Deserialize, Clone)]
pub struct TranscriptSegment {
    pub speaker: String,
    pub text: String,
    #[serde(default)]
    pub start_time: f64,
    #[serde(default)]
    pub end_time: f64,
}

#[derive(Serialize)]
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

#[derive(Deserialize)]
pub struct KnowledgeSearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize { 6 }

#[derive(Serialize)]
pub struct KnowledgeSearchResult {
    pub chunk: serde_json::Value,
    pub meeting: serde_json::Value,
    pub score: f64,
}

#[derive(Deserialize)]
pub struct UsageRecord {
    pub user_id: Option<Uuid>,
    pub session_id: String,
    pub model: String,
    pub provider: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[derive(Serialize)]
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

#[derive(Deserialize)]
pub struct SessionCreateRequest {
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub mode: Option<String>,
}

#[derive(Deserialize)]
pub struct SessionPatchRequest {
    pub title: Option<String>,
    pub status: Option<String>,
    pub model: Option<String>,
    pub mode: Option<String>,
}

#[derive(Serialize)]
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

#[derive(Deserialize)]
pub struct MessageCreateRequest {
    pub id: Uuid,
    pub role: String,
    pub content: serde_json::Value,
    pub timestamp: i64,
    pub token_usage: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct MessageOut {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: String,
    pub content: serde_json::Value,
    pub timestamp: i64,
    pub token_usage: Option<serde_json::Value>,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
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

#[derive(Serialize)]
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

// ─── Auth routes ─────────────────────────────────────────────────────────────

pub async fn register_admin(
    State(state): State<AppState>,
    Json(req): Json<RegisterAdminRequest>,
) -> Json<ApiResponse<AuthSession>> {
    match AuthService::register_admin(&state.db, &state.settings, req).await {
        Ok(session) => Json(ApiResponse::success(session)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn register_member(
    State(state): State<AppState>,
    Json(req): Json<RegisterMemberRequest>,
) -> Json<ApiResponse<AuthSession>> {
    match AuthService::register_member(&state.db, &state.settings, req).await {
        Ok(session) => Json(ApiResponse::success(session)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn sign_in(
    State(state): State<AppState>,
    Json(req): Json<SignInRequest>,
) -> Json<ApiResponse<AuthSession>> {
    match AuthService::sign_in(&state.db, &state.settings, req).await {
        Ok(session) => Json(ApiResponse::success(session)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn sign_out(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Json<ApiResponse<()>> {
    let _ = sqlx::query(
        "DELETE FROM auth_tokens WHERE user_id = $1 AND company_id = $2",
    )
    .bind(auth.user_id)
    .bind(auth.company_id)
    .execute(&state.db)
    .await;
    Json(ApiResponse::success(()))
}

pub async fn auth_me(
    State(state): State<AppState>,
    auth: AuthContext,
    headers: axum::http::HeaderMap,
) -> Json<ApiResponse<AuthSession>> {
    match sqlx::query(
        r#"
        SELECT u.email, u.name, c.name AS company_name, c.slug AS company_slug, cm.role
        FROM users u
        JOIN companies c ON c.id = $1
        JOIN company_members cm ON cm.user_id = $2 AND cm.company_id = $1
        WHERE u.id = $2
        "#,
    )
    .bind(auth.company_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(row) => {
            let token = headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .unwrap_or_default()
                .to_string();
            Json(ApiResponse::success(AuthSession {
                user_id: auth.user_id,
                email: row.get("email"),
                name: row.get("name"),
                company_id: auth.company_id,
                company_name: row.get("company_name"),
                company_slug: row.get("company_slug"),
                role: row.get("role"),
                token,
            }))
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

// ─── Company routes ──────────────────────────────────────────────────────────

pub async fn get_company_config(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Json<ApiResponse<CompanyConfigOut>> {
    match sqlx::query(
        "SELECT company_id, allowed_models, default_provider, default_model, hivemind_enabled, updated_at FROM company_config WHERE company_id = $1",
    )
    .bind(auth.company_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => {
            let models: serde_json::Value = row.get("allowed_models");
            let allowed_models = models.as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            Json(ApiResponse::success(CompanyConfigOut {
                company_id: row.get("company_id"),
                allowed_models,
                default_provider: row.get("default_provider"),
                default_model: row.get("default_model"),
                hivemind_enabled: row.get("hivemind_enabled"),
                updated_at: row.get("updated_at"),
            }))
        }
        Ok(None) => Json(ApiResponse::error("Company config not found")),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn update_company_config(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(patch): Json<CompanyConfigPatch>,
) -> Json<ApiResponse<CompanyConfigOut>> {
    if auth.role != "admin" {
        return Json(ApiResponse::error("Admin access required"));
    }

    let now = now_ms();

    // Upsert config
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
    .bind(auth.company_id)
    .bind(patch.allowed_models.as_ref().map(|v| serde_json::to_value(v).unwrap()))
    .bind(patch.default_provider)
    .bind(patch.default_model)
    .bind(patch.hivemind_enabled)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    .unwrap();

    // Re-fetch
    match sqlx::query(
        "SELECT company_id, allowed_models, default_provider, default_model, hivemind_enabled, updated_at FROM company_config WHERE company_id = $1",
    )
    .bind(auth.company_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(row) => {
            let models: serde_json::Value = row.get("allowed_models");
            let allowed_models = models.as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            Json(ApiResponse::success(CompanyConfigOut {
                company_id: row.get("company_id"),
                allowed_models,
                default_provider: row.get("default_provider"),
                default_model: row.get("default_model"),
                hivemind_enabled: row.get("hivemind_enabled"),
                updated_at: row.get("updated_at"),
            }))
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

// ─── Members routes ──────────────────────────────────────────────────────────

pub async fn list_members(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Json<ApiResponse<Vec<MemberOut>>> {
    match sqlx::query(
        r#"
        SELECT cm.id, cm.company_id, cm.user_id, cm.role, cm.joined_at,
               u.email, u.name
        FROM company_members cm
        JOIN users u ON u.id = cm.user_id
        WHERE cm.company_id = $1
        ORDER BY cm.joined_at DESC
        "#,
    )
    .bind(auth.company_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => {
            let members = rows.into_iter().map(|r| MemberOut {
                id: r.get("id"),
                company_id: r.get("company_id"),
                user_id: r.get("user_id"),
                role: r.get("role"),
                email: r.get("email"),
                name: r.get("name"),
                joined_at: r.get("joined_at"),
            }).collect();
            Json(ApiResponse::success(members))
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn remove_member(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(user_id): axum::extract::Path<Uuid>,
) -> Json<ApiResponse<()>> {
    if auth.role != "admin" {
        return Json(ApiResponse::error("Admin access required"));
    }
    match sqlx::query(
        "DELETE FROM company_members WHERE user_id = $1 AND company_id = $2",
    )
    .bind(user_id)
    .bind(auth.company_id)
    .execute(&state.db)
    .await
    {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn update_member_role(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path((user_id, role)): axum::extract::Path<(Uuid, String)>,
) -> Json<ApiResponse<()>> {
    if auth.role != "admin" {
        return Json(ApiResponse::error("Admin access required"));
    }
    match sqlx::query(
        "UPDATE company_members SET role = $1 WHERE user_id = $2 AND company_id = $3",
    )
    .bind(role)
    .bind(user_id)
    .bind(auth.company_id)
    .execute(&state.db)
    .await
    {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

// ─── Invites routes ──────────────────────────────────────────────────────────

pub async fn add_invite(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<InviteCreate>,
) -> Json<ApiResponse<InviteOut>> {
    if auth.role != "admin" {
        return Json(ApiResponse::error("Admin access required"));
    }
    let now = now_ms();
    let id = Uuid::new_v4();
    match sqlx::query(
        "INSERT INTO company_invites (id, company_id, email, role, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(auth.company_id)
    .bind(&req.email)
    .bind(&req.role)
    .bind(now)
    .execute(&state.db)
    .await
    {
        Ok(_) => Json(ApiResponse::success(InviteOut {
            id, company_id: auth.company_id, email: req.email, role: req.role, created_at: now, used_at: None,
        })),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn list_invites(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Json<ApiResponse<Vec<InviteOut>>> {
    match sqlx::query(
        "SELECT id, company_id, email, role, created_at, used_at FROM company_invites WHERE company_id = $1 ORDER BY created_at DESC",
    )
    .bind(auth.company_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => {
            let invites = rows.into_iter().map(|r| InviteOut {
                id: r.get("id"),
                company_id: r.get("company_id"),
                email: r.get("email"),
                role: r.get("role"),
                created_at: r.get("created_at"),
                used_at: r.get("used_at"),
            }).collect();
            Json(ApiResponse::success(invites))
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn remove_invite(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(invite_id): axum::extract::Path<Uuid>,
) -> Json<ApiResponse<()>> {
    if auth.role != "admin" {
        return Json(ApiResponse::error("Admin access required"));
    }
    match sqlx::query(
        "DELETE FROM company_invites WHERE id = $1 AND company_id = $2",
    )
    .bind(invite_id)
    .bind(auth.company_id)
    .execute(&state.db)
    .await
    {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

// ─── API Keys routes ─────────────────────────────────────────────────────────

pub async fn set_api_key(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<MemberApiKeySet>,
) -> Json<ApiResponse<ApiKeyOut>> {
    if auth.role != "admin" {
        return Json(ApiResponse::error("Admin access required"));
    }
    let encrypted = CryptoService::encrypt(&req.plain_key, &state.settings.encryption_secret);
    let now = now_ms();
    let id = Uuid::new_v4();

    match sqlx::query(
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
    .bind(auth.company_id)
    .bind(req.user_id)
    .bind(&req.provider)
    .bind(encrypted)
    .bind(&req.ollama_url)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    {
        Ok(_) => Json(ApiResponse::success(ApiKeyOut {
            id, company_id: auth.company_id, user_id: req.user_id,
            provider: req.provider, key_masked: mask_key(&req.plain_key),
            ollama_url: req.ollama_url, created_at: now, updated_at: now,
        })),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn list_api_keys(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(user_id): axum::extract::Path<Uuid>,
) -> Json<ApiResponse<Vec<ApiKeyOut>>> {
    match sqlx::query(
        "SELECT id, company_id, user_id, provider, key_encrypted, ollama_url, created_at, updated_at FROM member_api_keys WHERE user_id = $1 AND company_id = $2",
    )
    .bind(user_id)
    .bind(auth.company_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => {
            let keys = rows.into_iter().map(|r| ApiKeyOut {
                id: r.get("id"),
                company_id: r.get("company_id"),
                user_id: r.get("user_id"),
                provider: r.get("provider"),
                key_masked: "••••••••".to_string(),
                ollama_url: r.get("ollama_url"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }).collect();
            Json(ApiResponse::success(keys))
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn delete_api_key(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(key_id): axum::extract::Path<Uuid>,
) -> Json<ApiResponse<()>> {
    if auth.role != "admin" {
        return Json(ApiResponse::error("Admin access required"));
    }
    match sqlx::query(
        "DELETE FROM member_api_keys WHERE id = $1 AND company_id = $2",
    )
    .bind(key_id)
    .bind(auth.company_id)
    .execute(&state.db)
    .await
    {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

// ─── Meeting / Knowledge routes ──────────────────────────────────────────────

pub async fn ingest_meeting(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<MeetingIngestRequest>,
) -> Json<ApiResponse<MeetingOut>> {
    let now = now_ms();
    let meeting_id = Uuid::new_v4();
    let participants_json = serde_json::to_value(&req.participants).unwrap();

    match sqlx::query(
        r#"
        INSERT INTO meetings (id, company_id, title, date, duration_seconds, participants, summary, created_at,
                              vexa_meeting_id, vexa_platform, vexa_native_meeting_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(meeting_id)
    .bind(auth.company_id)
    .bind(&req.title)
    .bind(req.date)
    .bind(req.duration_seconds)
    .bind(&participants_json)
    .bind(None::<String>)
    .bind(now)
    .bind(req.vexa_meeting_id)
    .bind(&req.vexa_platform)
    .bind(&req.vexa_native_meeting_id)
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            let chunks = KnowledgeService::chunk_transcript(&req.transcript, meeting_id, req.date);
            for chunk in chunks {
                let _ = sqlx::query(
                    "INSERT INTO knowledge_chunks (id, meeting_id, text, speaker, timestamp, chunk_type, metadata, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(chunk.id)
                .bind(chunk.meeting_id)
                .bind(chunk.text)
                .bind(chunk.speaker)
                .bind(chunk.timestamp)
                .bind(chunk.chunk_type)
                .bind(chunk.metadata)
                .bind(chunk.created_at)
                .execute(&state.db)
                .await;
            }

            Json(ApiResponse::success(MeetingOut {
                id: meeting_id, company_id: auth.company_id, title: req.title,
                date: req.date, duration_seconds: req.duration_seconds,
                participants: participants_json, summary: None, created_at: now,
                vexa_meeting_id: req.vexa_meeting_id,
                vexa_platform: req.vexa_platform,
                vexa_native_meeting_id: req.vexa_native_meeting_id,
            }))
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn list_meetings(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Json<ApiResponse<Vec<MeetingOut>>> {
    match sqlx::query(
        "SELECT id, company_id, title, date, duration_seconds, participants, summary, created_at, vexa_meeting_id, vexa_platform, vexa_native_meeting_id FROM meetings WHERE company_id = $1 ORDER BY date DESC",
    )
    .bind(auth.company_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => {
            let meetings = rows.into_iter().map(|r| MeetingOut {
                id: r.get("id"),
                company_id: r.get("company_id"),
                title: r.get("title"),
                date: r.get("date"),
                duration_seconds: r.get("duration_seconds"),
                participants: r.get("participants"),
                summary: r.get("summary"),
                created_at: r.get("created_at"),
                vexa_meeting_id: r.get("vexa_meeting_id"),
                vexa_platform: r.get("vexa_platform"),
                vexa_native_meeting_id: r.get("vexa_native_meeting_id"),
            }).collect();
            Json(ApiResponse::success(meetings))
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn search_knowledge(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<KnowledgeSearchRequest>,
) -> Json<ApiResponse<Vec<KnowledgeSearchResult>>> {
    match KnowledgeService::search(&state.db, auth.company_id, &req.query, req.limit).await {
        Ok(results) => Json(ApiResponse::success(results)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

// ─── Token usage routes ──────────────────────────────────────────────────────

pub async fn record_usage(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<UsageRecord>,
) -> Json<ApiResponse<()>> {
    let now = now_ms();
    let id = Uuid::new_v4();
    let cost_cents = estimate_cost_cents(&req.model, req.input_tokens, req.output_tokens);

    match sqlx::query(
        "INSERT INTO token_usage (id, company_id, user_id, session_id, model, provider, input_tokens, output_tokens, cost_cents, recorded_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(id)
    .bind(auth.company_id)
    .bind(req.user_id)
    .bind(&req.session_id)
    .bind(&req.model)
    .bind(&req.provider)
    .bind(req.input_tokens)
    .bind(req.output_tokens)
    .bind(cost_cents)
    .bind(now)
    .execute(&state.db)
    .await
    {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn get_usage_summary(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Json<ApiResponse<Vec<UsageSummary>>> {
    match sqlx::query(
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
    .bind(auth.company_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => {
            let user_ids: Vec<Uuid> = rows.iter().map(|r| r.get("user_id")).collect();
            let users = sqlx::query(
                "SELECT id, email, name FROM users WHERE id = ANY($1)",
            )
            .bind(&user_ids)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

            let summaries = rows.into_iter().map(|r| {
                let uid: Uuid = r.get("user_id");
                let user = users.iter().find(|u| {
                    let id: Uuid = u.get("id");
                    id == uid
                });
                UsageSummary {
                    user_id: uid,
                    email: user.map(|u| u.get::<String, _>("email")).unwrap_or_default(),
                    name: user.map(|u| u.get::<String, _>("name")).unwrap_or_default(),
                    total_input_tokens: r.try_get::<i64, _>("total_input_tokens").unwrap_or(0),
                    total_output_tokens: r.try_get::<i64, _>("total_output_tokens").unwrap_or(0),
                    total_cost_cents: r.try_get::<i32, _>("total_cost_cents").unwrap_or(0) as i64,
                    session_count: r.try_get::<i64, _>("session_count").unwrap_or(0),
                    last_active_at: r.get("last_active_at"),
                }
            }).collect();
            Json(ApiResponse::success(summaries))
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

// ─── Remote sessions / messages / traces ─────────────────────────────────────

pub async fn create_session(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<SessionCreateRequest>,
) -> Json<ApiResponse<SessionOut>> {
    let id = Uuid::new_v4();
    let now = now_ms();
    let title = req.title.unwrap_or_else(|| "New conversation".into());
    let mode = req.mode.unwrap_or_else(|| "research".into());

    match sqlx::query(
        r#"
        INSERT INTO sessions (id, company_id, user_id, title, status, cwd, model, mode, created_at, updated_at)
        VALUES ($1, $2, $3, $4, 'idle', $5, $6, $7, $8, $8)
        "#,
    )
    .bind(id)
    .bind(auth.company_id)
    .bind(auth.user_id)
    .bind(&title)
    .bind(&req.cwd)
    .bind(&req.model)
    .bind(&mode)
    .bind(now)
    .execute(&state.db)
    .await
    {
        Ok(_) => Json(ApiResponse::success(SessionOut {
            id,
            company_id: auth.company_id,
            user_id: auth.user_id,
            title,
            status: "idle".into(),
            cwd: req.cwd,
            model: req.model,
            mode,
            created_at: now,
            updated_at: now,
        })),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn list_sessions(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Json<ApiResponse<Vec<SessionOut>>> {
    match sqlx::query(
        r#"
        SELECT id, company_id, user_id, title, status, cwd, model, mode, created_at, updated_at
        FROM sessions
        WHERE company_id = $1 AND user_id = $2
        ORDER BY updated_at DESC
        "#,
    )
    .bind(auth.company_id)
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => Json(ApiResponse::success(rows.into_iter().map(session_from_row).collect())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn get_session(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> Json<ApiResponse<SessionOut>> {
    match sqlx::query(
        r#"
        SELECT id, company_id, user_id, title, status, cwd, model, mode, created_at, updated_at
        FROM sessions
        WHERE id = $1 AND company_id = $2 AND user_id = $3
        "#,
    )
    .bind(session_id)
    .bind(auth.company_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => Json(ApiResponse::success(session_from_row(row))),
        Ok(None) => Json(ApiResponse::error("Session not found")),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn update_session(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
    Json(req): Json<SessionPatchRequest>,
) -> Json<ApiResponse<SessionOut>> {
    let now = now_ms();
    match sqlx::query(
        r#"
        UPDATE sessions
        SET title = COALESCE($4, title),
            status = COALESCE($5, status),
            model = COALESCE($6, model),
            mode = COALESCE($7, mode),
            updated_at = $8
        WHERE id = $1 AND company_id = $2 AND user_id = $3
        RETURNING id, company_id, user_id, title, status, cwd, model, mode, created_at, updated_at
        "#,
    )
    .bind(session_id)
    .bind(auth.company_id)
    .bind(auth.user_id)
    .bind(req.title)
    .bind(req.status)
    .bind(req.model)
    .bind(req.mode)
    .bind(now)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => Json(ApiResponse::success(session_from_row(row))),
        Ok(None) => Json(ApiResponse::error("Session not found")),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn delete_session(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> Json<ApiResponse<()>> {
    match sqlx::query(
        "DELETE FROM sessions WHERE id = $1 AND company_id = $2 AND user_id = $3",
    )
    .bind(session_id)
    .bind(auth.company_id)
    .bind(auth.user_id)
    .execute(&state.db)
    .await
    {
        Ok(result) if result.rows_affected() > 0 => Json(ApiResponse::success(())),
        Ok(_) => Json(ApiResponse::error("Session not found")),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn list_messages(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> Json<ApiResponse<Vec<MessageOut>>> {
    if !session_accessible(&state.db, session_id, auth.company_id, auth.user_id).await {
        return Json(ApiResponse::error("Session not found"));
    }

    match sqlx::query(
        "SELECT id, session_id, role, content, timestamp, token_usage FROM messages WHERE session_id = $1 ORDER BY timestamp ASC",
    )
    .bind(session_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => Json(ApiResponse::success(rows.into_iter().map(message_from_row).collect())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn create_message(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
    Json(req): Json<MessageCreateRequest>,
) -> Json<ApiResponse<MessageOut>> {
    if !session_accessible(&state.db, session_id, auth.company_id, auth.user_id).await {
        return Json(ApiResponse::error("Session not found"));
    }

    match sqlx::query(
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
    .fetch_one(&state.db)
    .await
    {
        Ok(row) => Json(ApiResponse::success(message_from_row(row))),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn list_trace_steps(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> Json<ApiResponse<Vec<TraceStepOut>>> {
    if !session_accessible(&state.db, session_id, auth.company_id, auth.user_id).await {
        return Json(ApiResponse::error("Session not found"));
    }

    match sqlx::query(
        "SELECT id, session_id, type, status, title, content, tool_name, tool_input, tool_output, is_error, timestamp, duration FROM trace_steps WHERE session_id = $1 ORDER BY timestamp ASC",
    )
    .bind(session_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => Json(ApiResponse::success(rows.into_iter().map(trace_step_from_row).collect())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn create_trace_step(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
    Json(req): Json<TraceStepCreateRequest>,
) -> Json<ApiResponse<TraceStepOut>> {
    if !session_accessible(&state.db, session_id, auth.company_id, auth.user_id).await {
        return Json(ApiResponse::error("Session not found"));
    }

    match sqlx::query(
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
    .bind(req.r#type)
    .bind(req.status)
    .bind(req.title)
    .bind(req.content)
    .bind(req.tool_name)
    .bind(req.tool_input)
    .bind(req.tool_output)
    .bind(req.is_error)
    .bind(req.timestamp)
    .bind(req.duration)
    .fetch_one(&state.db)
    .await
    {
        Ok(row) => Json(ApiResponse::success(trace_step_from_row(row))),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn update_trace_step(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path((session_id, trace_id)): axum::extract::Path<(Uuid, Uuid)>,
    Json(req): Json<TraceStepPatchRequest>,
) -> Json<ApiResponse<TraceStepOut>> {
    if !session_accessible(&state.db, session_id, auth.company_id, auth.user_id).await {
        return Json(ApiResponse::error("Session not found"));
    }

    match sqlx::query(
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
    .bind(req.r#type)
    .bind(req.status)
    .bind(req.title)
    .bind(req.content)
    .bind(req.tool_name)
    .bind(req.tool_input)
    .bind(req.tool_output)
    .bind(req.is_error)
    .bind(req.timestamp)
    .bind(req.duration)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => Json(ApiResponse::success(trace_step_from_row(row))),
        Ok(None) => Json(ApiResponse::error("Trace step not found")),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

// ─── Vexa integration routes ────────────────────────────────────────────────

pub async fn vexa_request_bot(
    State(state): State<AppState>,
    _auth: AuthContext,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> Json<ApiResponse<serde_json::Value>> {
    match reqwest::Client::new()
        .post(format!("{}/bots", state.settings.vexa_api_url))
        .header("X-API-Key", &state.settings.vexa_admin_token)
        .json(&body)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            match resp.json::<serde_json::Value>().await {
                Ok(json) => Json(ApiResponse::success(json)),
                Err(_) => Json(ApiResponse::error(format!("Vexa request failed: {}", status))),
            }
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn vexa_get_meetings(
    State(state): State<AppState>,
    _auth: AuthContext,
) -> Json<ApiResponse<serde_json::Value>> {
    match reqwest::Client::new()
        .get(format!("{}/meetings", state.settings.vexa_api_url))
        .header("X-API-Key", &state.settings.vexa_admin_token)
        .send()
        .await
    {
        Ok(resp) => {
            match resp.json::<serde_json::Value>().await {
                Ok(json) => Json(ApiResponse::success(json)),
                Err(_) => Json(ApiResponse::error("Failed to parse Vexa response")),
            }
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

// ─── Health check ────────────────────────────────────────────────────────────

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

// ─── Router setup ────────────────────────────────────────────────────────────

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/auth/register/admin", post(register_admin))
        .route("/auth/register/member", post(register_member))
        .route("/auth/signin", post(sign_in))
        .route("/auth/signout", post(sign_out))
        .route("/auth/me", get(auth_me))
        .route("/company/config", get(get_company_config))
        .route("/company/config", put(update_company_config))
        .route("/company/members", get(list_members))
        .route("/company/members/:user_id", delete(remove_member))
        .route("/company/members/:user_id/:role", put(update_member_role))
        .route("/company/invites", get(list_invites))
        .route("/company/invites", post(add_invite))
        .route("/company/invites/:invite_id", delete(remove_invite))
        .route("/company/apikeys/:user_id", get(list_api_keys))
        .route("/company/apikeys", post(set_api_key))
        .route("/company/apikeys/key/:key_id", delete(delete_api_key))
        .route("/meetings", get(list_meetings))
        .route("/meetings", post(ingest_meeting))
        .route("/knowledge/search", post(search_knowledge))
        .route("/sessions", get(list_sessions))
        .route("/sessions", post(create_session))
        .route("/sessions/:session_id", get(get_session))
        .route("/sessions/:session_id", patch(update_session))
        .route("/sessions/:session_id", delete(delete_session))
        .route("/sessions/:session_id/messages", get(list_messages))
        .route("/sessions/:session_id/messages", post(create_message))
        .route("/sessions/:session_id/traces", get(list_trace_steps))
        .route("/sessions/:session_id/traces", post(create_trace_step))
        .route("/sessions/:session_id/traces/:trace_id", patch(update_trace_step))
        .route("/usage", post(record_usage))
        .route("/usage/summary", get(get_usage_summary))
        .route("/vexa/bots", post(vexa_request_bot))
        .route("/vexa/meetings", get(vexa_get_meetings))
        .with_state(state)
        .layer(
            tower_http::cors::CorsLayer::permissive(),
        )
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

async fn session_accessible(
    db: &sqlx::PgPool,
    session_id: Uuid,
    company_id: Uuid,
    user_id: Uuid,
) -> bool {
    matches!(
        sqlx::query(
            "SELECT 1 FROM sessions WHERE id = $1 AND company_id = $2 AND user_id = $3",
        )
        .bind(session_id)
        .bind(company_id)
        .bind(user_id)
        .fetch_optional(db)
        .await,
        Ok(Some(_))
    )
}

fn session_from_row(row: sqlx::postgres::PgRow) -> SessionOut {
    SessionOut {
        id: row.get("id"),
        company_id: row.get("company_id"),
        user_id: row.get("user_id"),
        title: row.get("title"),
        status: row.get("status"),
        cwd: row.get("cwd"),
        model: row.get("model"),
        mode: row.get("mode"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn message_from_row(row: sqlx::postgres::PgRow) -> MessageOut {
    MessageOut {
        id: row.get("id"),
        session_id: row.get("session_id"),
        role: row.get("role"),
        content: row.get("content"),
        timestamp: row.get("timestamp"),
        token_usage: row.get("token_usage"),
    }
}

fn trace_step_from_row(row: sqlx::postgres::PgRow) -> TraceStepOut {
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

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        "••••••••".into()
    } else {
        format!("{}••••••••{}", &key[..4], &key[key.len()-4..])
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
