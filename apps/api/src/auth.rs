//! JWT authentication

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: i64,
}

#[derive(Debug)]
pub struct AuthError(pub String);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": self.0 })),
        )
            .into_response()
    }
}

/// Authenticated user extractor
#[allow(dead_code)]
pub struct AuthUser {
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Try to get token from cookie first, then Authorization header
        let jar = parts
            .extract::<CookieJar>()
            .await
            .map_err(|_| AuthError("Failed to extract cookies".to_string()))?;

        let token = jar
            .get("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                parts
                    .headers
                    .get("Authorization")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.strip_prefix("Bearer "))
                    .map(String::from)
            })
            .ok_or_else(|| AuthError("No token provided".to_string()))?;

        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "development_secret_change_in_production".to_string());

        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AuthError(format!("Invalid token: {}", e)))?;

        Ok(AuthUser {
            username: token_data.claims.sub,
        })
    }
}

/// Create JWT token
pub fn create_token(username: &str, secret: &str) -> Result<LoginResponse, AuthError> {
    let now = Utc::now();
    let exp = now + Duration::hours(24);

    let claims = Claims {
        sub: username.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthError(format!("Failed to create token: {}", e)))?;

    Ok(LoginResponse {
        token,
        expires_at: exp.timestamp(),
    })
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Hash a password (for generating initial password hash)
#[allow(dead_code)]
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AuthError(format!("Failed to hash password: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_token_success() {
        let secret = "test_secret_123";
        let username = "testuser";

        let result = create_token(username, secret);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert!(response.expires_at > Utc::now().timestamp());
    }

    #[test]
    fn test_token_contains_valid_claims() {
        let secret = "test_secret_456";
        let username = "admin";

        let response = create_token(username, secret).unwrap();

        // Decode the token to verify claims
        let token_data = decode::<Claims>(
            &response.token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .unwrap();

        assert_eq!(token_data.claims.sub, username);
        assert!(token_data.claims.exp > token_data.claims.iat);
    }

    #[test]
    fn test_hash_and_verify_password() {
        let password = "secure_password_123";
        let hash = hash_password(password).unwrap();

        // Verify the password matches the hash
        assert!(verify_password(password, &hash));

        // Verify wrong password doesn't match
        assert!(!verify_password("wrong_password", &hash));
    }

    #[test]
    fn test_verify_password_with_invalid_hash() {
        // Invalid hash format should return false, not panic
        assert!(!verify_password("password", "invalid_hash"));
        assert!(!verify_password("password", ""));
    }

    #[test]
    fn test_auth_error_display() {
        let error = AuthError("test error message".to_string());
        assert_eq!(error.0, "test error message");
    }
}
