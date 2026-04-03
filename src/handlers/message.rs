use axum::extract::{Path, State};
use axum::response::Json;
use uuid::Uuid;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthContext;
use crate::repos::message::MessageRepo;
use crate::repos::session::SessionRepo;
use crate::types::{MessageCreateRequest, MessageOut};

pub async fn list(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Vec<MessageOut>>, AppError> {
    ensure_session_accessible(&state, session_id, auth.company_id, auth.user_id).await?;

    let repo = MessageRepo::new(state.db.clone());
    let messages = repo.list_by_session(session_id).await?;
    Ok(Json(messages))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(session_id): Path<Uuid>,
    Json(req): Json<MessageCreateRequest>,
) -> Result<Json<MessageOut>, AppError> {
    ensure_session_accessible(&state, session_id, auth.company_id, auth.user_id).await?;

    let repo = MessageRepo::new(state.db.clone());
    let message = repo.create(session_id, req).await?;
    Ok(Json(message))
}

async fn ensure_session_accessible(
    state: &AppState,
    session_id: Uuid,
    company_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let repo = SessionRepo::new(state.db.clone());
    if !repo.is_accessible(session_id, company_id, user_id).await? {
        return Err(AppError::NotFound("Session not found".into()));
    }
    Ok(())
}
