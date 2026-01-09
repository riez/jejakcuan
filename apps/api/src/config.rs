//! Application configuration

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub username: String,
    pub password_hash: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://jejakcuan:jejakcuan_dev@localhost:5432/jejakcuan".to_string()
            }),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "development_secret_change_in_production".to_string()),
            username: env::var("AUTH_USERNAME").unwrap_or_else(|_| "admin".to_string()),
            password_hash: env::var("AUTH_PASSWORD_HASH").unwrap_or_else(|_| {
                // Default password: "admin123" - CHANGE IN PRODUCTION
                "$argon2id$v=19$m=19456,t=2,p=1$random_salt_here$hashed_password".to_string()
            }),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
        }
    }
}
