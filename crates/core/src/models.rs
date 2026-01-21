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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_creation() {
        let stock = Stock {
            symbol: "BBCA".to_string(),
            name: "Bank Central Asia Tbk".to_string(),
            sector: "Financials".to_string(),
            subsector: Some("Banks".to_string()),
        };

        assert_eq!(stock.symbol, "BBCA");
        assert_eq!(stock.sector, "Financials");
        assert!(stock.subsector.is_some());
    }

    #[test]
    fn test_stock_without_subsector() {
        let stock = Stock {
            symbol: "TLKM".to_string(),
            name: "Telkom Indonesia".to_string(),
            sector: "Communication Services".to_string(),
            subsector: None,
        };

        assert_eq!(stock.symbol, "TLKM");
        assert!(stock.subsector.is_none());
    }

    #[test]
    fn test_price_data_creation() {
        let price = PriceData {
            symbol: "BBRI".to_string(),
            timestamp: Utc::now(),
            open: Decimal::from(5000),
            high: Decimal::from(5200),
            low: Decimal::from(4900),
            close: Decimal::from(5100),
            volume: 10_000_000,
        };

        assert_eq!(price.symbol, "BBRI");
        assert!(price.high >= price.low);
        assert!(price.close >= price.low && price.close <= price.high);
        assert!(price.volume > 0);
    }

    #[test]
    fn test_price_data_ohlc_relationship() {
        let price = PriceData {
            symbol: "BMRI".to_string(),
            timestamp: Utc::now(),
            open: Decimal::from(6000),
            high: Decimal::from(6500),
            low: Decimal::from(5800),
            close: Decimal::from(6300),
            volume: 5_000_000,
        };

        // High should be >= all other prices
        assert!(price.high >= price.open);
        assert!(price.high >= price.close);
        assert!(price.high >= price.low);

        // Low should be <= all other prices
        assert!(price.low <= price.open);
        assert!(price.low <= price.close);
        assert!(price.low <= price.high);
    }

    #[test]
    fn test_stock_score_creation() {
        let score = StockScore {
            symbol: "ASII".to_string(),
            timestamp: Utc::now(),
            composite_score: 75.5,
            technical_score: 80.0,
            fundamental_score: 70.0,
            sentiment_score: 65.0,
            ml_score: 72.0,
        };

        assert_eq!(score.symbol, "ASII");
        assert!(score.composite_score >= 0.0 && score.composite_score <= 100.0);
        assert!(score.technical_score >= 0.0 && score.technical_score <= 100.0);
        assert!(score.fundamental_score >= 0.0 && score.fundamental_score <= 100.0);
    }

    #[test]
    fn test_stock_score_component_bounds() {
        // All scores should be valid (0-100 range)
        let score = StockScore {
            symbol: "UNVR".to_string(),
            timestamp: Utc::now(),
            composite_score: 50.0,
            technical_score: 0.0,     // Minimum
            fundamental_score: 100.0, // Maximum
            sentiment_score: 50.0,    // Middle
            ml_score: 75.0,
        };

        assert!(score.technical_score >= 0.0);
        assert!(score.fundamental_score <= 100.0);
    }

    #[test]
    fn test_stock_serialization() {
        let stock = Stock {
            symbol: "GOTO".to_string(),
            name: "GoTo Gojek Tokopedia".to_string(),
            sector: "Consumer Discretionary".to_string(),
            subsector: Some("E-Commerce".to_string()),
        };

        // Test that serialization works
        let json = serde_json::to_string(&stock).unwrap();
        assert!(json.contains("GOTO"));
        assert!(json.contains("E-Commerce"));

        // Test deserialization
        let deserialized: Stock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, stock.symbol);
        assert_eq!(deserialized.subsector, stock.subsector);
    }

    #[test]
    fn test_price_data_serialization() {
        let price = PriceData {
            symbol: "BRIS".to_string(),
            timestamp: Utc::now(),
            open: Decimal::from(2500),
            high: Decimal::from(2600),
            low: Decimal::from(2450),
            close: Decimal::from(2550),
            volume: 2_000_000,
        };

        let json = serde_json::to_string(&price).unwrap();
        assert!(json.contains("BRIS"));

        let deserialized: PriceData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, price.symbol);
        assert_eq!(deserialized.volume, price.volume);
    }

    #[test]
    fn test_stock_score_serialization() {
        let score = StockScore {
            symbol: "ANTM".to_string(),
            timestamp: Utc::now(),
            composite_score: 68.5,
            technical_score: 72.0,
            fundamental_score: 65.0,
            sentiment_score: 70.0,
            ml_score: 67.0,
        };

        let json = serde_json::to_string(&score).unwrap();
        assert!(json.contains("ANTM"));
        assert!(json.contains("68.5"));

        let deserialized: StockScore = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, score.symbol);
        assert!((deserialized.composite_score - 68.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_stock_clone() {
        let original = Stock {
            symbol: "INDF".to_string(),
            name: "Indofood".to_string(),
            sector: "Consumer Staples".to_string(),
            subsector: Some("Food".to_string()),
        };

        let cloned = original.clone();
        assert_eq!(cloned.symbol, original.symbol);
        assert_eq!(cloned.name, original.name);
    }

    #[test]
    fn test_price_data_clone() {
        let original = PriceData {
            symbol: "ICBP".to_string(),
            timestamp: Utc::now(),
            open: Decimal::from(10000),
            high: Decimal::from(10200),
            low: Decimal::from(9900),
            close: Decimal::from(10100),
            volume: 3_000_000,
        };

        let cloned = original.clone();
        assert_eq!(cloned.symbol, original.symbol);
        assert_eq!(cloned.close, original.close);
    }
}
