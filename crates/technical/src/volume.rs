//! Volume-based indicators (OBV, VPT)

use crate::error::TechnicalError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Calculate On-Balance Volume (OBV)
/// OBV adds volume on up days, subtracts on down days
pub fn calculate_obv(prices: &[Decimal], volumes: &[i64]) -> Result<Vec<i64>, TechnicalError> {
    if prices.len() != volumes.len() {
        return Err(TechnicalError::CalculationError(
            "Prices and volumes must have same length".to_string(),
        ));
    }

    if prices.len() < 2 {
        return Err(TechnicalError::InsufficientData {
            required: 2,
            actual: prices.len(),
        });
    }

    let mut obv = vec![volumes[0]];

    for i in 1..prices.len() {
        let prev_obv = obv[i - 1];
        let new_obv = if prices[i] > prices[i - 1] {
            prev_obv + volumes[i]
        } else if prices[i] < prices[i - 1] {
            prev_obv - volumes[i]
        } else {
            prev_obv
        };
        obv.push(new_obv);
    }

    Ok(obv)
}

/// Calculate Volume Price Trend (VPT)
/// VPT = Previous VPT + Volume × ((Close - Previous Close) / Previous Close)
pub fn calculate_vpt(prices: &[Decimal], volumes: &[i64]) -> Result<Vec<Decimal>, TechnicalError> {
    if prices.len() != volumes.len() {
        return Err(TechnicalError::CalculationError(
            "Prices and volumes must have same length".to_string(),
        ));
    }

    if prices.len() < 2 {
        return Err(TechnicalError::InsufficientData {
            required: 2,
            actual: prices.len(),
        });
    }

    let mut vpt = vec![Decimal::ZERO];

    for i in 1..prices.len() {
        let prev_vpt = vpt[i - 1];
        let price_change = if prices[i - 1] != Decimal::ZERO {
            (prices[i] - prices[i - 1]) / prices[i - 1]
        } else {
            Decimal::ZERO
        };
        let volume = Decimal::from(volumes[i]);
        vpt.push(prev_vpt + (volume * price_change));
    }

    Ok(vpt)
}

/// Calculate Relative Volume (RVOL)
/// RVOL = Current Volume / Average Volume of previous `period` values
pub fn calculate_rvol(volumes: &[i64], period: usize) -> Result<Vec<Decimal>, TechnicalError> {
    if volumes.len() < period + 1 {
        return Err(TechnicalError::InsufficientData {
            required: period + 1,
            actual: volumes.len(),
        });
    }

    let mut rvol = vec![Decimal::ZERO; period];

    for i in period..volumes.len() {
        let avg: i64 = volumes[i - period..i].iter().sum::<i64>() / period as i64;
        let current = volumes[i];

        if avg > 0 {
            rvol.push(Decimal::from(current) / Decimal::from(avg));
        } else {
            rvol.push(dec!(1));
        }
    }

    Ok(rvol)
}

/// Detect volume spike (RVOL > threshold)
pub fn is_volume_spike(rvol: Decimal, threshold: Decimal) -> bool {
    rvol > threshold
}

/// OBV divergence detection
/// Returns positive if OBV rising while price falling (bullish divergence)
/// Returns negative if OBV falling while price rising (bearish divergence)
pub fn obv_divergence(prices: &[Decimal], obv: &[i64], lookback: usize) -> Option<&'static str> {
    if prices.len() < lookback + 1 || obv.len() < lookback + 1 {
        return None;
    }

    let price_change = prices[prices.len() - 1] - prices[prices.len() - 1 - lookback];
    let obv_change = obv[obv.len() - 1] - obv[obv.len() - 1 - lookback];

    if price_change < Decimal::ZERO && obv_change > 0 {
        Some("bullish_divergence")
    } else if price_change > Decimal::ZERO && obv_change < 0 {
        Some("bearish_divergence")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obv() {
        let prices = vec![dec!(100), dec!(102), dec!(101), dec!(103), dec!(105)];
        let volumes = vec![1000, 1200, 800, 1500, 2000];

        let obv = calculate_obv(&prices, &volumes).unwrap();

        assert_eq!(obv.len(), 5);
        assert_eq!(obv[0], 1000); // Initial
        assert_eq!(obv[1], 2200); // +1200 (up day)
        assert_eq!(obv[2], 1400); // -800 (down day)
        assert_eq!(obv[3], 2900); // +1500 (up day)
        assert_eq!(obv[4], 4900); // +2000 (up day)
    }

    #[test]
    fn test_obv_flat_price() {
        let prices = vec![dec!(100), dec!(100), dec!(100)];
        let volumes = vec![1000, 1200, 800];

        let obv = calculate_obv(&prices, &volumes).unwrap();

        assert_eq!(obv[0], 1000);
        assert_eq!(obv[1], 1000); // Unchanged
        assert_eq!(obv[2], 1000); // Unchanged
    }

    #[test]
    fn test_obv_mismatched_lengths() {
        let prices = vec![dec!(100), dec!(102)];
        let volumes = vec![1000];
        let result = calculate_obv(&prices, &volumes);
        assert!(result.is_err());
    }

    #[test]
    fn test_obv_insufficient_data() {
        let prices = vec![dec!(100)];
        let volumes = vec![1000];
        let result = calculate_obv(&prices, &volumes);
        assert!(result.is_err());
    }

    #[test]
    fn test_vpt() {
        let prices = vec![dec!(100), dec!(110), dec!(105)];
        let volumes = vec![1000, 2000, 1500];

        let vpt = calculate_vpt(&prices, &volumes).unwrap();

        assert_eq!(vpt.len(), 3);
        assert_eq!(vpt[0], Decimal::ZERO);
        // VPT[1] = 0 + 2000 * ((110-100)/100) = 200
        assert_eq!(vpt[1], dec!(200));
        // VPT[2] = 200 + 1500 * ((105-110)/110) = 200 - 68.18... ≈ 131.82
        assert!(vpt[2] < vpt[1]); // Should decrease
    }

    #[test]
    fn test_vpt_zero_price() {
        let prices = vec![dec!(0), dec!(100)];
        let volumes = vec![1000, 2000];

        let vpt = calculate_vpt(&prices, &volumes).unwrap();
        assert_eq!(vpt[0], Decimal::ZERO);
        assert_eq!(vpt[1], Decimal::ZERO); // Division by zero handled
    }

    #[test]
    fn test_rvol() {
        let volumes = vec![1000, 1000, 1000, 1000, 2000];
        let rvol = calculate_rvol(&volumes, 4).unwrap();

        // Last RVOL should be 2.0 (2000 / 1000 avg)
        assert_eq!(rvol.len(), 5);
        assert_eq!(rvol[4], dec!(2));
    }

    #[test]
    fn test_rvol_insufficient_data() {
        let volumes = vec![1000, 2000];
        let result = calculate_rvol(&volumes, 4);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_volume_spike() {
        assert!(is_volume_spike(dec!(2.5), dec!(2)));
        assert!(!is_volume_spike(dec!(1.5), dec!(2)));
        assert!(!is_volume_spike(dec!(2), dec!(2)));
    }

    #[test]
    fn test_obv_divergence_bullish() {
        // Price falling but OBV rising = bullish divergence
        let prices = vec![dec!(100), dec!(98), dec!(95)];
        let obv = vec![1000, 1200, 1500];

        let result = obv_divergence(&prices, &obv, 2);
        assert_eq!(result, Some("bullish_divergence"));
    }

    #[test]
    fn test_obv_divergence_bearish() {
        // Price rising but OBV falling = bearish divergence
        let prices = vec![dec!(100), dec!(102), dec!(105)];
        let obv = vec![1000, 800, 600];

        let result = obv_divergence(&prices, &obv, 2);
        assert_eq!(result, Some("bearish_divergence"));
    }

    #[test]
    fn test_obv_divergence_none() {
        // Price and OBV moving in same direction = no divergence
        let prices = vec![dec!(100), dec!(102), dec!(105)];
        let obv = vec![1000, 1200, 1500];

        let result = obv_divergence(&prices, &obv, 2);
        assert!(result.is_none());
    }

    #[test]
    fn test_obv_divergence_insufficient_data() {
        let prices = vec![dec!(100)];
        let obv = vec![1000];

        let result = obv_divergence(&prices, &obv, 2);
        assert!(result.is_none());
    }
}
