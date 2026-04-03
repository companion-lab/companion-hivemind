use axum::extract::State;
use axum::response::Json;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthContext;
use crate::repos::usage::UsageRepo;
use crate::types::{UsageRecord, UsageSummary};

pub async fn record(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<UsageRecord>,
) -> Result<Json<()>, AppError> {
    let repo = UsageRepo::new(state.db.clone());
    let now = crate::util::now_ms();
    repo.record(auth.company_id, req, now).await?;
    Ok(Json(()))
}

pub async fn summary(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<Vec<UsageSummary>>, AppError> {
    let repo = UsageRepo::new(state.db.clone());
    let summaries = repo.get_summary(auth.company_id).await?;
    Ok(Json(summaries))
}
