use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        // Log full error chain for debugging
        tracing::error!(error = %self, "Request error");
        if let AppError::Internal(e) = &self {
            tracing::error!("Internal error chain: {:?}", e);
        }

        let body = Json(serde_json::json!({
            "ok": false,
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::Database(db_err) => {
                let msg = db_err.message();
                // Check for unique constraint violations
                if msg.contains("unique") || msg.contains("duplicate") || msg.contains("already exists") || msg.contains("UNIQUE") {
                    return AppError::Conflict(msg.to_string());
                }
                // Check for foreign key violations
                if msg.contains("foreign key") || msg.contains("violates") {
                    return AppError::BadRequest(msg.to_string());
                }
                AppError::Internal(err.into())
            }
            _ => AppError::Internal(err.into()),
        }
    }
}
