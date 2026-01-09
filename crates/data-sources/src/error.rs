//! Data source error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataSourceError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("API error: {0}")]
    ApiError(String),
}
