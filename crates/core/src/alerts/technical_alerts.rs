//! Technical indicator alerts
//!
//! Triggers alerts for:
//! - RSI overbought/oversold
//! - MACD crossovers
//! - Wyckoff phase transitions
//! - Volume spikes
//! - Price breakouts

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use super::AlertPriority;

/// Technical alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechnicalAlertType {
    RsiOverbought { rsi: Decimal },
    RsiOversold { rsi: Decimal },
    MacdBullishCrossover { macd: Decimal, signal: Decimal },
    MacdBearishCrossover { macd: Decimal, signal: Decimal },
    WyckoffAccumulation { confidence: u8 },
    WyckoffDistribution { confidence: u8 },
    WyckoffSpring { price: Decimal },
    WyckoffUpthrust { price: Decimal },
    VolumeSpike { rvol: Decimal },
    PriceBreakout { price: Decimal, resistance: Decimal },
    PriceBreakdown { price: Decimal, support: Decimal },
    GoldenCross { ema_short: Decimal, ema_long: Decimal },
    DeathCross { ema_short: Decimal, ema_long: Decimal },
    BollingerSqueeze { bandwidth: Decimal },
}

/// Technical alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalAlert {
    pub id: String,
    pub symbol: String,
    pub alert_type: TechnicalAlertType,
    pub priority: AlertPriority,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

impl TechnicalAlert {
    pub fn new(symbol: String, alert_type: TechnicalAlertType, priority: AlertPriority) -> Self {
        let id = format!("tech_{}_{}", symbol, Utc::now().timestamp_millis());
        let message = generate_tech_message(&symbol, &alert_type);
        Self {
            id,
            symbol,
            alert_type,
            priority,
            message,
            created_at: Utc::now(),
        }
    }
}

/// Technical alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalAlertConfig {
    pub rsi_overbought: Decimal,
    pub rsi_oversold: Decimal,
    pub rvol_spike_threshold: Decimal,
    pub wyckoff_min_confidence: u8,
    pub bollinger_squeeze_threshold: Decimal,
}

impl Default for TechnicalAlertConfig {
    fn default() -> Self {
        Self {
            rsi_overbought: dec!(70),
            rsi_oversold: dec!(30),
            rvol_spike_threshold: dec!(2.5),
            wyckoff_min_confidence: 70,
            bollinger_squeeze_threshold: dec!(0.05),
        }
    }
}

/// Input for technical alert evaluation
#[derive(Debug, Clone, Default)]
pub struct TechnicalAlertInput {
    pub symbol: String,
    pub current_price: Decimal,
    pub rsi: Option<Decimal>,
    pub macd: Option<Decimal>,
    pub macd_signal: Option<Decimal>,
    pub prev_macd: Option<Decimal>,
    pub prev_macd_signal: Option<Decimal>,
    pub rvol: Option<Decimal>,
    pub ema20: Option<Decimal>,
    pub ema50: Option<Decimal>,
    pub prev_ema20: Option<Decimal>,
    pub prev_ema50: Option<Decimal>,
    pub support: Option<Decimal>,
    pub resistance: Option<Decimal>,
    pub wyckoff_phase: Option<String>,
    pub wyckoff_confidence: Option<u8>,
    pub wyckoff_event: Option<String>,
    pub bollinger_bandwidth: Option<Decimal>,
}

/// Technical alert engine
pub struct TechnicalAlertEngine {
    config: TechnicalAlertConfig,
}

impl TechnicalAlertEngine {
    pub fn new() -> Self {
        Self {
            config: TechnicalAlertConfig::default(),
        }
    }

    pub fn with_config(config: TechnicalAlertConfig) -> Self {
        Self { config }
    }

    pub fn evaluate(&self, input: &TechnicalAlertInput) -> Vec<TechnicalAlert> {
        let mut alerts = Vec::new();

        // RSI alerts
        if let Some(rsi) = input.rsi {
            if rsi >= self.config.rsi_overbought {
                alerts.push(TechnicalAlert::new(
                    input.symbol.clone(),
                    TechnicalAlertType::RsiOverbought { rsi },
                    AlertPriority::Medium,
                ));
            } else if rsi <= self.config.rsi_oversold {
                alerts.push(TechnicalAlert::new(
                    input.symbol.clone(),
                    TechnicalAlertType::RsiOversold { rsi },
                    AlertPriority::Medium,
                ));
            }
        }

        // MACD crossover alerts
        if let (Some(macd), Some(signal), Some(prev_macd), Some(prev_signal)) = (
            input.macd,
            input.macd_signal,
            input.prev_macd,
            input.prev_macd_signal,
        ) {
            // Bullish crossover: MACD crosses above signal
            if prev_macd <= prev_signal && macd > signal {
                alerts.push(TechnicalAlert::new(
                    input.symbol.clone(),
                    TechnicalAlertType::MacdBullishCrossover { macd, signal },
                    AlertPriority::High,
                ));
            }
            // Bearish crossover: MACD crosses below signal
            if prev_macd >= prev_signal && macd < signal {
                alerts.push(TechnicalAlert::new(
                    input.symbol.clone(),
                    TechnicalAlertType::MacdBearishCrossover { macd, signal },
                    AlertPriority::High,
                ));
            }
        }

        // Volume spike
        if let Some(rvol) = input.rvol {
            if rvol >= self.config.rvol_spike_threshold {
                alerts.push(TechnicalAlert::new(
                    input.symbol.clone(),
                    TechnicalAlertType::VolumeSpike { rvol },
                    AlertPriority::Medium,
                ));
            }
        }

        // EMA crossovers (Golden Cross / Death Cross)
        if let (Some(ema20), Some(ema50), Some(prev20), Some(prev50)) = (
            input.ema20,
            input.ema50,
            input.prev_ema20,
            input.prev_ema50,
        ) {
            // Golden Cross: EMA20 crosses above EMA50
            if prev20 <= prev50 && ema20 > ema50 {
                alerts.push(TechnicalAlert::new(
                    input.symbol.clone(),
                    TechnicalAlertType::GoldenCross {
                        ema_short: ema20,
                        ema_long: ema50,
                    },
                    AlertPriority::High,
                ));
            }
            // Death Cross: EMA20 crosses below EMA50
            if prev20 >= prev50 && ema20 < ema50 {
                alerts.push(TechnicalAlert::new(
                    input.symbol.clone(),
                    TechnicalAlertType::DeathCross {
                        ema_short: ema20,
                        ema_long: ema50,
                    },
                    AlertPriority::High,
                ));
            }
        }

        // Price breakout/breakdown
        if let Some(resistance) = input.resistance {
            if input.current_price > resistance {
                alerts.push(TechnicalAlert::new(
                    input.symbol.clone(),
                    TechnicalAlertType::PriceBreakout {
                        price: input.current_price,
                        resistance,
                    },
                    AlertPriority::High,
                ));
            }
        }
        if let Some(support) = input.support {
            if input.current_price < support {
                alerts.push(TechnicalAlert::new(
                    input.symbol.clone(),
                    TechnicalAlertType::PriceBreakdown {
                        price: input.current_price,
                        support,
                    },
                    AlertPriority::High,
                ));
            }
        }

        // Wyckoff alerts
        if let (Some(ref phase), Some(confidence)) =
            (input.wyckoff_phase.clone(), input.wyckoff_confidence)
        {
            if confidence >= self.config.wyckoff_min_confidence {
                match phase.as_str() {
                    "accumulation" => {
                        alerts.push(TechnicalAlert::new(
                            input.symbol.clone(),
                            TechnicalAlertType::WyckoffAccumulation { confidence },
                            AlertPriority::High,
                        ));
                    }
                    "distribution" => {
                        alerts.push(TechnicalAlert::new(
                            input.symbol.clone(),
                            TechnicalAlertType::WyckoffDistribution { confidence },
                            AlertPriority::High,
                        ));
                    }
                    _ => {}
                }
            }
        }

        if let Some(ref event) = input.wyckoff_event {
            match event.as_str() {
                "spring" => {
                    alerts.push(TechnicalAlert::new(
                        input.symbol.clone(),
                        TechnicalAlertType::WyckoffSpring {
                            price: input.current_price,
                        },
                        AlertPriority::Critical,
                    ));
                }
                "upthrust" => {
                    alerts.push(TechnicalAlert::new(
                        input.symbol.clone(),
                        TechnicalAlertType::WyckoffUpthrust {
                            price: input.current_price,
                        },
                        AlertPriority::Critical,
                    ));
                }
                _ => {}
            }
        }

        // Bollinger squeeze
        if let Some(bandwidth) = input.bollinger_bandwidth {
            if bandwidth <= self.config.bollinger_squeeze_threshold {
                alerts.push(TechnicalAlert::new(
                    input.symbol.clone(),
                    TechnicalAlertType::BollingerSqueeze { bandwidth },
                    AlertPriority::Medium,
                ));
            }
        }

        alerts
    }
}

impl Default for TechnicalAlertEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_tech_message(symbol: &str, alert_type: &TechnicalAlertType) -> String {
    match alert_type {
        TechnicalAlertType::RsiOverbought { rsi } => {
            format!("{}: RSI overbought at {:.1} - potential reversal", symbol, rsi)
        }
        TechnicalAlertType::RsiOversold { rsi } => {
            format!("{}: RSI oversold at {:.1} - potential bounce", symbol, rsi)
        }
        TechnicalAlertType::MacdBullishCrossover { .. } => {
            format!("{}: MACD bullish crossover - buy signal", symbol)
        }
        TechnicalAlertType::MacdBearishCrossover { .. } => {
            format!("{}: MACD bearish crossover - sell signal", symbol)
        }
        TechnicalAlertType::WyckoffAccumulation { confidence } => {
            format!(
                "{}: Wyckoff accumulation detected ({}% confidence)",
                symbol, confidence
            )
        }
        TechnicalAlertType::WyckoffDistribution { confidence } => {
            format!(
                "{}: Wyckoff distribution detected ({}% confidence)",
                symbol, confidence
            )
        }
        TechnicalAlertType::WyckoffSpring { price } => {
            format!(
                "{}: Wyckoff spring at {} - strong bullish signal",
                symbol, price
            )
        }
        TechnicalAlertType::WyckoffUpthrust { price } => {
            format!(
                "{}: Wyckoff upthrust at {} - strong bearish signal",
                symbol, price
            )
        }
        TechnicalAlertType::VolumeSpike { rvol } => {
            format!(
                "{}: Volume spike at {:.1}x average - unusual activity",
                symbol, rvol
            )
        }
        TechnicalAlertType::PriceBreakout { price, resistance } => {
            format!(
                "{}: Price breakout above {} resistance at {}",
                symbol, resistance, price
            )
        }
        TechnicalAlertType::PriceBreakdown { price, support } => {
            format!(
                "{}: Price breakdown below {} support at {}",
                symbol, support, price
            )
        }
        TechnicalAlertType::GoldenCross { .. } => {
            format!("{}: Golden cross - EMA20 crossed above EMA50", symbol)
        }
        TechnicalAlertType::DeathCross { .. } => {
            format!("{}: Death cross - EMA20 crossed below EMA50", symbol)
        }
        TechnicalAlertType::BollingerSqueeze { bandwidth } => {
            format!(
                "{}: Bollinger squeeze (bandwidth {:.3}) - breakout imminent",
                symbol, bandwidth
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_overbought() {
        let engine = TechnicalAlertEngine::new();
        let input = TechnicalAlertInput {
            symbol: "BBCA".into(),
            rsi: Some(dec!(75)),
            ..Default::default()
        };
        let alerts = engine.evaluate(&input);
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, TechnicalAlertType::RsiOverbought { .. })));
    }

    #[test]
    fn test_rsi_oversold() {
        let engine = TechnicalAlertEngine::new();
        let input = TechnicalAlertInput {
            symbol: "BBCA".into(),
            rsi: Some(dec!(25)),
            ..Default::default()
        };
        let alerts = engine.evaluate(&input);
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, TechnicalAlertType::RsiOversold { .. })));
    }

    #[test]
    fn test_macd_bullish_crossover() {
        let engine = TechnicalAlertEngine::new();
        let input = TechnicalAlertInput {
            symbol: "BBCA".into(),
            macd: Some(dec!(1.5)),
            macd_signal: Some(dec!(1.0)),
            prev_macd: Some(dec!(0.9)),
            prev_macd_signal: Some(dec!(1.0)),
            ..Default::default()
        };
        let alerts = engine.evaluate(&input);
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, TechnicalAlertType::MacdBullishCrossover { .. })));
    }

    #[test]
    fn test_volume_spike() {
        let engine = TechnicalAlertEngine::new();
        let input = TechnicalAlertInput {
            symbol: "BBCA".into(),
            rvol: Some(dec!(3.0)),
            ..Default::default()
        };
        let alerts = engine.evaluate(&input);
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, TechnicalAlertType::VolumeSpike { .. })));
    }

    #[test]
    fn test_golden_cross() {
        let engine = TechnicalAlertEngine::new();
        let input = TechnicalAlertInput {
            symbol: "BBCA".into(),
            ema20: Some(dec!(100)),
            ema50: Some(dec!(95)),
            prev_ema20: Some(dec!(94)),
            prev_ema50: Some(dec!(95)),
            ..Default::default()
        };
        let alerts = engine.evaluate(&input);
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, TechnicalAlertType::GoldenCross { .. })));
    }

    #[test]
    fn test_price_breakout() {
        let engine = TechnicalAlertEngine::new();
        let input = TechnicalAlertInput {
            symbol: "BBCA".into(),
            current_price: dec!(10500),
            resistance: Some(dec!(10000)),
            ..Default::default()
        };
        let alerts = engine.evaluate(&input);
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, TechnicalAlertType::PriceBreakout { .. })));
    }

    #[test]
    fn test_wyckoff_spring() {
        let engine = TechnicalAlertEngine::new();
        let input = TechnicalAlertInput {
            symbol: "BBCA".into(),
            current_price: dec!(9500),
            wyckoff_event: Some("spring".into()),
            ..Default::default()
        };
        let alerts = engine.evaluate(&input);
        assert!(alerts
            .iter()
            .any(|a| matches!(a.alert_type, TechnicalAlertType::WyckoffSpring { .. })));
        assert!(alerts.iter().any(|a| a.priority == AlertPriority::Critical));
    }
}
