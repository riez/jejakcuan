//! Order Flow Analysis
//!
//! Order flow indicators measure buying and selling pressure:
//! - OBI (Order Book Imbalance): Measures bid/ask volume imbalance
//! - OFI (Order Flow Imbalance): Measures changes in bid/ask volumes

use crate::error::TechnicalError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Order book snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub timestamp: i64,
    pub bid_price: Decimal,
    pub bid_volume: i64,
    pub ask_price: Decimal,
    pub ask_volume: i64,
}

/// Order Book Imbalance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObiResult {
    pub obi: Decimal, // -1 to +1
    pub interpretation: String,
}

/// Order Flow Imbalance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfiResult {
    pub ofi: Decimal,
    pub cumulative_ofi: Decimal,
    pub interpretation: String,
}

/// Calculate Order Book Imbalance (OBI)
///
/// OBI = (Σ bid_volume - Σ ask_volume) / (Σ bid_volume + Σ ask_volume)
///
/// Returns value from -1 to +1:
/// - Positive: More buying pressure (bids > asks)
/// - Negative: More selling pressure (asks > bids)
/// - Above 0.2: Strong buying pressure
/// - Below -0.2: Strong selling pressure
pub fn calculate_obi(bid_volume: i64, ask_volume: i64) -> ObiResult {
    let total = bid_volume + ask_volume;

    if total == 0 {
        return ObiResult {
            obi: Decimal::ZERO,
            interpretation: "no_volume".to_string(),
        };
    }

    let obi = Decimal::from(bid_volume - ask_volume) / Decimal::from(total);

    let interpretation = if obi > dec!(0.2) {
        "strong_buying_pressure"
    } else if obi > dec!(0.05) {
        "buying_pressure"
    } else if obi < dec!(-0.2) {
        "strong_selling_pressure"
    } else if obi < dec!(-0.05) {
        "selling_pressure"
    } else {
        "neutral"
    };

    ObiResult {
        obi,
        interpretation: interpretation.to_string(),
    }
}

/// Calculate OBI from multiple price levels
/// Weights levels closer to mid-price more heavily
pub fn calculate_obi_multilevel(
    bids: &[(Decimal, i64)], // (price, volume)
    asks: &[(Decimal, i64)],
    mid_price: Decimal,
    max_distance_pct: Decimal,
) -> ObiResult {
    let mut weighted_bid_vol = Decimal::ZERO;
    let mut weighted_ask_vol = Decimal::ZERO;

    // Weight bids by proximity to mid price
    for (price, volume) in bids {
        let distance_pct = (mid_price - *price).abs() / mid_price * dec!(100);
        if distance_pct <= max_distance_pct {
            let weight = dec!(1) - (distance_pct / max_distance_pct);
            weighted_bid_vol += Decimal::from(*volume) * weight;
        }
    }

    // Weight asks by proximity to mid price
    for (price, volume) in asks {
        let distance_pct = (*price - mid_price).abs() / mid_price * dec!(100);
        if distance_pct <= max_distance_pct {
            let weight = dec!(1) - (distance_pct / max_distance_pct);
            weighted_ask_vol += Decimal::from(*volume) * weight;
        }
    }

    let total = weighted_bid_vol + weighted_ask_vol;

    if total == Decimal::ZERO {
        return ObiResult {
            obi: Decimal::ZERO,
            interpretation: "no_volume".to_string(),
        };
    }

    let obi = (weighted_bid_vol - weighted_ask_vol) / total;

    let interpretation = if obi > dec!(0.2) {
        "strong_buying_pressure"
    } else if obi > dec!(0.05) {
        "buying_pressure"
    } else if obi < dec!(-0.2) {
        "strong_selling_pressure"
    } else if obi < dec!(-0.05) {
        "selling_pressure"
    } else {
        "neutral"
    };

    ObiResult {
        obi,
        interpretation: interpretation.to_string(),
    }
}

/// Calculate Order Flow Imbalance (OFI) between two snapshots
///
/// OFI measures the change in bid/ask volumes:
/// - If bid price increases: ΔV_bid = new bid volume
/// - If bid price unchanged: ΔV_bid = new - old bid volume
/// - If bid price decreases: ΔV_bid = -old bid volume
///
/// Same logic applies for ask side.
///
/// OFI(t) = ΔV_bid - ΔV_ask
pub fn calculate_ofi(prev: &OrderBookSnapshot, current: &OrderBookSnapshot) -> OfiResult {
    // Calculate bid side contribution
    let delta_bid = if current.bid_price > prev.bid_price {
        current.bid_volume
    } else if current.bid_price == prev.bid_price {
        current.bid_volume - prev.bid_volume
    } else {
        -prev.bid_volume
    };

    // Calculate ask side contribution
    let delta_ask = if current.ask_price < prev.ask_price {
        current.ask_volume
    } else if current.ask_price == prev.ask_price {
        current.ask_volume - prev.ask_volume
    } else {
        -prev.ask_volume
    };

    let ofi = Decimal::from(delta_bid - delta_ask);

    let interpretation = if ofi > Decimal::ZERO {
        "buying_pressure"
    } else if ofi < Decimal::ZERO {
        "selling_pressure"
    } else {
        "neutral"
    };

    OfiResult {
        ofi,
        cumulative_ofi: ofi, // Will be accumulated externally
        interpretation: interpretation.to_string(),
    }
}

/// Calculate cumulative OFI for a series of snapshots
pub fn calculate_cumulative_ofi(
    snapshots: &[OrderBookSnapshot],
) -> Result<Vec<OfiResult>, TechnicalError> {
    if snapshots.len() < 2 {
        return Err(TechnicalError::InsufficientData {
            required: 2,
            actual: snapshots.len(),
        });
    }

    let mut results = vec![OfiResult {
        ofi: Decimal::ZERO,
        cumulative_ofi: Decimal::ZERO,
        interpretation: "initial".to_string(),
    }];

    let mut cumulative = Decimal::ZERO;

    for i in 1..snapshots.len() {
        let mut ofi_result = calculate_ofi(&snapshots[i - 1], &snapshots[i]);
        cumulative += ofi_result.ofi;
        ofi_result.cumulative_ofi = cumulative;
        results.push(ofi_result);
    }

    Ok(results)
}

/// Volume-Adjusted Mid Price (VAMP)
/// VAMP = (P_bid × Q_ask + P_ask × Q_bid) / (Q_bid + Q_ask)
/// Gives more weight to the side with less liquidity
pub fn calculate_vamp(
    bid_price: Decimal,
    bid_volume: i64,
    ask_price: Decimal,
    ask_volume: i64,
) -> Decimal {
    let total_volume = bid_volume + ask_volume;

    if total_volume == 0 {
        return (bid_price + ask_price) / dec!(2);
    }

    let bid_vol = Decimal::from(bid_volume);
    let ask_vol = Decimal::from(ask_volume);

    (bid_price * ask_vol + ask_price * bid_vol) / Decimal::from(total_volume)
}

/// Calculate buy/sell volume split based on price position
/// Buy volume = Volume × (Close - Low) / (High - Low)
/// Sell volume = Volume × (High - Close) / (High - Low)
pub fn split_volume(high: Decimal, low: Decimal, close: Decimal, volume: i64) -> (i64, i64) {
    let range = high - low;

    if range == Decimal::ZERO {
        // No range - split evenly
        return (volume / 2, volume / 2);
    }

    let buy_ratio = (close - low) / range;
    let sell_ratio = (high - close) / range;

    let vol_dec = Decimal::from(volume);
    let buy_vol = (vol_dec * buy_ratio)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0) as i64;
    let sell_vol = (vol_dec * sell_ratio)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0) as i64;

    (buy_vol, sell_vol)
}

/// Calculate Chaikin Money Flow Multiplier
/// MFM = ((Close - Low) - (High - Close)) / (High - Low)
pub fn money_flow_multiplier(high: Decimal, low: Decimal, close: Decimal) -> Decimal {
    let range = high - low;

    if range == Decimal::ZERO {
        return Decimal::ZERO;
    }

    ((close - low) - (high - close)) / range
}

/// Calculate Accumulation/Distribution Line
pub fn calculate_adl(
    highs: &[Decimal],
    lows: &[Decimal],
    closes: &[Decimal],
    volumes: &[i64],
) -> Result<Vec<Decimal>, TechnicalError> {
    if highs.len() != lows.len() || lows.len() != closes.len() || closes.len() != volumes.len() {
        return Err(TechnicalError::CalculationError(
            "All inputs must have same length".to_string(),
        ));
    }

    if highs.is_empty() {
        return Ok(vec![]);
    }

    let mut adl = Vec::with_capacity(highs.len());
    let mut cumulative = Decimal::ZERO;

    for i in 0..highs.len() {
        let mfm = money_flow_multiplier(highs[i], lows[i], closes[i]);
        let mfv = mfm * Decimal::from(volumes[i]);
        cumulative += mfv;
        adl.push(cumulative);
    }

    Ok(adl)
}

/// Generate order flow score for technical analysis
/// Combines OBI, OFI trend, and volume analysis
pub fn order_flow_score(
    obi: Decimal,
    ofi_trend: Decimal, // Positive = rising, negative = falling
    volume_spike: bool,
) -> Decimal {
    let mut score = dec!(50);

    // OBI contribution (±20 points)
    score += obi * dec!(20);

    // OFI trend contribution (±20 points)
    let ofi_normalized = ofi_trend.max(dec!(-1)).min(dec!(1));
    score += ofi_normalized * dec!(20);

    // Volume spike bonus
    if volume_spike {
        if obi > Decimal::ZERO {
            score += dec!(10);
        } else if obi < Decimal::ZERO {
            score -= dec!(5);
        }
    }

    score.max(Decimal::ZERO).min(dec!(100))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obi_calculation() {
        // Equal volume
        let obi = calculate_obi(1000, 1000);
        assert_eq!(obi.obi, Decimal::ZERO);
        assert_eq!(obi.interpretation, "neutral");

        // Strong buying
        let obi = calculate_obi(8000, 2000);
        assert!(obi.obi > dec!(0.2));
        assert_eq!(obi.interpretation, "strong_buying_pressure");

        // Strong selling
        let obi = calculate_obi(2000, 8000);
        assert!(obi.obi < dec!(-0.2));
        assert_eq!(obi.interpretation, "strong_selling_pressure");
    }

    #[test]
    fn test_obi_no_volume() {
        let obi = calculate_obi(0, 0);
        assert_eq!(obi.obi, Decimal::ZERO);
        assert_eq!(obi.interpretation, "no_volume");
    }

    #[test]
    fn test_obi_buying_pressure() {
        let obi = calculate_obi(600, 400);
        assert!(obi.obi > dec!(0.05));
        assert!(obi.obi <= dec!(0.2));
        assert_eq!(obi.interpretation, "buying_pressure");
    }

    #[test]
    fn test_obi_selling_pressure() {
        let obi = calculate_obi(400, 600);
        assert!(obi.obi < dec!(-0.05));
        assert!(obi.obi >= dec!(-0.2));
        assert_eq!(obi.interpretation, "selling_pressure");
    }

    #[test]
    fn test_obi_multilevel() {
        let bids = vec![(dec!(99), 1000), (dec!(98), 500), (dec!(97), 200)];
        let asks = vec![(dec!(101), 800), (dec!(102), 400), (dec!(103), 100)];
        let mid_price = dec!(100);

        let obi = calculate_obi_multilevel(&bids, &asks, mid_price, dec!(5));
        // Should have more bid weight, so positive
        assert!(obi.obi != Decimal::ZERO);
    }

    #[test]
    fn test_ofi_calculation() {
        let prev = OrderBookSnapshot {
            timestamp: 1,
            bid_price: dec!(100),
            bid_volume: 1000,
            ask_price: dec!(101),
            ask_volume: 1000,
        };

        // Bid price increased (bullish)
        let current = OrderBookSnapshot {
            timestamp: 2,
            bid_price: dec!(101),
            bid_volume: 1500,
            ask_price: dec!(102),
            ask_volume: 800,
        };

        let ofi = calculate_ofi(&prev, &current);
        assert!(ofi.ofi > Decimal::ZERO);
    }

    #[test]
    fn test_ofi_selling_pressure() {
        let prev = OrderBookSnapshot {
            timestamp: 1,
            bid_price: dec!(100),
            bid_volume: 1000,
            ask_price: dec!(101),
            ask_volume: 1000,
        };

        // Bid price decreased (bearish)
        let current = OrderBookSnapshot {
            timestamp: 2,
            bid_price: dec!(99),
            bid_volume: 500,
            ask_price: dec!(100),
            ask_volume: 1500,
        };

        let ofi = calculate_ofi(&prev, &current);
        assert!(ofi.ofi < Decimal::ZERO);
        assert_eq!(ofi.interpretation, "selling_pressure");
    }

    #[test]
    fn test_cumulative_ofi() {
        let snapshots = vec![
            OrderBookSnapshot {
                timestamp: 1,
                bid_price: dec!(100),
                bid_volume: 1000,
                ask_price: dec!(101),
                ask_volume: 1000,
            },
            OrderBookSnapshot {
                timestamp: 2,
                bid_price: dec!(101),
                bid_volume: 1200,
                ask_price: dec!(102),
                ask_volume: 900,
            },
            OrderBookSnapshot {
                timestamp: 3,
                bid_price: dec!(102),
                bid_volume: 1500,
                ask_price: dec!(103),
                ask_volume: 800,
            },
        ];

        let results = calculate_cumulative_ofi(&snapshots).unwrap();
        assert_eq!(results.len(), 3);
        // First result should be initial
        assert_eq!(results[0].interpretation, "initial");
        // Cumulative should be increasing
        assert!(results[2].cumulative_ofi >= results[1].cumulative_ofi);
    }

    #[test]
    fn test_cumulative_ofi_insufficient_data() {
        let snapshots = vec![OrderBookSnapshot {
            timestamp: 1,
            bid_price: dec!(100),
            bid_volume: 1000,
            ask_price: dec!(101),
            ask_volume: 1000,
        }];

        let result = calculate_cumulative_ofi(&snapshots);
        assert!(result.is_err());
    }

    #[test]
    fn test_vamp() {
        let vamp = calculate_vamp(dec!(100), 1000, dec!(102), 500);
        // More ask volume (500) means price weighted toward bid
        // More bid volume (1000) means price weighted toward ask
        assert!(vamp > dec!(100) && vamp < dec!(102));
    }

    #[test]
    fn test_vamp_equal_volume() {
        let vamp = calculate_vamp(dec!(100), 1000, dec!(102), 1000);
        // Equal volumes = mid price
        assert_eq!(vamp, dec!(101));
    }

    #[test]
    fn test_vamp_zero_volume() {
        let vamp = calculate_vamp(dec!(100), 0, dec!(102), 0);
        // No volume = simple mid
        assert_eq!(vamp, dec!(101));
    }

    #[test]
    fn test_volume_split() {
        let (buy, sell) = split_volume(dec!(110), dec!(100), dec!(108), 1000);
        // Close is 80% of the way up, so buy should be ~800
        assert!(buy > 700 && buy < 900);
        assert!(sell > 100 && sell < 300);
    }

    #[test]
    fn test_volume_split_no_range() {
        let (buy, sell) = split_volume(dec!(100), dec!(100), dec!(100), 1000);
        // No range - split evenly
        assert_eq!(buy, 500);
        assert_eq!(sell, 500);
    }

    #[test]
    fn test_money_flow_multiplier() {
        // Close at high
        let mfm = money_flow_multiplier(dec!(110), dec!(100), dec!(110));
        assert_eq!(mfm, dec!(1));

        // Close at low
        let mfm = money_flow_multiplier(dec!(110), dec!(100), dec!(100));
        assert_eq!(mfm, dec!(-1));

        // Close at mid
        let mfm = money_flow_multiplier(dec!(110), dec!(100), dec!(105));
        assert_eq!(mfm, dec!(0));
    }

    #[test]
    fn test_money_flow_multiplier_no_range() {
        let mfm = money_flow_multiplier(dec!(100), dec!(100), dec!(100));
        assert_eq!(mfm, Decimal::ZERO);
    }

    #[test]
    fn test_adl() {
        let highs = vec![dec!(110), dec!(112), dec!(111)];
        let lows = vec![dec!(100), dec!(102), dec!(105)];
        let closes = vec![dec!(108), dec!(110), dec!(109)];
        let volumes = vec![1000, 1500, 1200];

        let adl = calculate_adl(&highs, &lows, &closes, &volumes).unwrap();
        assert_eq!(adl.len(), 3);
    }

    #[test]
    fn test_adl_empty() {
        let highs: Vec<Decimal> = vec![];
        let lows: Vec<Decimal> = vec![];
        let closes: Vec<Decimal> = vec![];
        let volumes: Vec<i64> = vec![];

        let adl = calculate_adl(&highs, &lows, &closes, &volumes).unwrap();
        assert!(adl.is_empty());
    }

    #[test]
    fn test_adl_mismatched_lengths() {
        let highs = vec![dec!(110), dec!(112)];
        let lows = vec![dec!(100)];
        let closes = vec![dec!(108)];
        let volumes = vec![1000];

        let result = calculate_adl(&highs, &lows, &closes, &volumes);
        assert!(result.is_err());
    }

    #[test]
    fn test_order_flow_score() {
        // Strong buy signal
        let score = order_flow_score(dec!(0.5), dec!(0.8), true);
        assert!(score > dec!(70));

        // Strong sell signal
        let score = order_flow_score(dec!(-0.5), dec!(-0.8), false);
        assert!(score < dec!(30));

        // Neutral
        let score = order_flow_score(dec!(0), dec!(0), false);
        assert_eq!(score, dec!(50));
    }

    #[test]
    fn test_order_flow_score_clamping() {
        // Score should be clamped between 0 and 100
        let score = order_flow_score(dec!(1), dec!(1), true);
        assert!(score <= dec!(100));

        let score = order_flow_score(dec!(-1), dec!(-1), false);
        assert!(score >= dec!(0));
    }
}
