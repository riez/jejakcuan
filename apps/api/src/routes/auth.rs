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
    // Verify credentials
    if req.username != state.config.username {
        return Err(AuthError("Invalid credentials".to_string()));
    }

    // For development, accept "admin123" directly if hash is default
    let valid = if state.config.password_hash.contains("random_salt_here") {
        req.password == "admin123"
    } else {
        verify_password(&req.password, &state.config.password_hash)
    };

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

async fn logout(jar: CookieJar) -> CookieJar {
    jar.remove(Cookie::from("token"))
}
