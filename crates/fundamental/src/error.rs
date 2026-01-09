//! Error types for fundamental analysis

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FundamentalError {
    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("No peers found for sector: {0}")]
    NoPeersFound(String),
}
