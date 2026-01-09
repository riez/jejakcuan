//! Broker summary data models

use chrono::NaiveDate;
use rust_decimal::prelude::FromStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Broker transaction summary for a stock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerSummary {
    pub date: NaiveDate,
    pub symbol: String,
    pub broker_code: String,
    pub buy_volume: i64,
    pub sell_volume: i64,
    pub buy_value: Decimal,
    pub sell_value: Decimal,
    pub net_volume: i64,
    pub net_value: Decimal,
}

/// Aggregated broker activity for a stock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerActivity {
    pub symbol: String,
    pub date: NaiveDate,
    pub top_buyers: Vec<BrokerSummary>,
    pub top_sellers: Vec<BrokerSummary>,
    pub foreign_net: Decimal,
    pub domestic_net: Decimal,
}

/// Broker accumulation score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerAccumulationScore {
    pub symbol: String,
    pub score: Decimal,
    pub institutional_buying: bool,
    pub foreign_buying: bool,
    pub concentration_index: Decimal, // HHI
    pub days_accumulated: i32,
}

/// Broker classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokerCategory {
    ForeignInstitutional,
    LocalInstitutional,
    Retail,
    Unknown,
}

impl BrokerCategory {
    /// Weight for scoring (foreign institutional = highest)
    pub fn weight(&self) -> Decimal {
        match self {
            BrokerCategory::ForeignInstitutional => Decimal::from(1),
            BrokerCategory::LocalInstitutional => Decimal::from_str("0.8").unwrap(),
            BrokerCategory::Retail => Decimal::from_str("0.3").unwrap(),
            BrokerCategory::Unknown => Decimal::from_str("0.5").unwrap(),
        }
    }
}
