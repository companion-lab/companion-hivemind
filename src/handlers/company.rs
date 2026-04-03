use axum::extract::State;
use axum::response::Json;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthContext;
use crate::repos::company::CompanyRepo;
use crate::types::{CompanyConfigOut, CompanyConfigPatch};

pub async fn get_config(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<CompanyConfigOut>, AppError> {
    let repo = CompanyRepo::new(state.db.clone());
    repo.get_config(auth.company_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Company config not found".into()))
        .map(Json)
}

pub async fn update_config(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(patch): Json<CompanyConfigPatch>,
) -> Result<Json<CompanyConfigOut>, AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let repo = CompanyRepo::new(state.db.clone());
    let now = crate::util::now_ms();
    let config = repo.update_config(auth.company_id, patch, now).await?;
    Ok(Json(config))
}
