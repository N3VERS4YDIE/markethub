use axum::{extract::State, routing::post, Json, Router};

use crate::{
    models::{
        self,
        user::{AuthTokenResponse, LoginRequest, RegisterUserRequest},
    },
    repositories::UserRepository,
    services::AuthService,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> crate::Result<Json<models::ApiResponse<AuthTokenResponse>>> {
    let service = auth_service(&state);
    let response = service.register(payload).await?;
    Ok(Json(models::ApiResponse::new(response)))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> crate::Result<Json<models::ApiResponse<AuthTokenResponse>>> {
    let service = auth_service(&state);
    let response = service.login(payload).await?;
    Ok(Json(models::ApiResponse::new(response)))
}

fn auth_service(state: &AppState) -> AuthService {
    AuthService::new(UserRepository::new(state.db.clone()), state.jwt.clone())
}
