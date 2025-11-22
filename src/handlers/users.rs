use axum::{extract::State, routing::get, Json, Router};

use crate::{
    middleware::auth::AuthenticatedUser,
    models::{self, user::UserProfileResponse},
    repositories::UserRepository,
    services::UserService,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

async fn me(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> crate::Result<Json<models::ApiResponse<UserProfileResponse>>> {
    let service = UserService::new(UserRepository::new(state.db.clone()));
    let profile = service.get_profile(user.user_id).await?;
    let response = UserProfileResponse { user: profile };
    Ok(Json(models::ApiResponse::new(response)))
}
