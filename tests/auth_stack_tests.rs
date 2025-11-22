use std::sync::Arc;

use markethub::{
    error::AppError,
    models::user::{LoginRequest, RegisterUserRequest},
    repositories::UserRepository,
    services::AuthService,
    utils::jwt::JwtConfig,
};
use sqlx::PgPool;

fn auth_service(pool: &PgPool) -> AuthService {
    let users = UserRepository::new(pool.clone());
    AuthService::new(users, Arc::new(JwtConfig::new("test-secret", 4)))
}

fn register_payload(email: &str) -> RegisterUserRequest {
    RegisterUserRequest {
        email: email.to_string(),
        password: "StrongPass123!".into(),
        full_name: "Test User".into(),
        phone: Some("+1234567890".into()),
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn register_and_login_round_trip(pool: PgPool) {
    let service = auth_service(&pool);
    let payload = register_payload("alice@example.com");

    let register = service.register(payload.clone()).await.unwrap();
    assert_eq!(register.user.email, "alice@example.com");
    assert!(!register.token.is_empty());

    let login = service
        .login(LoginRequest {
            email: "alice@example.com".into(),
            password: "StrongPass123!".into(),
        })
        .await
        .unwrap();

    assert_eq!(login.user.id, register.user.id);
    assert!(!login.token.is_empty(), "login should return a token");
}

#[sqlx::test(migrations = "./migrations")]
async fn duplicate_emails_are_rejected(pool: PgPool) {
    let service = auth_service(&pool);
    let payload = register_payload("duplicate@example.com");

    service.register(payload.clone()).await.unwrap();
    let err = service.register(payload).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
}

#[sqlx::test(migrations = "./migrations")]
async fn invalid_credentials_fail_login(pool: PgPool) {
    let service = auth_service(&pool);
    let payload = register_payload("bob@example.com");

    service.register(payload).await.unwrap();
    let err = service
        .login(LoginRequest {
            email: "bob@example.com".into(),
            password: "TotallyWrong".into(),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Authentication(_)));
}
