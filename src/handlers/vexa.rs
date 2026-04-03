use axum::response::Json;
use axum::extract::State;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthContext;

pub async fn request_bot(
    State(state): State<AppState>,
    _auth: AuthContext,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/bots", state.settings.vexa_api_url))
        .header("X-API-Key", &state.settings.vexa_admin_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let status = resp.status();
    let json = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Vexa request failed: {}", status)))?;

    Ok(Json(json))
}

pub async fn get_meetings(
    State(state): State<AppState>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, AppError> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/meetings", state.settings.vexa_api_url))
        .header("X-API-Key", &state.settings.vexa_admin_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let json = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to parse Vexa response")))?;

    Ok(Json(json))
}
