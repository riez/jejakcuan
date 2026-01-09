//! Yahoo Finance response models

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Stock quote from Yahoo Finance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YahooQuote {
    pub symbol: String,
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub regular_market_price: Option<f64>,
    pub regular_market_change: Option<f64>,
    pub regular_market_change_percent: Option<f64>,
    pub regular_market_volume: Option<i64>,
    pub regular_market_open: Option<f64>,
    pub regular_market_high: Option<f64>,
    pub regular_market_low: Option<f64>,
    pub regular_market_previous_close: Option<f64>,
    pub market_cap: Option<i64>,
    pub trailing_pe: Option<f64>,
    pub price_to_book: Option<f64>,
    pub fifty_two_week_high: Option<f64>,
    pub fifty_two_week_low: Option<f64>,
}

/// Historical price data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YahooOHLCV {
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
    pub adj_close: Option<Decimal>,
}

/// Stock info with fundamentals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YahooStockInfo {
    pub symbol: String,
    pub name: String,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub market_cap: Option<i64>,
    pub pe_ratio: Option<f64>,
    pub pb_ratio: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub eps: Option<f64>,
    pub revenue: Option<i64>,
    pub profit_margin: Option<f64>,
}

// Internal response structures for parsing
#[derive(Debug, Deserialize)]
pub(crate) struct QuoteResponse {
    #[serde(rename = "quoteResponse")]
    pub quote_response: QuoteResponseInner,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QuoteResponseInner {
    pub result: Vec<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChartResponse {
    pub chart: ChartResult,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChartResult {
    pub result: Option<Vec<ChartData>>,
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChartData {
    pub timestamp: Vec<i64>,
    pub indicators: Indicators,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Indicators {
    pub quote: Vec<QuoteIndicator>,
    #[serde(rename = "adjclose")]
    pub adj_close: Option<Vec<AdjCloseIndicator>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QuoteIndicator {
    pub open: Vec<Option<f64>>,
    pub high: Vec<Option<f64>>,
    pub low: Vec<Option<f64>>,
    pub close: Vec<Option<f64>>,
    pub volume: Vec<Option<i64>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AdjCloseIndicator {
    #[serde(rename = "adjclose")]
    pub adj_close: Vec<Option<f64>>,
}
