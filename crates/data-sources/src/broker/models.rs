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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broker_category_weights() {
        // Foreign institutional should have highest weight
        assert_eq!(
            BrokerCategory::ForeignInstitutional.weight(),
            Decimal::from(1)
        );
        // Local institutional should be 0.8
        assert_eq!(
            BrokerCategory::LocalInstitutional.weight(),
            Decimal::from_str("0.8").unwrap()
        );
        // Retail should be 0.3
        assert_eq!(
            BrokerCategory::Retail.weight(),
            Decimal::from_str("0.3").unwrap()
        );
        // Unknown should be 0.5
        assert_eq!(
            BrokerCategory::Unknown.weight(),
            Decimal::from_str("0.5").unwrap()
        );
    }

    #[test]
    fn test_broker_category_weight_ordering() {
        // Ensure weight ordering makes sense: Foreign > Local > Unknown > Retail
        assert!(
            BrokerCategory::ForeignInstitutional.weight()
                > BrokerCategory::LocalInstitutional.weight()
        );
        assert!(BrokerCategory::LocalInstitutional.weight() > BrokerCategory::Unknown.weight());
        assert!(BrokerCategory::Unknown.weight() > BrokerCategory::Retail.weight());
    }

    #[test]
    fn test_broker_summary_net_calculation() {
        let summary = BrokerSummary {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            symbol: "BBCA".to_string(),
            broker_code: "BK".to_string(),
            buy_volume: 1_000_000,
            sell_volume: 500_000,
            buy_value: Decimal::from(100_000_000i64),
            sell_value: Decimal::from(50_000_000i64),
            net_volume: 500_000,
            net_value: Decimal::from(50_000_000i64),
        };

        assert_eq!(summary.net_volume, summary.buy_volume - summary.sell_volume);
        assert_eq!(summary.net_value, summary.buy_value - summary.sell_value);
    }

    #[test]
    fn test_broker_summary_negative_net() {
        let summary = BrokerSummary {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            symbol: "BBRI".to_string(),
            broker_code: "CC".to_string(),
            buy_volume: 300_000,
            sell_volume: 800_000,
            buy_value: Decimal::from(30_000_000i64),
            sell_value: Decimal::from(80_000_000i64),
            net_volume: -500_000,
            net_value: Decimal::from(-50_000_000i64),
        };

        assert!(summary.net_volume < 0);
        assert!(summary.net_value < Decimal::ZERO);
    }

    #[test]
    fn test_broker_activity_structure() {
        let activity = BrokerActivity {
            symbol: "BBCA".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            top_buyers: vec![],
            top_sellers: vec![],
            foreign_net: Decimal::from(100_000_000i64),
            domestic_net: Decimal::from(-50_000_000i64),
        };

        // Foreign buying, domestic selling
        assert!(activity.foreign_net > Decimal::ZERO);
        assert!(activity.domestic_net < Decimal::ZERO);
    }

    #[test]
    fn test_broker_accumulation_score() {
        let score = BrokerAccumulationScore {
            symbol: "TLKM".to_string(),
            score: Decimal::from(75),
            institutional_buying: true,
            foreign_buying: true,
            concentration_index: Decimal::from(1500),
            days_accumulated: 5,
        };

        assert!(score.institutional_buying);
        assert!(score.foreign_buying);
        assert!(score.score > Decimal::from(50)); // Bullish
        assert!(score.days_accumulated > 0);
    }

    #[test]
    fn test_broker_category_equality() {
        assert_eq!(
            BrokerCategory::ForeignInstitutional,
            BrokerCategory::ForeignInstitutional
        );
        assert_ne!(
            BrokerCategory::ForeignInstitutional,
            BrokerCategory::LocalInstitutional
        );
        assert_ne!(BrokerCategory::Retail, BrokerCategory::Unknown);
    }
}
