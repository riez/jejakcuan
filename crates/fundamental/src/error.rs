//! Error types for fundamental analysis

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FundamentalError {
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),
}
