use axum::extract::State;
use axum::response::Json;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthContext;
use crate::repos::meeting::MeetingRepo;
use crate::services::knowledge::KnowledgeService;
use crate::types::{MeetingIngestRequest, MeetingOut};

pub async fn ingest(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<MeetingIngestRequest>,
) -> Result<Json<MeetingOut>, AppError> {
    let repo = MeetingRepo::new(state.db.clone());
    let now = crate::util::now_ms();
    let meeting_out = repo.create(auth.company_id, req.clone(), now).await?;

    // Ingest knowledge chunks into vector store
    let docs = KnowledgeService::chunk_transcript(&req.transcript, meeting_out.id, req.date);
    if let Err(e) = KnowledgeService::ingest_documents(
        &state.db,
        &state.vector_store,
        auth.company_id,
        meeting_out.id,
        &docs,
    )
    .await
    {
        tracing::error!(error = %e, "Failed to ingest knowledge chunks into vector store");
    }

    Ok(Json(meeting_out))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<Vec<MeetingOut>>, AppError> {
    let repo = MeetingRepo::new(state.db.clone());
    let meetings = repo.list(auth.company_id).await?;
    Ok(Json(meetings))
}
