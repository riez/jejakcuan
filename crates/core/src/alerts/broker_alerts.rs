//! Broker accumulation and institutional flow alerts
//!
//! Triggers alerts when:
//! - 3+ institutional codes show coordinated buying
//! - Foreign institutional flow exceeds thresholds
//! - Accumulation score crosses configurable levels

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Alert priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl AlertPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertPriority::Critical => "critical",
            AlertPriority::High => "high",
            AlertPriority::Medium => "medium",
            AlertPriority::Low => "low",
        }
    }
}

/// Alert types for broker flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrokerAlertType {
    CoordinatedBuying {
        broker_count: usize,
        broker_codes: Vec<String>,
    },
    ForeignInflow {
        net_value: Decimal,
        threshold: Decimal,
    },
    ForeignOutflow {
        net_value: Decimal,
        threshold: Decimal,
    },
    InstitutionalAccumulation {
        score: Decimal,
        days_accumulated: i32,
    },
    InstitutionalDistribution {
        score: Decimal,
        days_distributed: i32,
    },
    HighConcentration {
        hhi: Decimal,
        top_broker: String,
    },
}

/// Broker flow alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerAlert {
    pub id: String,
    pub symbol: String,
    pub alert_type: BrokerAlertType,
    pub priority: AlertPriority,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub triggered_value: Decimal,
    pub threshold_value: Decimal,
}

impl BrokerAlert {
    /// Create new broker alert
    pub fn new(
        symbol: String,
        alert_type: BrokerAlertType,
        priority: AlertPriority,
        triggered_value: Decimal,
        threshold_value: Decimal,
    ) -> Self {
        let id = format!("broker_{}_{}", symbol, Utc::now().timestamp_millis());
        let message = generate_alert_message(&symbol, &alert_type);

        Self {
            id,
            symbol,
            alert_type,
            priority,
            message,
            created_at: Utc::now(),
            triggered_value,
            threshold_value,
        }
    }
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerAlertConfig {
    /// Minimum institutional brokers for coordinated buying alert
    pub coordinated_broker_threshold: usize,
    /// Foreign inflow threshold (in IDR)
    pub foreign_inflow_threshold: Decimal,
    /// Foreign outflow threshold (in IDR)
    pub foreign_outflow_threshold: Decimal,
    /// Accumulation score threshold (0-100)
    pub accumulation_score_threshold: Decimal,
    /// Distribution score threshold (0-100)
    pub distribution_score_threshold: Decimal,
    /// HHI concentration threshold
    pub hhi_threshold: Decimal,
    /// Minimum days for accumulation alert
    pub min_accumulation_days: i32,
}

impl Default for BrokerAlertConfig {
    fn default() -> Self {
        Self {
            coordinated_broker_threshold: 3,
            foreign_inflow_threshold: dec!(10_000_000_000), // 10B IDR
            foreign_outflow_threshold: dec!(-5_000_000_000), // -5B IDR
            accumulation_score_threshold: dec!(75),
            distribution_score_threshold: dec!(25),
            hhi_threshold: dec!(0.20),
            min_accumulation_days: 3,
        }
    }
}

/// Alert input data from broker analysis
#[derive(Debug, Clone)]
pub struct BrokerAlertInput {
    pub symbol: String,
    pub institutional_net: Decimal,
    pub foreign_net: Decimal,
    pub accumulation_score: Decimal,
    pub days_accumulated: i32,
    pub coordinated_buying: bool,
    pub institutional_buyer_codes: Vec<String>,
    pub hhi: Decimal,
    pub top_broker_code: Option<String>,
}

/// Broker alert engine
pub struct BrokerAlertEngine {
    config: BrokerAlertConfig,
}

impl BrokerAlertEngine {
    /// Create new alert engine with default config
    pub fn new() -> Self {
        Self {
            config: BrokerAlertConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: BrokerAlertConfig) -> Self {
        Self { config }
    }

    /// Evaluate broker data and generate alerts
    pub fn evaluate(&self, input: &BrokerAlertInput) -> Vec<BrokerAlert> {
        let mut alerts = Vec::new();

        // Check coordinated buying
        if input.coordinated_buying
            && input.institutional_buyer_codes.len() >= self.config.coordinated_broker_threshold
        {
            alerts.push(BrokerAlert::new(
                input.symbol.clone(),
                BrokerAlertType::CoordinatedBuying {
                    broker_count: input.institutional_buyer_codes.len(),
                    broker_codes: input.institutional_buyer_codes.clone(),
                },
                AlertPriority::High,
                Decimal::from(input.institutional_buyer_codes.len() as i32),
                Decimal::from(self.config.coordinated_broker_threshold as i32),
            ));
        }

        // Check foreign inflow
        if input.foreign_net >= self.config.foreign_inflow_threshold {
            alerts.push(BrokerAlert::new(
                input.symbol.clone(),
                BrokerAlertType::ForeignInflow {
                    net_value: input.foreign_net,
                    threshold: self.config.foreign_inflow_threshold,
                },
                AlertPriority::High,
                input.foreign_net,
                self.config.foreign_inflow_threshold,
            ));
        }

        // Check foreign outflow
        if input.foreign_net <= self.config.foreign_outflow_threshold {
            alerts.push(BrokerAlert::new(
                input.symbol.clone(),
                BrokerAlertType::ForeignOutflow {
                    net_value: input.foreign_net,
                    threshold: self.config.foreign_outflow_threshold,
                },
                AlertPriority::Medium,
                input.foreign_net,
                self.config.foreign_outflow_threshold,
            ));
        }

        // Check accumulation
        if input.accumulation_score >= self.config.accumulation_score_threshold
            && input.days_accumulated >= self.config.min_accumulation_days
        {
            alerts.push(BrokerAlert::new(
                input.symbol.clone(),
                BrokerAlertType::InstitutionalAccumulation {
                    score: input.accumulation_score,
                    days_accumulated: input.days_accumulated,
                },
                AlertPriority::High,
                input.accumulation_score,
                self.config.accumulation_score_threshold,
            ));
        }

        // Check distribution
        if input.accumulation_score <= self.config.distribution_score_threshold {
            alerts.push(BrokerAlert::new(
                input.symbol.clone(),
                BrokerAlertType::InstitutionalDistribution {
                    score: input.accumulation_score,
                    days_distributed: input.days_accumulated.abs(),
                },
                AlertPriority::Medium,
                input.accumulation_score,
                self.config.distribution_score_threshold,
            ));
        }

        // Check high concentration
        if input.hhi >= self.config.hhi_threshold {
            if let Some(ref top_broker) = input.top_broker_code {
                alerts.push(BrokerAlert::new(
                    input.symbol.clone(),
                    BrokerAlertType::HighConcentration {
                        hhi: input.hhi,
                        top_broker: top_broker.clone(),
                    },
                    AlertPriority::Medium,
                    input.hhi,
                    self.config.hhi_threshold,
                ));
            }
        }

        alerts
    }

    /// Get current configuration
    pub fn config(&self) -> &BrokerAlertConfig {
        &self.config
    }
}

impl Default for BrokerAlertEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate human-readable alert message
fn generate_alert_message(symbol: &str, alert_type: &BrokerAlertType) -> String {
    match alert_type {
        BrokerAlertType::CoordinatedBuying {
            broker_count,
            broker_codes,
        } => {
            format!(
                "{}: Coordinated buying detected - {} institutional brokers ({}) accumulating",
                symbol,
                broker_count,
                broker_codes.join(", ")
            )
        }
        BrokerAlertType::ForeignInflow { net_value, .. } => {
            format!(
                "{}: Significant foreign inflow of Rp{:.0}",
                symbol, net_value
            )
        }
        BrokerAlertType::ForeignOutflow { net_value, .. } => {
            format!("{}: Foreign outflow detected of Rp{:.0}", symbol, net_value)
        }
        BrokerAlertType::InstitutionalAccumulation {
            score,
            days_accumulated,
        } => {
            format!(
                "{}: Institutional accumulation score {} for {} consecutive days",
                symbol, score, days_accumulated
            )
        }
        BrokerAlertType::InstitutionalDistribution {
            score,
            days_distributed,
        } => {
            format!(
                "{}: Institutional distribution detected - score {} for {} days",
                symbol, score, days_distributed
            )
        }
        BrokerAlertType::HighConcentration { hhi, top_broker } => {
            format!(
                "{}: High broker concentration (HHI: {:.2}) - {} is dominant",
                symbol, hhi, top_broker
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(
        symbol: &str,
        accumulation_score: Decimal,
        days_accumulated: i32,
        coordinated: bool,
    ) -> BrokerAlertInput {
        BrokerAlertInput {
            symbol: symbol.to_string(),
            institutional_net: Decimal::from(1_000_000_000i64),
            foreign_net: Decimal::from(5_000_000_000i64),
            accumulation_score,
            days_accumulated,
            coordinated_buying: coordinated,
            institutional_buyer_codes: if coordinated {
                vec!["BK".into(), "KZ".into(), "CC".into(), "SQ".into()]
            } else {
                vec![]
            },
            hhi: dec!(0.15),
            top_broker_code: Some("BK".into()),
        }
    }

    #[test]
    fn test_no_alerts_normal_activity() {
        let engine = BrokerAlertEngine::new();
        let input = make_input("BBCA", dec!(50), 1, false);

        let alerts = engine.evaluate(&input);

        assert!(alerts.is_empty());
    }

    #[test]
    fn test_coordinated_buying_alert() {
        let engine = BrokerAlertEngine::new();
        let input = make_input("BBCA", dec!(70), 3, true);

        let alerts = engine.evaluate(&input);

        let coord_alert = alerts
            .iter()
            .find(|a| matches!(a.alert_type, BrokerAlertType::CoordinatedBuying { .. }));

        assert!(coord_alert.is_some());
        assert_eq!(coord_alert.unwrap().priority, AlertPriority::High);
    }

    #[test]
    fn test_accumulation_alert() {
        let engine = BrokerAlertEngine::new();
        let input = make_input("BBCA", dec!(80), 5, false);

        let alerts = engine.evaluate(&input);

        let accum_alert = alerts.iter().find(|a| {
            matches!(
                a.alert_type,
                BrokerAlertType::InstitutionalAccumulation { .. }
            )
        });

        assert!(accum_alert.is_some());
    }

    #[test]
    fn test_distribution_alert() {
        let engine = BrokerAlertEngine::new();
        let mut input = make_input("BBCA", dec!(20), -3, false);
        input.foreign_net = Decimal::ZERO; // Reset to avoid foreign alerts

        let alerts = engine.evaluate(&input);

        let dist_alert = alerts.iter().find(|a| {
            matches!(
                a.alert_type,
                BrokerAlertType::InstitutionalDistribution { .. }
            )
        });

        assert!(dist_alert.is_some());
    }

    #[test]
    fn test_foreign_inflow_alert() {
        let engine = BrokerAlertEngine::new();
        let mut input = make_input("BBCA", dec!(50), 1, false);
        input.foreign_net = dec!(15_000_000_000); // 15B

        let alerts = engine.evaluate(&input);

        let inflow_alert = alerts
            .iter()
            .find(|a| matches!(a.alert_type, BrokerAlertType::ForeignInflow { .. }));

        assert!(inflow_alert.is_some());
        assert_eq!(inflow_alert.unwrap().priority, AlertPriority::High);
    }

    #[test]
    fn test_foreign_outflow_alert() {
        let engine = BrokerAlertEngine::new();
        let mut input = make_input("BBCA", dec!(50), 1, false);
        input.foreign_net = dec!(-8_000_000_000); // -8B

        let alerts = engine.evaluate(&input);

        let outflow_alert = alerts
            .iter()
            .find(|a| matches!(a.alert_type, BrokerAlertType::ForeignOutflow { .. }));

        assert!(outflow_alert.is_some());
    }

    #[test]
    fn test_high_concentration_alert() {
        let engine = BrokerAlertEngine::new();
        let mut input = make_input("BBCA", dec!(50), 1, false);
        input.hhi = dec!(0.25);

        let alerts = engine.evaluate(&input);

        let conc_alert = alerts
            .iter()
            .find(|a| matches!(a.alert_type, BrokerAlertType::HighConcentration { .. }));

        assert!(conc_alert.is_some());
    }

    #[test]
    fn test_custom_config() {
        let config = BrokerAlertConfig {
            accumulation_score_threshold: dec!(60), // Lower threshold
            min_accumulation_days: 2,
            ..Default::default()
        };

        let engine = BrokerAlertEngine::with_config(config);
        let input = make_input("BBCA", dec!(65), 2, false);

        let alerts = engine.evaluate(&input);

        let accum_alert = alerts.iter().find(|a| {
            matches!(
                a.alert_type,
                BrokerAlertType::InstitutionalAccumulation { .. }
            )
        });

        assert!(accum_alert.is_some());
    }

    #[test]
    fn test_alert_message_generation() {
        let alert_type = BrokerAlertType::CoordinatedBuying {
            broker_count: 4,
            broker_codes: vec!["BK".into(), "KZ".into(), "CC".into(), "SQ".into()],
        };

        let message = generate_alert_message("BBCA", &alert_type);

        assert!(message.contains("BBCA"));
        assert!(message.contains("4"));
        assert!(message.contains("BK, KZ, CC, SQ"));
    }

    #[test]
    fn test_alert_priority() {
        assert_eq!(AlertPriority::Critical.as_str(), "critical");
        assert_eq!(AlertPriority::High.as_str(), "high");
        assert_eq!(AlertPriority::Medium.as_str(), "medium");
        assert_eq!(AlertPriority::Low.as_str(), "low");
    }
}
