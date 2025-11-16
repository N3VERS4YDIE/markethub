use anyhow::Context;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .context("Invalid PORT")?,
            database_url: env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            jwt_secret: env::var("JWT_SECRET").context("JWT_SECRET must be set")?,
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .context("Invalid JWT_EXPIRATION_HOURS")?,
        })
    }
}
