//! Alert system for JejakCuan
//!
//! Provides various alert types:
//! - Broker accumulation alerts (foreign flow, institutional buying)
//! - Technical indicator alerts (RSI, MACD, Wyckoff, breakouts)
//! - Price alerts
//! - Volume alerts

mod broker_alerts;
mod technical_alerts;

pub use broker_alerts::*;
pub use technical_alerts::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unified alert type encompassing all alert categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "category")]
pub enum Alert {
    Broker(BrokerAlert),
    Technical(TechnicalAlert),
}

impl Alert {
    pub fn id(&self) -> &str {
        match self {
            Alert::Broker(a) => &a.id,
            Alert::Technical(a) => &a.id,
        }
    }

    pub fn symbol(&self) -> &str {
        match self {
            Alert::Broker(a) => &a.symbol,
            Alert::Technical(a) => &a.symbol,
        }
    }

    pub fn priority(&self) -> AlertPriority {
        match self {
            Alert::Broker(a) => a.priority,
            Alert::Technical(a) => a.priority,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Alert::Broker(a) => &a.message,
            Alert::Technical(a) => &a.message,
        }
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        match self {
            Alert::Broker(a) => a.created_at,
            Alert::Technical(a) => a.created_at,
        }
    }
}

/// Alert subscription for user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSubscription {
    pub user_id: String,
    pub symbols: Vec<String>,
    pub alert_types: AlertTypeFilter,
    pub min_priority: AlertPriority,
    pub channels: Vec<NotificationChannel>,
}

/// Filter for alert types user wants to receive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTypeFilter {
    pub broker_alerts: bool,
    pub technical_alerts: bool,
    pub coordinated_buying: bool,
    pub foreign_flow: bool,
    pub wyckoff_events: bool,
    pub rsi_signals: bool,
    pub macd_crossovers: bool,
    pub volume_spikes: bool,
    pub price_breakouts: bool,
}

impl Default for AlertTypeFilter {
    fn default() -> Self {
        Self {
            broker_alerts: true,
            technical_alerts: true,
            coordinated_buying: true,
            foreign_flow: true,
            wyckoff_events: true,
            rsi_signals: true,
            macd_crossovers: true,
            volume_spikes: true,
            price_breakouts: true,
        }
    }
}

/// Notification channels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationChannel {
    Email,
    Telegram,
    WebPush,
    Webhook,
    InApp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_accessors() {
        let broker_alert = BrokerAlert::new(
            "BBCA".to_string(),
            BrokerAlertType::CoordinatedBuying {
                broker_count: 3,
                broker_codes: vec!["BK".into(), "CC".into(), "KZ".into()],
            },
            AlertPriority::High,
            rust_decimal_macros::dec!(3),
            rust_decimal_macros::dec!(3),
        );

        let alert = Alert::Broker(broker_alert);
        assert_eq!(alert.symbol(), "BBCA");
        assert_eq!(alert.priority(), AlertPriority::High);
        assert!(alert.message().contains("BBCA"));
    }
}
