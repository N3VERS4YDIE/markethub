use axum::{
    extract::FromRequestParts,
    http::{self, request::Parts},
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let token = bearer_token(parts).map(|value| value.to_string());
        let jwt = state.jwt.clone();

        async move {
            let token =
                token.ok_or_else(|| AppError::Authentication("Missing bearer token".into()))?;

            let claims = jwt
                .verify(&token)
                .map_err(|_| AppError::Authentication("Invalid token".into()))?;

            Ok(Self {
                user_id: claims.sub,
                email: claims.email,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaybeAuthenticatedUser(pub Option<AuthenticatedUser>);

impl FromRequestParts<AppState> for MaybeAuthenticatedUser {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let token = bearer_token(parts).map(|value| value.to_string());
        let jwt = state.jwt.clone();

        async move {
            match token {
                Some(token) => {
                    let claims = jwt
                        .verify(&token)
                        .map_err(|_| AppError::Authentication("Invalid token".into()))?;
                    Ok(Self(Some(AuthenticatedUser {
                        user_id: claims.sub,
                        email: claims.email,
                    })))
                }
                None => Ok(Self(None)),
            }
        }
    }
}

fn bearer_token(parts: &mut Parts) -> Option<&str> {
    parts
        .headers
        .get(http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}
