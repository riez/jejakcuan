//! JejakCuan API library
//!
//! This module exports the API router and related components for both
//! the main server binary and integration tests.

use axum::{
    http::{header, HeaderValue, Method},
    routing::get,
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

pub mod auth;
pub mod config;
pub mod notifications;
pub mod routes;

use config::Config;
use routes::{
    admin_routes, analysis_routes, auth_routes, financials_routes, stock_routes, streaming_routes,
    watchlist_routes, JobManager,
};

/// Application state shared across all handlers
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub job_manager: Arc<JobManager>,
}

/// Create the application router with all routes configured
pub fn create_app(db: PgPool, config: Config) -> Router {
    let job_manager = Arc::new(JobManager::new());
    let state = Arc::new(AppState {
        db,
        config,
        job_manager,
    });

    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/api/auth", auth_routes())
        .nest("/api/stocks", stock_routes())
        .nest("/api/financials", financials_routes())
        .nest("/api/analysis", analysis_routes())
        .nest("/api/watchlist", watchlist_routes())
        .nest("/api", streaming_routes())
        .nest("/api/admin", admin_routes())
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::list([
                    "http://localhost:5173".parse::<HeaderValue>().unwrap(),
                    "http://localhost:3000".parse::<HeaderValue>().unwrap(),
                    "http://127.0.0.1:5173".parse::<HeaderValue>().unwrap(),
                ]))
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                    header::ACCEPT,
                    header::COOKIE,
                ])
                .allow_credentials(true),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn root() -> &'static str {
    "JejakCuan API v0.1.0"
}

async fn health() -> &'static str {
    "OK"
}

#[cfg(test)]
pub mod test_utils {
    //! Test utilities for API testing

    use super::*;

    /// Create a test configuration
    pub fn test_config() -> Config {
        Config {
            database_url: "postgres://test:test@localhost:5432/test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            jwt_secret: "test_secret_for_testing_only".to_string(),
            username: "admin".to_string(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$random_salt_here$hashed_password"
                .to_string(),
            host: "127.0.0.1".to_string(),
            port: 0, // Random port for testing
        }
    }
}
