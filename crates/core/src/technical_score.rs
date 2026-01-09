//! Technical Score Engine
//!
//! Combines multiple technical indicators into a single 0-100 score.
//!
//! Score breakdown (default weights):
//! - Order Flow (OBI/OFI): 25%
//! - Broker Accumulation: 25%
//! - EMA Position: 15%
//! - Fibonacci Support: 15%
//! - Volume Analysis: 10%
//! - RSI/MACD Signals: 10%

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Technical score component breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalScoreBreakdown {
    pub total_score: Decimal,
    pub order_flow_score: Decimal,
    pub broker_score: Decimal,
    pub ema_score: Decimal,
    pub fibonacci_score: Decimal,
    pub volume_score: Decimal,
    pub momentum_score: Decimal,
    pub signals: Vec<String>,
}

/// Weights for technical score components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalWeights {
    pub order_flow: Decimal,
    pub broker: Decimal,
    pub ema: Decimal,
    pub fibonacci: Decimal,
    pub volume: Decimal,
    pub momentum: Decimal,
}

impl Default for TechnicalWeights {
    fn default() -> Self {
        Self {
            order_flow: dec!(0.25),
            broker: dec!(0.25),
            ema: dec!(0.15),
            fibonacci: dec!(0.15),
            volume: dec!(0.10),
            momentum: dec!(0.10),
        }
    }
}

/// Input data for technical score calculation
#[derive(Debug, Clone)]
pub struct TechnicalScoreInput {
    // Price data
    pub current_price: Decimal,
    pub prices: Vec<Decimal>,
    pub volumes: Vec<i64>,
    pub highs: Vec<Decimal>,
    pub lows: Vec<Decimal>,

    // Order flow (optional)
    pub obi: Option<Decimal>,
    pub ofi_trend: Option<Decimal>,

    // Broker data (optional)
    pub broker_score: Option<Decimal>,
    pub institutional_buying: bool,
    pub foreign_buying: bool,

    // Pre-calculated indicators (optional, will calculate if missing)
    pub ema20: Option<Decimal>,
    pub ema50: Option<Decimal>,
    pub rsi: Option<Decimal>,
    pub macd_histogram: Option<Decimal>,
}

impl Default for TechnicalScoreInput {
    fn default() -> Self {
        Self {
            current_price: Decimal::ZERO,
            prices: vec![],
            volumes: vec![],
            highs: vec![],
            lows: vec![],
            obi: None,
            ofi_trend: None,
            broker_score: None,
            institutional_buying: false,
            foreign_buying: false,
            ema20: None,
            ema50: None,
            rsi: None,
            macd_histogram: None,
        }
    }
}

/// Technical Score Engine
pub struct TechnicalScoreEngine {
    weights: TechnicalWeights,
}

impl TechnicalScoreEngine {
    /// Create new engine with default weights
    #[must_use]
    pub fn new() -> Self {
        Self {
            weights: TechnicalWeights::default(),
        }
    }

    /// Create engine with custom weights
    #[must_use]
    pub fn with_weights(weights: TechnicalWeights) -> Self {
        Self { weights }
    }

    /// Calculate technical score from input data
    #[must_use]
    pub fn calculate(&self, input: &TechnicalScoreInput) -> TechnicalScoreBreakdown {
        let mut signals = Vec::new();

        // 1. Order Flow Score (0-100)
        let order_flow_score = self.calculate_order_flow_score(input, &mut signals);

        // 2. Broker Score (0-100)
        let broker_score = self.calculate_broker_score(input, &mut signals);

        // 3. EMA Score (0-100)
        let ema_score = self.calculate_ema_score(input, &mut signals);

        // 4. Fibonacci Score (0-100)
        let fibonacci_score = self.calculate_fibonacci_score(input, &mut signals);

        // 5. Volume Score (0-100)
        let volume_score = self.calculate_volume_score(input, &mut signals);

        // 6. Momentum Score (RSI/MACD) (0-100)
        let momentum_score = self.calculate_momentum_score(input, &mut signals);

        // Calculate weighted total
        let total_score = (order_flow_score * self.weights.order_flow)
            + (broker_score * self.weights.broker)
            + (ema_score * self.weights.ema)
            + (fibonacci_score * self.weights.fibonacci)
            + (volume_score * self.weights.volume)
            + (momentum_score * self.weights.momentum);

        TechnicalScoreBreakdown {
            total_score: total_score.round_dp(2),
            order_flow_score: order_flow_score.round_dp(2),
            broker_score: broker_score.round_dp(2),
            ema_score: ema_score.round_dp(2),
            fibonacci_score: fibonacci_score.round_dp(2),
            volume_score: volume_score.round_dp(2),
            momentum_score: momentum_score.round_dp(2),
            signals,
        }
    }

    fn calculate_order_flow_score(
        &self,
        input: &TechnicalScoreInput,
        signals: &mut Vec<String>,
    ) -> Decimal {
        let mut score = dec!(50);

        // OBI contribution
        if let Some(obi) = input.obi {
            // OBI ranges from -1 to +1
            // Map to 0-100 contribution (centered at 50)
            let obi_contribution = obi * dec!(40);
            score += obi_contribution;

            if obi > dec!(0.2) {
                signals.push("Strong buying pressure (OBI)".to_string());
            } else if obi < dec!(-0.2) {
                signals.push("Strong selling pressure (OBI)".to_string());
            }
        }

        // OFI trend contribution
        if let Some(ofi) = input.ofi_trend {
            let ofi_normalized = ofi.max(dec!(-1)).min(dec!(1));
            score += ofi_normalized * dec!(10);

            if ofi > dec!(0.5) {
                signals.push("Positive order flow trend".to_string());
            }
        }

        score.max(Decimal::ZERO).min(dec!(100))
    }

    fn calculate_broker_score(
        &self,
        input: &TechnicalScoreInput,
        signals: &mut Vec<String>,
    ) -> Decimal {
        // Use pre-calculated broker score if available
        if let Some(score) = input.broker_score {
            if input.institutional_buying {
                signals.push("Institutional accumulation detected".to_string());
            }
            if input.foreign_buying {
                signals.push("Foreign net buying".to_string());
            }
            return score;
        }

        // Default neutral score
        let mut score = dec!(50);

        if input.institutional_buying {
            score += dec!(20);
            signals.push("Institutional buying".to_string());
        }

        if input.foreign_buying {
            score += dec!(10);
            signals.push("Foreign buying".to_string());
        }

        score.min(dec!(100))
    }

    fn calculate_ema_score(
        &self,
        input: &TechnicalScoreInput,
        signals: &mut Vec<String>,
    ) -> Decimal {
        let mut score = dec!(50);

        // Price vs EMA20
        if let Some(ema20) = input.ema20 {
            if input.current_price > ema20 {
                score += dec!(15);
                signals.push("Price above EMA20".to_string());
            } else {
                score -= dec!(10);
            }

            // Distance from EMA (closer = more significant)
            if ema20 > Decimal::ZERO {
                let distance_pct = ((input.current_price - ema20) / ema20 * dec!(100)).abs();
                if distance_pct < dec!(2) {
                    signals.push("Price near EMA20 (potential support/resistance)".to_string());
                }
            }
        }

        // EMA20 vs EMA50 (trend)
        if let (Some(ema20), Some(ema50)) = (input.ema20, input.ema50) {
            if ema20 > ema50 {
                score += dec!(15);
                signals.push("EMA20 above EMA50 (uptrend)".to_string());
            } else {
                score -= dec!(10);
            }
        }

        // Calculate EMA slope if we have enough price data
        if input.prices.len() >= 25 {
            // Simple slope check: compare recent EMA to older EMA
            // This is a simplified version; actual EMA slope would require the full EMA series
            let recent_avg: Decimal = input.prices[input.prices.len() - 5..]
                .iter()
                .sum::<Decimal>()
                / dec!(5);
            let older_avg: Decimal = input.prices[input.prices.len() - 10..input.prices.len() - 5]
                .iter()
                .sum::<Decimal>()
                / dec!(5);

            if recent_avg > older_avg {
                score += dec!(10);
            }
        }

        score.max(Decimal::ZERO).min(dec!(100))
    }

    fn calculate_fibonacci_score(
        &self,
        input: &TechnicalScoreInput,
        signals: &mut Vec<String>,
    ) -> Decimal {
        if input.highs.is_empty() || input.lows.is_empty() {
            return dec!(50);
        }

        let high = input.highs.iter().max().copied().unwrap_or(Decimal::ZERO);
        let low = input.lows.iter().min().copied().unwrap_or(Decimal::ZERO);

        if high == low {
            return dec!(50);
        }

        let range = high - low;
        let price = input.current_price;

        // Calculate Fibonacci levels
        let fib_382 = high - (range * dec!(0.382));
        let fib_500 = high - (range * dec!(0.5));
        let fib_618 = high - (range * dec!(0.618));

        // Score based on proximity to key levels
        let mut score = dec!(50);

        let distances = [
            ((price - fib_382).abs(), "38.2%"),
            ((price - fib_500).abs(), "50%"),
            ((price - fib_618).abs(), "61.8%"),
        ];

        let min_distance = distances.iter().min_by_key(|d| d.0).unwrap();
        let proximity_pct = min_distance.0 / range * dec!(100);

        if proximity_pct < dec!(2) {
            score += dec!(30);
            signals.push(format!(
                "Price at Fibonacci {} level (strong support)",
                min_distance.1
            ));
        } else if proximity_pct < dec!(5) {
            score += dec!(20);
            signals.push(format!("Price near Fibonacci {} level", min_distance.1));
        } else if proximity_pct < dec!(10) {
            score += dec!(10);
        }

        // Bonus if price is above 50% retracement (stronger position)
        if price > fib_500 {
            score += dec!(10);
        }

        score.max(Decimal::ZERO).min(dec!(100))
    }

    fn calculate_volume_score(
        &self,
        input: &TechnicalScoreInput,
        signals: &mut Vec<String>,
    ) -> Decimal {
        if input.volumes.len() < 20 {
            return dec!(50);
        }

        let mut score = dec!(50);

        // Calculate average volume
        let avg_volume: i64 = input.volumes.iter().sum::<i64>() / input.volumes.len() as i64;
        let current_volume = *input.volumes.last().unwrap_or(&0);

        if avg_volume > 0 {
            let rvol = Decimal::from(current_volume) / Decimal::from(avg_volume);

            // Volume spike detection
            if rvol > dec!(2) {
                score += dec!(20);
                signals.push("Volume spike (>2x average)".to_string());
            } else if rvol > dec!(1.5) {
                score += dec!(10);
                signals.push("Above average volume".to_string());
            } else if rvol < dec!(0.5) {
                score -= dec!(10);
            }
        }

        // OBV trend (if we have enough data)
        if input.prices.len() >= 10 && input.volumes.len() >= 10 {
            let recent_prices = &input.prices[input.prices.len() - 10..];
            let price_up = recent_prices.last() > recent_prices.first();

            // Simple volume trend
            let recent_vol: i64 = input.volumes[input.volumes.len() - 5..].iter().sum::<i64>();
            let older_vol: i64 = input.volumes[input.volumes.len() - 10..input.volumes.len() - 5]
                .iter()
                .sum::<i64>();
            let vol_increasing = recent_vol > older_vol;

            if price_up && vol_increasing {
                score += dec!(15);
                signals.push("Price up with increasing volume (bullish)".to_string());
            } else if !price_up && vol_increasing {
                score -= dec!(10);
                signals.push("Price down with increasing volume (bearish)".to_string());
            }
        }

        score.max(Decimal::ZERO).min(dec!(100))
    }

    fn calculate_momentum_score(
        &self,
        input: &TechnicalScoreInput,
        signals: &mut Vec<String>,
    ) -> Decimal {
        let mut score = dec!(50);

        // RSI
        if let Some(rsi) = input.rsi {
            if rsi > dec!(70) {
                score -= dec!(15);
                signals.push("RSI overbought (>70)".to_string());
            } else if rsi < dec!(30) {
                score += dec!(20);
                signals.push("RSI oversold (<30) - potential bounce".to_string());
            } else if rsi > dec!(50) {
                score += dec!(10);
            }
        }

        // MACD
        if let Some(macd_hist) = input.macd_histogram {
            if macd_hist > Decimal::ZERO {
                score += dec!(15);
                signals.push("MACD bullish (histogram positive)".to_string());
            } else {
                score -= dec!(10);
            }
        }

        score.max(Decimal::ZERO).min(dec!(100))
    }
}

impl Default for TechnicalScoreEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_weights_sum_to_one() {
        let weights = TechnicalWeights::default();
        let sum = weights.order_flow
            + weights.broker
            + weights.ema
            + weights.fibonacci
            + weights.volume
            + weights.momentum;
        assert_eq!(sum, dec!(1));
    }

    #[test]
    fn test_neutral_score() {
        let engine = TechnicalScoreEngine::new();
        let input = TechnicalScoreInput::default();
        let result = engine.calculate(&input);

        // With no data, should get neutral score around 50
        assert!(result.total_score >= dec!(40) && result.total_score <= dec!(60));
    }

    #[test]
    fn test_bullish_scenario() {
        let engine = TechnicalScoreEngine::new();
        let input = TechnicalScoreInput {
            current_price: dec!(100),
            obi: Some(dec!(0.3)),
            ofi_trend: Some(dec!(0.5)),
            broker_score: Some(dec!(75)),
            institutional_buying: true,
            foreign_buying: true,
            ema20: Some(dec!(95)),
            ema50: Some(dec!(90)),
            rsi: Some(dec!(55)),
            macd_histogram: Some(dec!(1)),
            ..Default::default()
        };

        let result = engine.calculate(&input);
        assert!(result.total_score > dec!(65));
        assert!(!result.signals.is_empty());
    }

    #[test]
    fn test_bearish_scenario() {
        let engine = TechnicalScoreEngine::new();
        let input = TechnicalScoreInput {
            current_price: dec!(100),
            obi: Some(dec!(-0.3)),
            broker_score: Some(dec!(30)),
            institutional_buying: false,
            foreign_buying: false,
            ema20: Some(dec!(105)),
            ema50: Some(dec!(110)),
            rsi: Some(dec!(75)),
            macd_histogram: Some(dec!(-1)),
            ..Default::default()
        };

        let result = engine.calculate(&input);
        assert!(result.total_score < dec!(45));
    }

    #[test]
    fn test_score_bounds() {
        let engine = TechnicalScoreEngine::new();

        // Test extreme bullish case
        let bullish_input = TechnicalScoreInput {
            current_price: dec!(100),
            obi: Some(dec!(1.0)),
            ofi_trend: Some(dec!(1.0)),
            broker_score: Some(dec!(100)),
            institutional_buying: true,
            foreign_buying: true,
            ema20: Some(dec!(90)),
            ema50: Some(dec!(80)),
            rsi: Some(dec!(25)),
            macd_histogram: Some(dec!(10)),
            ..Default::default()
        };

        let result = engine.calculate(&bullish_input);
        assert!(result.total_score <= dec!(100));
        assert!(result.total_score >= dec!(0));

        // Test extreme bearish case
        let bearish_input = TechnicalScoreInput {
            current_price: dec!(100),
            obi: Some(dec!(-1.0)),
            ofi_trend: Some(dec!(-1.0)),
            broker_score: Some(dec!(0)),
            institutional_buying: false,
            foreign_buying: false,
            ema20: Some(dec!(110)),
            ema50: Some(dec!(120)),
            rsi: Some(dec!(80)),
            macd_histogram: Some(dec!(-10)),
            ..Default::default()
        };

        let result = engine.calculate(&bearish_input);
        assert!(result.total_score <= dec!(100));
        assert!(result.total_score >= dec!(0));
    }

    #[test]
    fn test_custom_weights() {
        let weights = TechnicalWeights {
            order_flow: dec!(0.50),
            broker: dec!(0.20),
            ema: dec!(0.10),
            fibonacci: dec!(0.10),
            volume: dec!(0.05),
            momentum: dec!(0.05),
        };

        let engine = TechnicalScoreEngine::with_weights(weights);
        let input = TechnicalScoreInput {
            current_price: dec!(100),
            obi: Some(dec!(0.5)),
            broker_score: Some(dec!(50)),
            ..Default::default()
        };

        let result = engine.calculate(&input);
        // Order flow should have more impact with 50% weight
        assert!(result.total_score > dec!(50));
    }

    #[test]
    fn test_fibonacci_scoring() {
        let engine = TechnicalScoreEngine::new();

        // Price at 50% Fibonacci level
        let input = TechnicalScoreInput {
            current_price: dec!(50),
            highs: vec![dec!(100)],
            lows: vec![dec!(0)],
            ..Default::default()
        };

        let result = engine.calculate(&input);
        // Should detect proximity to 50% level
        assert!(result
            .signals
            .iter()
            .any(|s| s.contains("Fibonacci") && s.contains("50%")));
    }

    #[test]
    fn test_volume_scoring() {
        let engine = TechnicalScoreEngine::new();

        // Create volume data with a spike at the end
        let mut volumes = vec![100_000i64; 20];
        volumes[19] = 300_000; // 3x spike

        let input = TechnicalScoreInput {
            current_price: dec!(100),
            volumes,
            ..Default::default()
        };

        let result = engine.calculate(&input);
        assert!(result
            .signals
            .iter()
            .any(|s| s.contains("Volume spike") || s.contains("average volume")));
    }
}
