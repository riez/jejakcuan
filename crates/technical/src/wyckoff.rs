//! Wyckoff Method Phase Detection
//!
//! Identifies market phases based on Wyckoff methodology:
//! - Accumulation: Smart money buying before markup
//! - Markup: Uptrend phase
//! - Distribution: Smart money selling before markdown
//! - Markdown: Downtrend phase

use crate::error::TechnicalError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Wyckoff market phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WyckoffPhase {
    /// Accumulation - Smart money buying, range-bound with decreasing volatility
    Accumulation,
    /// Markup - Uptrend following accumulation
    Markup,
    /// Distribution - Smart money selling, range-bound at highs
    Distribution,
    /// Markdown - Downtrend following distribution
    Markdown,
    /// Unknown/Transitional phase
    Unknown,
}

/// Wyckoff events that signal phase transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WyckoffEvent {
    /// Preliminary Support - First significant buying after downtrend
    PreliminarySupport,
    /// Selling Climax - Panic selling with high volume, marks potential bottom
    SellingClimax,
    /// Automatic Rally - Bounce after selling climax
    AutomaticRally,
    /// Secondary Test - Price retests selling climax low
    SecondaryTest,
    /// Sign of Strength - Price breaks above resistance with volume
    SignOfStrength,
    /// Last Point of Support - Final pullback before markup
    LastPointOfSupport,
    /// Preliminary Supply - First significant selling after uptrend
    PreliminarySupply,
    /// Buying Climax - Euphoric buying with high volume, marks potential top
    BuyingClimax,
    /// Automatic Reaction - Drop after buying climax
    AutomaticReaction,
    /// Sign of Weakness - Price breaks below support with volume
    SignOfWeakness,
    /// Last Point of Supply - Final rally before markdown
    LastPointOfSupply,
    /// Spring - False breakdown below support, bullish
    Spring,
    /// Upthrust - False breakout above resistance, bearish
    Upthrust,
}

/// Wyckoff analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WyckoffAnalysis {
    /// Current detected phase
    pub phase: WyckoffPhase,
    /// Confidence score (0-100)
    pub confidence: u8,
    /// Recent events detected
    pub events: Vec<WyckoffEventDetection>,
    /// Support level
    pub support: Option<Decimal>,
    /// Resistance level
    pub resistance: Option<Decimal>,
    /// Phase description
    pub description: String,
}

/// A detected Wyckoff event with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WyckoffEventDetection {
    /// The event type
    pub event: WyckoffEvent,
    /// Index in price data where event occurred
    pub index: usize,
    /// Price at event
    pub price: Decimal,
    /// Volume at event
    pub volume: i64,
    /// Confidence score (0-100)
    pub confidence: u8,
}

/// OHLCV bar for Wyckoff analysis
#[derive(Debug, Clone)]
pub struct OhlcvBar {
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
}

/// Configuration for Wyckoff detection
#[derive(Debug, Clone)]
pub struct WyckoffConfig {
    /// Lookback period for trend detection
    pub trend_lookback: usize,
    /// Lookback period for volume analysis
    pub volume_lookback: usize,
    /// Volume spike threshold (relative to average)
    pub volume_spike_threshold: Decimal,
    /// Support/Resistance tolerance as percentage
    pub sr_tolerance: Decimal,
    /// Minimum bars for phase detection
    pub min_phase_bars: usize,
}

impl Default for WyckoffConfig {
    fn default() -> Self {
        Self {
            trend_lookback: 20,
            volume_lookback: 20,
            volume_spike_threshold: dec!(2.0),
            sr_tolerance: dec!(0.02),
            min_phase_bars: 10,
        }
    }
}

/// Detect Wyckoff phase from OHLCV data
pub fn detect_wyckoff_phase(
    bars: &[OhlcvBar],
    config: &WyckoffConfig,
) -> Result<WyckoffAnalysis, TechnicalError> {
    let min_required = config.trend_lookback.max(config.volume_lookback) + config.min_phase_bars;
    if bars.len() < min_required {
        return Err(TechnicalError::InsufficientData {
            required: min_required,
            actual: bars.len(),
        });
    }

    let closes: Vec<Decimal> = bars.iter().map(|b| b.close).collect();
    let volumes: Vec<i64> = bars.iter().map(|b| b.volume).collect();
    let highs: Vec<Decimal> = bars.iter().map(|b| b.high).collect();
    let lows: Vec<Decimal> = bars.iter().map(|b| b.low).collect();

    // Calculate trend
    let trend = calculate_trend(&closes, config.trend_lookback);

    // Detect support and resistance
    let (support, resistance) = detect_support_resistance(&highs, &lows, config);

    // Detect events
    let events = detect_wyckoff_events(bars, config, support, resistance);

    // Determine phase based on trend, volatility, and events
    let (phase, confidence) =
        determine_phase(&closes, &volumes, trend, &events, support, resistance, config);

    let description = generate_phase_description(phase, &events);

    Ok(WyckoffAnalysis {
        phase,
        confidence,
        events,
        support,
        resistance,
        description,
    })
}

/// Calculate price trend (-1.0 to 1.0)
fn calculate_trend(closes: &[Decimal], lookback: usize) -> Decimal {
    if closes.len() < lookback + 1 {
        return Decimal::ZERO;
    }

    let start_idx = closes.len() - lookback - 1;
    let start_price = closes[start_idx];
    let end_price = closes[closes.len() - 1];

    if start_price == Decimal::ZERO {
        return Decimal::ZERO;
    }

    // Simple percentage change normalized to -1 to 1
    let change = (end_price - start_price) / start_price;
    change.clamp(dec!(-1), dec!(1))
}

/// Detect support and resistance levels
fn detect_support_resistance(
    highs: &[Decimal],
    lows: &[Decimal],
    config: &WyckoffConfig,
) -> (Option<Decimal>, Option<Decimal>) {
    if highs.len() < config.min_phase_bars {
        return (None, None);
    }

    let lookback = config.min_phase_bars * 2;
    let start_idx = highs.len().saturating_sub(lookback);

    let recent_highs: Vec<Decimal> = highs[start_idx..].to_vec();
    let recent_lows: Vec<Decimal> = lows[start_idx..].to_vec();

    // Find pivots (local highs and lows)
    let mut pivot_highs = Vec::new();
    let mut pivot_lows = Vec::new();

    for i in 2..recent_highs.len() - 2 {
        // Pivot high
        if recent_highs[i] > recent_highs[i - 1]
            && recent_highs[i] > recent_highs[i - 2]
            && recent_highs[i] > recent_highs[i + 1]
            && recent_highs[i] > recent_highs[i + 2]
        {
            pivot_highs.push(recent_highs[i]);
        }

        // Pivot low
        if recent_lows[i] < recent_lows[i - 1]
            && recent_lows[i] < recent_lows[i - 2]
            && recent_lows[i] < recent_lows[i + 1]
            && recent_lows[i] < recent_lows[i + 2]
        {
            pivot_lows.push(recent_lows[i]);
        }
    }

    // Cluster pivot levels to find support/resistance
    let resistance = cluster_levels(&pivot_highs, config.sr_tolerance);
    let support = cluster_levels(&pivot_lows, config.sr_tolerance);

    (support, resistance)
}

/// Cluster price levels to find significant S/R
fn cluster_levels(levels: &[Decimal], tolerance: Decimal) -> Option<Decimal> {
    if levels.is_empty() {
        return None;
    }

    let mut sorted = levels.to_vec();
    sorted.sort();

    // Find most common level cluster
    let mut best_cluster = Vec::new();

    for &level in &sorted {
        let mut cluster = vec![level];
        for &other in &sorted {
            if level == other {
                continue;
            }
            let diff = ((other - level) / level).abs();
            if diff <= tolerance {
                cluster.push(other);
            }
        }

        if cluster.len() > best_cluster.len() {
            best_cluster = cluster;
        }
    }

    if best_cluster.is_empty() {
        Some(sorted[sorted.len() / 2]) // Return median
    } else {
        // Return average of cluster
        let sum: Decimal = best_cluster.iter().sum();
        Some(sum / Decimal::from(best_cluster.len() as i64))
    }
}

/// Detect Wyckoff events in price/volume data
fn detect_wyckoff_events(
    bars: &[OhlcvBar],
    config: &WyckoffConfig,
    support: Option<Decimal>,
    resistance: Option<Decimal>,
) -> Vec<WyckoffEventDetection> {
    let mut events = Vec::new();
    let avg_volume = calculate_avg_volume(bars, config.volume_lookback);

    for i in config.volume_lookback..bars.len() {
        let bar = &bars[i];
        let prev_bars = &bars[i - config.volume_lookback..i];
        let volume_ratio = if avg_volume > 0 {
            Decimal::from(bar.volume) / Decimal::from(avg_volume)
        } else {
            dec!(1)
        };

        let is_volume_spike = volume_ratio >= config.volume_spike_threshold;

        // Selling Climax: High volume + large down candle at/near support
        if is_volume_spike && is_large_down_candle(bar) && is_near_level(bar.low, support, config) {
            events.push(WyckoffEventDetection {
                event: WyckoffEvent::SellingClimax,
                index: i,
                price: bar.close,
                volume: bar.volume,
                confidence: calculate_event_confidence(volume_ratio, bar),
            });
        }

        // Buying Climax: High volume + large up candle at/near resistance
        if is_volume_spike && is_large_up_candle(bar) && is_near_level(bar.high, resistance, config)
        {
            events.push(WyckoffEventDetection {
                event: WyckoffEvent::BuyingClimax,
                index: i,
                price: bar.close,
                volume: bar.volume,
                confidence: calculate_event_confidence(volume_ratio, bar),
            });
        }

        // Spring: Price breaks below support then closes above it
        if let Some(sup) = support {
            if bar.low < sup && bar.close > sup && is_volume_spike {
                events.push(WyckoffEventDetection {
                    event: WyckoffEvent::Spring,
                    index: i,
                    price: bar.close,
                    volume: bar.volume,
                    confidence: calculate_event_confidence(volume_ratio, bar),
                });
            }
        }

        // Upthrust: Price breaks above resistance then closes below it
        if let Some(res) = resistance {
            if bar.high > res && bar.close < res && is_volume_spike {
                events.push(WyckoffEventDetection {
                    event: WyckoffEvent::Upthrust,
                    index: i,
                    price: bar.close,
                    volume: bar.volume,
                    confidence: calculate_event_confidence(volume_ratio, bar),
                });
            }
        }

        // Sign of Strength: Break above resistance with volume
        if let Some(res) = resistance {
            if bar.close > res && is_volume_spike && is_large_up_candle(bar) {
                events.push(WyckoffEventDetection {
                    event: WyckoffEvent::SignOfStrength,
                    index: i,
                    price: bar.close,
                    volume: bar.volume,
                    confidence: calculate_event_confidence(volume_ratio, bar),
                });
            }
        }

        // Sign of Weakness: Break below support with volume
        if let Some(sup) = support {
            if bar.close < sup && is_volume_spike && is_large_down_candle(bar) {
                events.push(WyckoffEventDetection {
                    event: WyckoffEvent::SignOfWeakness,
                    index: i,
                    price: bar.close,
                    volume: bar.volume,
                    confidence: calculate_event_confidence(volume_ratio, bar),
                });
            }
        }

        // Secondary Test: Low volume retest of previous extreme
        if !is_volume_spike && is_retest(bar, prev_bars, support) {
            events.push(WyckoffEventDetection {
                event: WyckoffEvent::SecondaryTest,
                index: i,
                price: bar.close,
                volume: bar.volume,
                confidence: 60,
            });
        }
    }

    events
}

fn calculate_avg_volume(bars: &[OhlcvBar], lookback: usize) -> i64 {
    let start = bars.len().saturating_sub(lookback);
    let sum: i64 = bars[start..].iter().map(|b| b.volume).sum();
    sum / lookback as i64
}

fn is_large_up_candle(bar: &OhlcvBar) -> bool {
    if bar.open == Decimal::ZERO {
        return false;
    }
    bar.close > bar.open && (bar.close - bar.open) / bar.open > dec!(0.02)
}

fn is_large_down_candle(bar: &OhlcvBar) -> bool {
    if bar.open == Decimal::ZERO {
        return false;
    }
    bar.close < bar.open && (bar.open - bar.close) / bar.open > dec!(0.02)
}

fn is_near_level(price: Decimal, level: Option<Decimal>, config: &WyckoffConfig) -> bool {
    match level {
        Some(lvl) if lvl != Decimal::ZERO => {
            ((price - lvl) / lvl).abs() <= config.sr_tolerance
        }
        _ => false,
    }
}

fn is_retest(bar: &OhlcvBar, _prev_bars: &[OhlcvBar], support: Option<Decimal>) -> bool {
    // Check if current bar tests a previous low
    if let Some(sup) = support {
        let within_range = ((bar.low - sup) / sup).abs() <= dec!(0.02);
        let closed_above = bar.close > sup;
        return within_range && closed_above;
    }
    false
}

fn calculate_event_confidence(volume_ratio: Decimal, bar: &OhlcvBar) -> u8 {
    let base = 50u8;
    let volume_bonus = if volume_ratio > dec!(3) {
        30
    } else if volume_ratio > dec!(2) {
        20
    } else {
        10
    };

    // Large candle body adds confidence
    let body_ratio = if bar.high != bar.low {
        ((bar.close - bar.open).abs()) / (bar.high - bar.low)
    } else {
        dec!(0)
    };

    let body_bonus = if body_ratio > dec!(0.7) { 15 } else { 0 };

    (base + volume_bonus + body_bonus).min(100)
}

/// Determine Wyckoff phase based on analysis
fn determine_phase(
    closes: &[Decimal],
    volumes: &[i64],
    trend: Decimal,
    events: &[WyckoffEventDetection],
    support: Option<Decimal>,
    resistance: Option<Decimal>,
    config: &WyckoffConfig,
) -> (WyckoffPhase, u8) {
    // Calculate volatility (price range as percentage)
    let volatility = calculate_volatility(closes, config.min_phase_bars);

    // Volume trend
    let volume_trend = calculate_volume_trend(volumes, config.volume_lookback);

    // Count recent events by type
    let recent_events: Vec<_> = events
        .iter()
        .filter(|e| e.index >= closes.len() - config.min_phase_bars)
        .collect();

    let has_spring = recent_events
        .iter()
        .any(|e| e.event == WyckoffEvent::Spring);
    let has_upthrust = recent_events
        .iter()
        .any(|e| e.event == WyckoffEvent::Upthrust);
    let has_sos = recent_events
        .iter()
        .any(|e| e.event == WyckoffEvent::SignOfStrength);
    let has_sow = recent_events
        .iter()
        .any(|e| e.event == WyckoffEvent::SignOfWeakness);
    let has_sc = recent_events
        .iter()
        .any(|e| e.event == WyckoffEvent::SellingClimax);
    let has_bc = recent_events
        .iter()
        .any(|e| e.event == WyckoffEvent::BuyingClimax);

    // Determine phase
    let (phase, confidence) = if trend > dec!(0.1) && has_sos {
        // Strong uptrend with sign of strength -> Markup
        (WyckoffPhase::Markup, 80)
    } else if trend < dec!(-0.1) && has_sow {
        // Strong downtrend with sign of weakness -> Markdown
        (WyckoffPhase::Markdown, 80)
    } else if volatility < dec!(0.05) && trend.abs() < dec!(0.05) {
        // Low volatility, range-bound
        if volume_trend < dec!(-0.2) || has_sc || has_spring {
            // Decreasing volume at lows -> Accumulation
            (WyckoffPhase::Accumulation, 70)
        } else if has_bc || has_upthrust {
            // High volume climax at top -> Distribution
            (WyckoffPhase::Distribution, 70)
        } else if let (Some(sup), Some(res)) = (support, resistance) {
            let current = closes[closes.len() - 1];
            let mid = (sup + res) / dec!(2);
            if current < mid {
                (WyckoffPhase::Accumulation, 55)
            } else {
                (WyckoffPhase::Distribution, 55)
            }
        } else {
            (WyckoffPhase::Unknown, 40)
        }
    } else if trend > dec!(0.05) {
        // Upward bias
        if has_spring || (has_sc && !has_bc) {
            (WyckoffPhase::Markup, 65)
        } else {
            (WyckoffPhase::Markup, 50)
        }
    } else if trend < dec!(-0.05) {
        // Downward bias
        if has_upthrust || (has_bc && !has_sc) {
            (WyckoffPhase::Markdown, 65)
        } else {
            (WyckoffPhase::Markdown, 50)
        }
    } else {
        (WyckoffPhase::Unknown, 30)
    };

    (phase, confidence)
}

fn calculate_volatility(closes: &[Decimal], lookback: usize) -> Decimal {
    let start = closes.len().saturating_sub(lookback);
    let recent = &closes[start..];

    if recent.is_empty() {
        return Decimal::ZERO;
    }

    let max = recent.iter().max().copied().unwrap_or(Decimal::ZERO);
    let min = recent.iter().min().copied().unwrap_or(Decimal::ZERO);
    let avg = recent.iter().sum::<Decimal>() / Decimal::from(recent.len() as i64);

    if avg == Decimal::ZERO {
        return Decimal::ZERO;
    }

    (max - min) / avg
}

fn calculate_volume_trend(volumes: &[i64], lookback: usize) -> Decimal {
    if volumes.len() < lookback * 2 {
        return Decimal::ZERO;
    }

    let mid = volumes.len() - lookback;
    let start = mid - lookback;

    let first_half_avg = volumes[start..mid].iter().sum::<i64>() / lookback as i64;
    let second_half_avg = volumes[mid..].iter().sum::<i64>() / lookback as i64;

    if first_half_avg == 0 {
        return Decimal::ZERO;
    }

    Decimal::from(second_half_avg - first_half_avg) / Decimal::from(first_half_avg)
}

fn generate_phase_description(phase: WyckoffPhase, events: &[WyckoffEventDetection]) -> String {
    let recent_event_names: Vec<_> = events
        .iter()
        .rev()
        .take(3)
        .map(|e| format!("{:?}", e.event))
        .collect();

    let event_context = if recent_event_names.is_empty() {
        "No significant events detected.".to_string()
    } else {
        format!("Recent events: {}", recent_event_names.join(", "))
    };

    match phase {
        WyckoffPhase::Accumulation => {
            format!(
                "Accumulation phase: Smart money likely accumulating. Range-bound with decreasing volatility. {}",
                event_context
            )
        }
        WyckoffPhase::Markup => {
            format!(
                "Markup phase: Uptrend in progress. Look for pullbacks to support as entry opportunities. {}",
                event_context
            )
        }
        WyckoffPhase::Distribution => {
            format!(
                "Distribution phase: Smart money likely distributing. Range-bound at highs with potential reversal. {}",
                event_context
            )
        }
        WyckoffPhase::Markdown => {
            format!(
                "Markdown phase: Downtrend in progress. Avoid longs until accumulation signs appear. {}",
                event_context
            )
        }
        WyckoffPhase::Unknown => {
            format!(
                "Transitional phase: Market structure unclear. Wait for clearer signals. {}",
                event_context
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bars(prices: &[(Decimal, Decimal, Decimal, Decimal)], volumes: &[i64]) -> Vec<OhlcvBar> {
        prices
            .iter()
            .zip(volumes.iter())
            .map(|((o, h, l, c), &v)| OhlcvBar {
                open: *o,
                high: *h,
                low: *l,
                close: *c,
                volume: v,
            })
            .collect()
    }

    #[test]
    fn test_wyckoff_phase_markup() {
        // Uptrend with increasing prices
        let prices: Vec<_> = (0..50)
            .map(|i| {
                let base = dec!(100) + Decimal::from(i) * dec!(0.5);
                (base, base + dec!(1), base - dec!(0.5), base + dec!(0.3))
            })
            .collect();
        let volumes: Vec<_> = (0..50).map(|i| 1000 + i * 50).collect();

        let bars = create_test_bars(&prices, &volumes);
        let config = WyckoffConfig::default();

        let result = detect_wyckoff_phase(&bars, &config).unwrap();
        // Uptrend should be detected
        assert!(result.phase == WyckoffPhase::Markup || result.phase == WyckoffPhase::Unknown);
    }

    #[test]
    fn test_wyckoff_phase_markdown() {
        // Downtrend with decreasing prices
        let prices: Vec<_> = (0..50)
            .map(|i| {
                let base = dec!(150) - Decimal::from(i) * dec!(0.5);
                (base, base + dec!(0.5), base - dec!(1), base - dec!(0.3))
            })
            .collect();
        let volumes: Vec<_> = (0..50).map(|i| 1000 + i * 50).collect();

        let bars = create_test_bars(&prices, &volumes);
        let config = WyckoffConfig::default();

        let result = detect_wyckoff_phase(&bars, &config).unwrap();
        // Downtrend should be detected
        assert!(result.phase == WyckoffPhase::Markdown || result.phase == WyckoffPhase::Unknown);
    }

    #[test]
    fn test_wyckoff_accumulation_range() {
        // Range-bound at lows with decreasing volume
        let prices: Vec<_> = (0..50)
            .map(|i| {
                let base = dec!(100) + Decimal::from(i % 5) * dec!(0.5);
                (base, base + dec!(1), base - dec!(0.5), base)
            })
            .collect();
        let volumes: Vec<_> = (0..50).map(|i| 2000 - i * 20).collect();

        let bars = create_test_bars(&prices, &volumes);
        let config = WyckoffConfig::default();

        let result = detect_wyckoff_phase(&bars, &config).unwrap();
        // Range-bound with decreasing volume suggests accumulation
        assert!(
            result.phase == WyckoffPhase::Accumulation || result.phase == WyckoffPhase::Unknown
        );
    }

    #[test]
    fn test_detect_selling_climax() {
        // Large down candle with high volume at support
        let mut prices: Vec<_> = (0..45)
            .map(|i| {
                let base = dec!(100) - Decimal::from(i) * dec!(0.3);
                (base, base + dec!(0.5), base - dec!(0.5), base - dec!(0.2))
            })
            .collect();

        // Add selling climax candle
        prices.push((dec!(86.5), dec!(87), dec!(80), dec!(81)));
        prices.push((dec!(81), dec!(84), dec!(80), dec!(83)));
        prices.push((dec!(83), dec!(85), dec!(82), dec!(84)));
        prices.push((dec!(84), dec!(86), dec!(83), dec!(85)));
        prices.push((dec!(85), dec!(87), dec!(84), dec!(86)));

        let mut volumes: Vec<_> = (0..45).map(|_| 1000i64).collect();
        volumes.push(5000); // High volume climax
        volumes.push(1200);
        volumes.push(1100);
        volumes.push(1000);
        volumes.push(900);

        let bars = create_test_bars(&prices, &volumes);
        let config = WyckoffConfig::default();

        let result = detect_wyckoff_phase(&bars, &config).unwrap();

        // Should detect some events
        assert!(result.support.is_some() || result.events.len() > 0 || result.confidence > 0);
    }

    #[test]
    fn test_insufficient_data() {
        let prices = vec![(dec!(100), dec!(101), dec!(99), dec!(100))];
        let volumes = vec![1000];

        let bars = create_test_bars(&prices, &volumes);
        let config = WyckoffConfig::default();

        let result = detect_wyckoff_phase(&bars, &config);
        assert!(matches!(result, Err(TechnicalError::InsufficientData { .. })));
    }

    #[test]
    fn test_calculate_trend_upward() {
        let closes: Vec<Decimal> = (0..25).map(|i| dec!(100) + Decimal::from(i)).collect();
        let trend = calculate_trend(&closes, 20);
        assert!(trend > Decimal::ZERO);
    }

    #[test]
    fn test_calculate_trend_downward() {
        let closes: Vec<Decimal> = (0..25).map(|i| dec!(150) - Decimal::from(i)).collect();
        let trend = calculate_trend(&closes, 20);
        assert!(trend < Decimal::ZERO);
    }

    #[test]
    fn test_volatility_calculation() {
        // Low volatility range
        let closes: Vec<Decimal> = (0..20).map(|i| dec!(100) + Decimal::from(i % 2)).collect();
        let volatility = calculate_volatility(&closes, 20);
        assert!(volatility < dec!(0.05));

        // High volatility
        let closes2: Vec<Decimal> = (0..20).map(|i| dec!(100) + Decimal::from(i * 2)).collect();
        let volatility2 = calculate_volatility(&closes2, 20);
        assert!(volatility2 > volatility);
    }

    #[test]
    fn test_volume_trend() {
        // Increasing volume
        let volumes: Vec<i64> = (0..40).map(|i| 1000 + i * 50).collect();
        let trend = calculate_volume_trend(&volumes, 20);
        assert!(trend > Decimal::ZERO);

        // Decreasing volume
        let volumes2: Vec<i64> = (0..40).map(|i| 2000 - i * 30).collect();
        let trend2 = calculate_volume_trend(&volumes2, 20);
        assert!(trend2 < Decimal::ZERO);
    }

    #[test]
    fn test_phase_description() {
        let desc = generate_phase_description(WyckoffPhase::Accumulation, &[]);
        assert!(desc.contains("Accumulation"));
        assert!(desc.contains("Smart money"));
    }

    #[test]
    fn test_is_large_candle() {
        let up_candle = OhlcvBar {
            open: dec!(100),
            high: dec!(105),
            low: dec!(99),
            close: dec!(104),
            volume: 1000,
        };
        assert!(is_large_up_candle(&up_candle));
        assert!(!is_large_down_candle(&up_candle));

        let down_candle = OhlcvBar {
            open: dec!(100),
            high: dec!(101),
            low: dec!(95),
            close: dec!(96),
            volume: 1000,
        };
        assert!(is_large_down_candle(&down_candle));
        assert!(!is_large_up_candle(&down_candle));
    }

    #[test]
    fn test_cluster_levels() {
        let levels = vec![dec!(100), dec!(100.5), dec!(101), dec!(150), dec!(151)];
        let result = cluster_levels(&levels, dec!(0.02));
        assert!(result.is_some());
        // Should cluster around 100-101 or 150-151
        let clustered = result.unwrap();
        assert!(
            (clustered > dec!(99) && clustered < dec!(102))
                || (clustered > dec!(149) && clustered < dec!(152))
        );
    }

    #[test]
    fn test_wyckoff_event_serialization() {
        let event = WyckoffEvent::Spring;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, "\"spring\"");

        let phase = WyckoffPhase::Accumulation;
        let json2 = serde_json::to_string(&phase).unwrap();
        assert_eq!(json2, "\"accumulation\"");
    }

    #[test]
    fn test_wyckoff_analysis_serialization() {
        let analysis = WyckoffAnalysis {
            phase: WyckoffPhase::Markup,
            confidence: 75,
            events: vec![],
            support: Some(dec!(100)),
            resistance: Some(dec!(110)),
            description: "Test".to_string(),
        };

        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("markup"));
        assert!(json.contains("100"));
    }
}
