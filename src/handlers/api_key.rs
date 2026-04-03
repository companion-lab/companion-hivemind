use axum::extract::{Path, State};
use axum::response::Json;
use uuid::Uuid;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthContext;
use crate::repos::api_key::ApiKeyRepo;
use crate::services::crypto::CryptoService;
use crate::types::{ApiKeyOut, MemberApiKeySet};

pub async fn set(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<MemberApiKeySet>,
) -> Result<Json<ApiKeyOut>, AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let encrypted = CryptoService::encrypt(&req.plain_key, &state.settings.encryption_secret);
    let now = crate::util::now_ms();

    let repo = ApiKeyRepo::new(state.db.clone());
    let key = repo.upsert(auth.company_id, req, encrypted, now).await?;
    Ok(Json(key))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<ApiKeyOut>>, AppError> {
    let repo = ApiKeyRepo::new(state.db.clone());
    let keys = repo.list(user_id, auth.company_id).await?;
    Ok(Json(keys))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(key_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let repo = ApiKeyRepo::new(state.db.clone());
    repo.delete(key_id, auth.company_id).await?;
    Ok(Json(()))
}
