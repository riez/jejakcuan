//! Database models (row types)

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StockRow {
    pub id: i32,
    pub symbol: String,
    pub name: String,
    pub sector: Option<String>,
    pub subsector: Option<String>,
    pub listing_date: Option<NaiveDate>,
    pub market_cap: Option<i64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StockPriceRow {
    pub time: DateTime<Utc>,
    pub symbol: String,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
    pub value: Option<Decimal>,
    pub frequency: Option<i64>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BrokerSummaryRow {
    pub time: DateTime<Utc>,
    pub symbol: String,
    pub broker_code: String,
    pub buy_volume: i64,
    pub sell_volume: i64,
    pub buy_value: Decimal,
    pub sell_value: Decimal,
    pub net_volume: i64,
    pub net_value: Decimal,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BrokerRow {
    pub code: String,
    pub name: String,
    pub category: String,
    pub weight: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FinancialsRow {
    pub id: i32,
    pub symbol: String,
    pub period_end: NaiveDate,
    pub revenue: Option<Decimal>,
    pub net_income: Option<Decimal>,
    pub total_assets: Option<Decimal>,
    pub total_equity: Option<Decimal>,
    pub total_debt: Option<Decimal>,
    pub ebitda: Option<Decimal>,
    pub free_cash_flow: Option<Decimal>,
    pub eps: Option<Decimal>,
    pub book_value_per_share: Option<Decimal>,
    pub pe_ratio: Option<Decimal>,
    pub pb_ratio: Option<Decimal>,
    pub ev_ebitda: Option<Decimal>,
    pub roe: Option<Decimal>,
    pub roa: Option<Decimal>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StockScoreRow {
    pub time: DateTime<Utc>,
    pub symbol: String,
    pub composite_score: Decimal,
    pub technical_score: Decimal,
    pub fundamental_score: Decimal,
    pub sentiment_score: Decimal,
    pub ml_score: Decimal,
    pub technical_breakdown: Option<serde_json::Value>,
    pub fundamental_breakdown: Option<serde_json::Value>,
    pub sentiment_breakdown: Option<serde_json::Value>,
    pub ml_breakdown: Option<serde_json::Value>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WatchlistRow {
    pub id: i32,
    pub symbol: String,
    pub sort_order: i32,
    pub notes: Option<String>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SettingsRow {
    pub id: i32,
    pub score_weights: serde_json::Value,
    pub api_keys: serde_json::Value,
    pub preferences: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}
