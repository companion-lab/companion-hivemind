use axum::extract::{Path, State};
use axum::response::Json;
use uuid::Uuid;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthContext;
use crate::repos::session::SessionRepo;
use crate::types::{SessionCreateRequest, SessionOut, SessionPatchRequest};

pub async fn create(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<SessionCreateRequest>,
) -> Result<Json<SessionOut>, AppError> {
    let repo = SessionRepo::new(state.db.clone());
    let now = crate::util::now_ms();
    let session = repo.create(auth.company_id, auth.user_id, req, now).await?;
    Ok(Json(session))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<Vec<SessionOut>>, AppError> {
    let repo = SessionRepo::new(state.db.clone());
    let sessions = repo.list(auth.company_id, auth.user_id).await?;
    Ok(Json(sessions))
}

pub async fn get(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(session_id): Path<Uuid>,
) -> Result<Json<SessionOut>, AppError> {
    let repo = SessionRepo::new(state.db.clone());
    repo.get_by_id(session_id, auth.company_id, auth.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Session not found".into()))
        .map(Json)
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(session_id): Path<Uuid>,
    Json(req): Json<SessionPatchRequest>,
) -> Result<Json<SessionOut>, AppError> {
    let repo = SessionRepo::new(state.db.clone());
    let now = crate::util::now_ms();
    repo.update(session_id, auth.company_id, auth.user_id, req, now)
        .await?
        .ok_or_else(|| AppError::NotFound("Session not found".into()))
        .map(Json)
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(session_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    let repo = SessionRepo::new(state.db.clone());
    let affected = repo.delete(session_id, auth.company_id, auth.user_id).await?;
    if affected == 0 {
        return Err(AppError::NotFound("Session not found".into()));
    }
    Ok(Json(()))
}
