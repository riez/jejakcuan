//! Technical analysis error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TechnicalError {
    #[error("Insufficient data: required {required}, got {actual}")]
    InsufficientData { required: usize, actual: usize },

    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}
