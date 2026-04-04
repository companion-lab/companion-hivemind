use axum::http::StatusCode;
use axum::response::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::AppState;
use crate::repos::auth;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    ok: bool,
    error: String,
}

#[axum::async_trait]
impl axum::extract::FromRequestParts<AppState> for AuthContext {
    type Rejection = (StatusCode, Json<ApiError>);

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
                Json(ApiError {
                    ok: false,
                    error: "Missing Authorization header".into(),
                }),
            ))?;

        let token = auth_header.strip_prefix("Bearer ").ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                ok: false,
                error: "Invalid Authorization format".into(),
            }),
        ))?;

        let claims = auth::validate_token(&state.settings.jwt_secret, token).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    ok: false,
                    error: "Invalid or expired token".into(),
                }),
            )
        })?;

        // Validate that the company still exists in the database.
        // This prevents FK violations when the DB was reset but the client
        // is still using an old JWT token.
        let company_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM companies WHERE id = $1)",
        )
        .bind(claims.company_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to validate company: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    ok: false,
                    error: "Internal server error".into(),
                }),
            )
        })?;

        if !company_exists {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    ok: false,
                    error: "Company not found. Please re-authenticate.".into(),
                }),
            ));
        }

        Ok(AuthContext {
            user_id: claims.user_id,
            company_id: claims.company_id,
            role: claims.role,
        })
    }
}
