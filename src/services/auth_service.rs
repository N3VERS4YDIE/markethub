use std::sync::Arc;

use validator::Validate;

use crate::{
    error::AppError,
    models::user::{AuthTokenResponse, LoginRequest, PublicUser, RegisterUserRequest, User},
    repositories::UserRepository,
    utils::{jwt::JwtConfig, password},
};

#[derive(Clone)]
pub struct AuthService {
    users: UserRepository,
    jwt: Arc<JwtConfig>,
}

impl AuthService {
    pub fn new(users: UserRepository, jwt: Arc<JwtConfig>) -> Self {
        Self { users, jwt }
    }

    pub async fn register(&self, payload: RegisterUserRequest) -> crate::Result<AuthTokenResponse> {
        payload
            .validate()
            .map_err(|err| AppError::Validation(err.to_string()))?;

        if self.users.email_exists(&payload.email).await? {
            return Err(AppError::Conflict("Email already registered".into()));
        }

        let password_hash =
            password::hash_password(&payload.password).map_err(AppError::Internal)?;

        let user = self
            .users
            .create(
                &payload.email,
                &password_hash,
                &payload.full_name,
                payload.phone.as_deref(),
            )
            .await?;

        self.build_response(user)
    }

    pub async fn login(&self, payload: LoginRequest) -> crate::Result<AuthTokenResponse> {
        payload
            .validate()
            .map_err(|err| AppError::Validation(err.to_string()))?;

        let user = self
            .users
            .find_by_email(&payload.email)
            .await?
            .ok_or_else(|| AppError::Authentication("Invalid credentials".into()))?;

        let is_valid = password::verify_password(&payload.password, &user.password_hash)
            .map_err(AppError::Internal)?;

        if !is_valid {
            return Err(AppError::Authentication("Invalid credentials".into()));
        }

        self.build_response(user)
    }

    fn build_response(&self, user: User) -> crate::Result<AuthTokenResponse> {
        let claims = self.jwt.claims_for(user.id, user.email.clone());
        let token = self
            .jwt
            .generate(&claims)
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(AuthTokenResponse {
            token,
            user: PublicUser::from(user),
        })
    }
}
