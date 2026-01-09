//! Exponential Moving Average (EMA) calculations

use crate::error::TechnicalError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Calculate EMA for a series of prices
///
/// EMA = Price(t) × k + EMA(y) × (1 − k)
/// where k = 2 / (N + 1), N = period
pub fn calculate_ema(prices: &[Decimal], period: usize) -> Result<Vec<Decimal>, TechnicalError> {
    if prices.len() < period {
        return Err(TechnicalError::InsufficientData {
            required: period,
            actual: prices.len(),
        });
    }

    if period == 0 {
        return Err(TechnicalError::InvalidPeriod(
            "Period must be > 0".to_string(),
        ));
    }

    let k = Decimal::from(2) / Decimal::from(period as i64 + 1);
    let one_minus_k = dec!(1) - k;

    let mut ema_values = Vec::with_capacity(prices.len());

    // First EMA is SMA of first `period` values
    let initial_sma: Decimal =
        prices[..period].iter().sum::<Decimal>() / Decimal::from(period as i64);

    // Fill with None for periods before we have enough data
    for _ in 0..period - 1 {
        ema_values.push(Decimal::ZERO);
    }

    ema_values.push(initial_sma);

    // Calculate EMA for remaining prices
    for price in prices.iter().skip(period) {
        let prev_ema = *ema_values.last().unwrap();
        let new_ema = (*price * k) + (prev_ema * one_minus_k);
        ema_values.push(new_ema);
    }

    Ok(ema_values)
}

/// Calculate EMA 20 (commonly used)
pub fn calculate_ema20(prices: &[Decimal]) -> Result<Vec<Decimal>, TechnicalError> {
    calculate_ema(prices, 20)
}

/// Calculate EMA 50
pub fn calculate_ema50(prices: &[Decimal]) -> Result<Vec<Decimal>, TechnicalError> {
    calculate_ema(prices, 50)
}

/// Calculate EMA 200
pub fn calculate_ema200(prices: &[Decimal]) -> Result<Vec<Decimal>, TechnicalError> {
    calculate_ema(prices, 200)
}

/// Check if price is above EMA
pub fn is_price_above_ema(price: Decimal, ema: Decimal) -> bool {
    price > ema
}

/// Calculate EMA slope (positive = uptrend)
pub fn ema_slope(ema_values: &[Decimal], lookback: usize) -> Option<Decimal> {
    if ema_values.len() < lookback + 1 {
        return None;
    }

    let current = ema_values[ema_values.len() - 1];
    let previous = ema_values[ema_values.len() - 1 - lookback];

    if previous == Decimal::ZERO {
        return None;
    }

    Some((current - previous) / previous * dec!(100))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_ema_calculation() {
        let prices = vec![
            dec!(100),
            dec!(102),
            dec!(104),
            dec!(103),
            dec!(105),
            dec!(107),
            dec!(106),
            dec!(108),
            dec!(110),
            dec!(109),
        ];

        let ema = calculate_ema(&prices, 5).unwrap();
        assert_eq!(ema.len(), prices.len());

        // First 4 values should be zero (not enough data)
        assert_eq!(ema[0], Decimal::ZERO);
        assert_eq!(ema[3], Decimal::ZERO);

        // 5th value should be SMA of first 5
        let expected_sma = (dec!(100) + dec!(102) + dec!(104) + dec!(103) + dec!(105)) / dec!(5);
        assert_eq!(ema[4], expected_sma);
    }

    #[test]
    fn test_insufficient_data() {
        let prices = vec![dec!(100), dec!(102)];
        let result = calculate_ema(&prices, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_period() {
        let prices = vec![dec!(100), dec!(102), dec!(104)];
        let result = calculate_ema(&prices, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_price_above_ema() {
        assert!(is_price_above_ema(dec!(105), dec!(100)));
        assert!(!is_price_above_ema(dec!(95), dec!(100)));
        assert!(!is_price_above_ema(dec!(100), dec!(100)));
    }

    #[test]
    fn test_ema_slope() {
        let ema_values = vec![dec!(100), dec!(102), dec!(105), dec!(108), dec!(110)];
        let slope = ema_slope(&ema_values, 2);
        assert!(slope.is_some());
        // Slope should be positive (uptrend)
        assert!(slope.unwrap() > Decimal::ZERO);
    }

    #[test]
    fn test_ema_slope_insufficient_data() {
        let ema_values = vec![dec!(100)];
        let slope = ema_slope(&ema_values, 2);
        assert!(slope.is_none());
    }
}
