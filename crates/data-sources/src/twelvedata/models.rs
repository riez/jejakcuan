//! Data models for TwelveData API responses

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Real-time price update from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub event: String,
    pub symbol: String,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub exchange: Option<String>,
    #[serde(default)]
    pub mic_code: Option<String>,
    #[serde(rename = "type", default)]
    pub instrument_type: Option<String>,
    #[serde(default)]
    pub price: Option<Decimal>,
    #[serde(default)]
    pub bid: Option<Decimal>,
    #[serde(default)]
    pub ask: Option<Decimal>,
    #[serde(default)]
    pub day_volume: Option<i64>,
    #[serde(default)]
    pub timestamp: Option<i64>,
}

impl PriceUpdate {
    pub fn datetime(&self) -> Option<DateTime<Utc>> {
        self.timestamp.and_then(|ts| {
            DateTime::from_timestamp(ts, 0)
        })
    }
}

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub datetime: String,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    #[serde(default)]
    pub volume: Option<i64>,
}

/// Time series metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesMeta {
    pub symbol: String,
    pub interval: String,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub exchange_timezone: Option<String>,
    #[serde(default)]
    pub exchange: Option<String>,
    #[serde(default)]
    pub mic_code: Option<String>,
    #[serde(rename = "type", default)]
    pub instrument_type: Option<String>,
}

/// Time series response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesResponse {
    pub meta: TimeSeriesMeta,
    pub values: Vec<TimeSeriesPoint>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Latest price response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestPrice {
    pub price: Decimal,
}

/// Quote response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub symbol: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub exchange: Option<String>,
    #[serde(default)]
    pub mic_code: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub datetime: Option<String>,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub open: Option<Decimal>,
    #[serde(default)]
    pub high: Option<Decimal>,
    #[serde(default)]
    pub low: Option<Decimal>,
    #[serde(default)]
    pub close: Option<Decimal>,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub previous_close: Option<Decimal>,
    #[serde(default)]
    pub change: Option<Decimal>,
    #[serde(default)]
    pub percent_change: Option<Decimal>,
    #[serde(default)]
    pub fifty_two_week_high: Option<Decimal>,
    #[serde(default)]
    pub fifty_two_week_low: Option<Decimal>,
}

/// Market mover entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketMover {
    pub symbol: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub exchange: Option<String>,
    #[serde(default)]
    pub close: Option<Decimal>,
    #[serde(default)]
    pub change: Option<Decimal>,
    #[serde(default)]
    pub percent_change: Option<Decimal>,
    #[serde(default)]
    pub volume: Option<i64>,
}

/// Market movers response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketMoversResponse {
    #[serde(default)]
    pub gainers: Vec<MarketMover>,
    #[serde(default)]
    pub losers: Vec<MarketMover>,
}

/// Stock symbol info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub symbol: String,
    pub name: String,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub exchange: Option<String>,
    #[serde(default)]
    pub mic_code: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(rename = "type", default)]
    pub instrument_type: Option<String>,
}

/// Exchange info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub name: String,
    pub code: String,
    pub country: String,
    #[serde(default)]
    pub timezone: Option<String>,
}

/// Time interval for data requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval {
    Min1,
    Min5,
    Min15,
    Min30,
    Min45,
    Hour1,
    Hour2,
    Hour4,
    Day1,
    Week1,
    Month1,
}

impl Interval {
    pub fn as_str(&self) -> &'static str {
        match self {
            Interval::Min1 => "1min",
            Interval::Min5 => "5min",
            Interval::Min15 => "15min",
            Interval::Min30 => "30min",
            Interval::Min45 => "45min",
            Interval::Hour1 => "1h",
            Interval::Hour2 => "2h",
            Interval::Hour4 => "4h",
            Interval::Day1 => "1day",
            Interval::Week1 => "1week",
            Interval::Month1 => "1month",
        }
    }
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// WebSocket subscription action
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeAction {
    pub action: String,
    pub params: SubscribeParams,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscribeParams {
    pub symbols: Vec<String>,
}

impl SubscribeAction {
    pub fn subscribe(symbols: Vec<String>) -> Self {
        Self {
            action: "subscribe".to_string(),
            params: SubscribeParams { symbols },
        }
    }

    pub fn unsubscribe(symbols: Vec<String>) -> Self {
        Self {
            action: "unsubscribe".to_string(),
            params: SubscribeParams { symbols },
        }
    }
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum WebSocketMessage {
    #[serde(rename = "price")]
    Price(PriceUpdate),
    #[serde(rename = "subscribe-status")]
    SubscribeStatus {
        status: String,
        #[serde(default)]
        success: Vec<WebSocketSymbolStatus>,
        #[serde(default)]
        fails: Vec<WebSocketSymbolStatus>,
    },
    #[serde(rename = "unsubscribe-status")]
    UnsubscribeStatus { status: String },
    #[serde(rename = "heartbeat")]
    Heartbeat,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketSymbolStatus {
    pub symbol: String,
    #[serde(default)]
    pub exchange: Option<String>,
    #[serde(default)]
    pub mic_code: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(rename = "type", default)]
    pub instrument_type: Option<String>,
}
