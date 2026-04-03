use axum::http::StatusCode;
use axum::response::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::AppState;
use crate::repos::auth;
use crate::types::Claims;

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

        Ok(AuthContext {
            user_id: claims.user_id,
            company_id: claims.company_id,
            role: claims.role,
        })
    }
}
