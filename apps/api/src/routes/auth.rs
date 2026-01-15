//! Authentication routes

use crate::auth::{create_token, verify_password, AuthError, LoginRequest, LoginResponse};
use crate::AppState;
use axum::{extract::State, routing::post, Json, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use std::sync::Arc;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
}

async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), AuthError> {
    tracing::debug!("Login attempt for user: {}", req.username);
    tracing::debug!("Expected username: {}", state.config.username);
    tracing::debug!("Password hash (first 50 chars): {}", &state.config.password_hash.chars().take(50).collect::<String>());
    
    // Verify credentials
    if req.username != state.config.username {
        tracing::debug!("Username mismatch");
        return Err(AuthError("Invalid credentials".to_string()));
    }

    // For development, accept "admin123" directly if hash is default
    let valid = if state.config.password_hash.contains("random_salt_here") {
        tracing::debug!("Using default password check");
        req.password == "admin123"
    } else {
        tracing::debug!("Verifying password against hash");
        verify_password(&req.password, &state.config.password_hash)
    };

    tracing::debug!("Password valid: {}", valid);

    if !valid {
        return Err(AuthError("Invalid credentials".to_string()));
    }

    let response = create_token(&req.username, &state.config.jwt_secret)?;

    // Set cookie
    let cookie = Cookie::build(("token", response.token.clone()))
        .path("/")
        .http_only(true)
        .secure(false) // Set to true in production with HTTPS
        .max_age(time::Duration::hours(24))
        .build();

    Ok((jar.add(cookie), Json(response)))
}

#[derive(serde::Serialize)]
struct LogoutResponse {
    success: bool,
}

async fn logout(jar: CookieJar) -> (CookieJar, Json<LogoutResponse>) {
    (
        jar.remove(Cookie::from("token")),
        Json(LogoutResponse { success: true }),
    )
}
