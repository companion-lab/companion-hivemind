use uuid::Uuid;

use crate::config::Settings;
use crate::errors::AppError;
use crate::repos::auth::{self, AuthRepo};
use crate::types::{AuthSession, RegisterAdminRequest, RegisterMemberRequest, SignInRequest};

#[derive(Clone)]
pub struct AuthService;

impl AuthService {
    pub async fn register_admin(
        &self,
        repo: &AuthRepo,
        settings: &Settings,
        req: RegisterAdminRequest,
    ) -> Result<AuthSession, AppError> {
        if req.company_name.trim().is_empty() {
            return Err(AppError::BadRequest("Company name is required".into()));
        }
        if !req.email.contains('@') {
            return Err(AppError::BadRequest("Valid email is required".into()));
        }
        if req.password.len() < 8 {
            return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
        }

        if repo.find_user_by_email(&req.email).await?.is_some() {
            return Err(AppError::BadRequest("Email already registered".into()));
        }

        let now = crate::util::now_ms();
        let company_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let slug = req.company_slug.clone().unwrap_or_else(|| auth::slugify(&req.company_name));
        let password_hash = auth::hash_password(&req.password)?;

        repo.create_company(company_id, &req.company_name, &slug, now).await?;
        repo.create_user(user_id, &req.email, &req.name, &password_hash, now).await?;
        repo.create_membership(Uuid::new_v4(), company_id, user_id, "admin", now).await?;
        repo.create_default_config(company_id, now).await?;

        let token = auth::create_token(
            &settings.jwt_secret,
            user_id,
            company_id,
            "admin",
            settings.jwt_ttl_seconds,
        )?;
        let expires_at = now + (settings.jwt_ttl_seconds * 1000);
        repo.create_auth_token(&token, user_id, company_id, now, expires_at).await?;

        Ok(AuthSession::new(
            user_id,
            req.email,
            req.name,
            company_id,
            req.company_name,
            slug,
            "admin".into(),
            token,
        ))
    }

    pub async fn register_member(
        &self,
        repo: &AuthRepo,
        settings: &Settings,
        req: RegisterMemberRequest,
    ) -> Result<AuthSession, AppError> {
        if !req.email.contains('@') {
            return Err(AppError::BadRequest("Valid email is required".into()));
        }
        if req.password.len() < 8 {
            return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
        }

        let company = repo
            .find_company_by_slug(&req.company_slug)
            .await?
            .ok_or_else(|| AppError::BadRequest("Company not found".into()))?;

        let invite = repo
            .find_invite(&req.email, company.id)
            .await?
            .ok_or_else(|| AppError::BadRequest("No invitation found".into()))?;

        if repo.find_user_by_email(&req.email).await?.is_some() {
            return Err(AppError::BadRequest("Email already registered".into()));
        }

        let now = crate::util::now_ms();
        let user_id = Uuid::new_v4();
        let password_hash = auth::hash_password(&req.password)?;

        repo.create_user(user_id, &req.email, &req.name, &password_hash, now).await?;
        repo.create_membership(Uuid::new_v4(), company.id, user_id, &invite.role, now).await?;
        repo.mark_invite_used(invite.id, now).await?;

        let token = auth::create_token(
            &settings.jwt_secret,
            user_id,
            company.id,
            &invite.role,
            settings.jwt_ttl_seconds,
        )?;
        let expires_at = now + (settings.jwt_ttl_seconds * 1000);
        repo.create_auth_token(&token, user_id, company.id, now, expires_at).await?;

        Ok(AuthSession::new(
            user_id,
            req.email,
            req.name,
            company.id,
            company.name,
            company.slug,
            invite.role,
            token,
        ))
    }

    pub async fn sign_in(
        &self,
        repo: &AuthRepo,
        settings: &Settings,
        req: SignInRequest,
    ) -> Result<AuthSession, AppError> {
        let user = repo
            .find_user_by_email(&req.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("No account found".into()))?;

        if !auth::verify_password(&req.password, &user.password_hash)? {
            return Err(AppError::Unauthorized("Incorrect password".into()));
        }

        let membership = repo
            .find_membership(user.id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("No company membership found".into()))?;

        let company = repo.find_company_by_id(membership.company_id).await?;

        let now = crate::util::now_ms();
        let token = auth::create_token(
            &settings.jwt_secret,
            user.id,
            company.id,
            &membership.role,
            settings.jwt_ttl_seconds,
        )?;
        let expires_at = now + (settings.jwt_ttl_seconds * 1000);
        repo.create_auth_token(&token, user.id, company.id, now, expires_at).await?;

        Ok(AuthSession::new(
            user.id,
            user.email,
            user.name,
            company.id,
            company.name,
            company.slug,
            membership.role,
            token,
        ))
    }
}
