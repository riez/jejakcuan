use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents a stock on the Indonesian Stock Exchange (IDX)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub subsector: Option<String>,
}

/// OHLCV price data for a stock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
}

/// Composite score for a stock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockScore {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub composite_score: f64,
    pub technical_score: f64,
    pub fundamental_score: f64,
    pub sentiment_score: f64,
    pub ml_score: f64,
}
