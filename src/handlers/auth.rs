use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Json;

use crate::AppState;
use crate::errors::AppError;
use crate::repos::auth::AuthRepo;
use crate::types::{AuthSession, RegisterAdminRequest, RegisterMemberRequest, SignInRequest};

pub async fn register_admin(
    State(state): State<AppState>,
    Json(req): Json<RegisterAdminRequest>,
) -> Result<Json<AuthSession>, AppError> {
    let repo = AuthRepo::new(state.db.clone());
    let session = state.services.auth.register_admin(&repo, &state.settings, req).await?;
    Ok(Json(session))
}

pub async fn register_member(
    State(state): State<AppState>,
    Json(req): Json<RegisterMemberRequest>,
) -> Result<Json<AuthSession>, AppError> {
    let repo = AuthRepo::new(state.db.clone());
    let session = state.services.auth.register_member(&repo, &state.settings, req).await?;
    Ok(Json(session))
}

pub async fn sign_in(
    State(state): State<AppState>,
    Json(req): Json<SignInRequest>,
) -> Result<Json<AuthSession>, AppError> {
    let repo = AuthRepo::new(state.db.clone());
    let session = state.services.auth.sign_in(&repo, &state.settings, req).await?;
    Ok(Json(session))
}

pub async fn sign_out(
    State(state): State<AppState>,
    auth: crate::middleware::AuthContext,
) -> Result<Json<()>, AppError> {
    let repo = AuthRepo::new(state.db.clone());
    repo.delete_auth_tokens(auth.user_id, auth.company_id).await?;
    Ok(Json(()))
}

pub async fn auth_me(
    State(state): State<AppState>,
    auth: crate::middleware::AuthContext,
    headers: HeaderMap,
) -> Result<Json<AuthSession>, AppError> {
    let repo = AuthRepo::new(state.db.clone());
    let ctx = repo.find_user_context(auth.user_id, auth.company_id).await?
        .ok_or_else(|| AppError::NotFound("User context not found".into()))?;

    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or_default()
        .to_string();

    let session = AuthSession::new(
        auth.user_id,
        ctx.email,
        ctx.name,
        auth.company_id,
        ctx.company_name,
        ctx.company_slug,
        ctx.role,
        token,
    );

    Ok(Json(session))
}
