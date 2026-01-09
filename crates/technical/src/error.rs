//! Error types for technical indicators

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TechnicalError {
    #[error("Insufficient data: need {required} points, got {actual}")]
    InsufficientData { required: usize, actual: usize },

    #[error("Invalid period: {0}")]
    InvalidPeriod(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),
}
