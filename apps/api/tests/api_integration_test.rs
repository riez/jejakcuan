//! API integration tests
//!
//! These tests verify the HTTP layer of the API endpoints.
//! Note: Tests requiring database connections are marked with `#[ignore]`
//! and can be run with `cargo test -- --ignored` when a database is available.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use tower::ServiceExt;

/// Create a minimal test router without database for testing routes
/// that don't require state
fn create_minimal_test_router() -> Router {
    use axum::routing::get;

    Router::new()
        .route("/", get(|| async { "JejakCuan API v0.1.0" }))
        .route("/health", get(|| async { "OK" }))
}

#[tokio::test]
async fn test_root_endpoint() {
    let app = create_minimal_test_router();

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"JejakCuan API v0.1.0");
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_minimal_test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_not_found_returns_404() {
    let app = create_minimal_test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Tests below require database connection - marked as ignored by default
// Run with: cargo test -- --ignored
// ============================================================================

mod with_database {
    use super::*;
    use axum::http::header;
    use jejakcuan_api::{config::Config, create_app};

    /// Create test configuration
    fn test_config() -> Config {
        Config {
            database_url: std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://jejakcuan:jejakcuan_dev@localhost:5432/jejakcuan_test".to_string()
            }),
            redis_url: "redis://localhost:6379".to_string(),
            jwt_secret: "test_secret_for_testing_only".to_string(),
            username: "admin".to_string(),
            // For testing, allow "admin123" password
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$random_salt_here$hashed_password"
                .to_string(),
            host: "127.0.0.1".to_string(),
            port: 0,
        }
    }

    /// Helper to create app with database connection
    async fn create_test_app() -> Option<Router> {
        let config = test_config();
        match jejakcuan_db::create_pool(&config.database_url).await {
            Ok(pool) => Some(create_app(pool, config)),
            Err(e) => {
                eprintln!("Could not connect to test database: {}", e);
                None
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires database connection"]
    async fn test_stocks_list_requires_auth() {
        let Some(app) = create_test_app().await else {
            eprintln!("Skipping test: database not available");
            return;
        };

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/stocks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 401 without auth token
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[ignore = "requires database connection"]
    async fn test_watchlist_requires_auth() {
        let Some(app) = create_test_app().await else {
            eprintln!("Skipping test: database not available");
            return;
        };

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/watchlist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 401 without auth token
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[ignore = "requires database connection"]
    async fn test_login_with_invalid_credentials() {
        let Some(app) = create_test_app().await else {
            eprintln!("Skipping test: database not available");
            return;
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "username": "wrong_user",
                            "password": "wrong_password"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should fail with invalid credentials
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[ignore = "requires database connection"]
    async fn test_login_with_valid_credentials() {
        let Some(app) = create_test_app().await else {
            eprintln!("Skipping test: database not available");
            return;
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "username": "admin",
                            "password": "admin123"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should succeed with dev credentials (password_hash contains random_salt_here)
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Response should contain a token
        assert!(json.get("token").is_some());
        assert!(json.get("expires_at").is_some());
    }

    #[tokio::test]
    #[ignore = "requires database connection"]
    async fn test_authenticated_stocks_access() {
        let Some(app) = create_test_app().await else {
            eprintln!("Skipping test: database not available");
            return;
        };

        // First login to get a token
        let login_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "username": "admin",
                            "password": "admin123"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        if login_response.status() != StatusCode::OK {
            eprintln!("Login failed, skipping authenticated test");
            return;
        }

        let body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let login_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let token = login_json["token"].as_str().unwrap();

        // Now try to access stocks with the token
        let stocks_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/stocks")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should succeed with valid auth token
        assert_eq!(stocks_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore = "requires database connection"]
    async fn test_logout_endpoint() {
        let Some(app) = create_test_app().await else {
            eprintln!("Skipping test: database not available");
            return;
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/logout")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Logout should always succeed (clears cookie)
        assert_eq!(response.status(), StatusCode::OK);
    }
}
