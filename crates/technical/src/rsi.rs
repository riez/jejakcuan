//! Relative Strength Index (RSI) calculations

use crate::error::TechnicalError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Calculate RSI for a series of prices
///
/// RSI = 100 - (100 / (1 + RS))
/// RS = Average Gain / Average Loss
pub fn calculate_rsi(prices: &[Decimal], period: usize) -> Result<Vec<Decimal>, TechnicalError> {
    if prices.len() < period + 1 {
        return Err(TechnicalError::InsufficientData {
            required: period + 1,
            actual: prices.len(),
        });
    }

    let mut rsi_values = vec![Decimal::ZERO; period];

    // Calculate price changes
    let changes: Vec<Decimal> = prices.windows(2).map(|w| w[1] - w[0]).collect();

    // Initial averages
    let mut avg_gain = changes[..period]
        .iter()
        .filter(|c| **c > Decimal::ZERO)
        .sum::<Decimal>()
        / Decimal::from(period as i64);

    let mut avg_loss = changes[..period]
        .iter()
        .filter(|c| **c < Decimal::ZERO)
        .map(|c| c.abs())
        .sum::<Decimal>()
        / Decimal::from(period as i64);

    // First RSI
    let rs = if avg_loss == Decimal::ZERO {
        dec!(100)
    } else {
        avg_gain / avg_loss
    };
    rsi_values.push(dec!(100) - (dec!(100) / (dec!(1) + rs)));

    // Calculate remaining RSI values using smoothed averages
    for change in changes.iter().skip(period) {
        let gain = if *change > Decimal::ZERO {
            *change
        } else {
            Decimal::ZERO
        };
        let loss = if *change < Decimal::ZERO {
            change.abs()
        } else {
            Decimal::ZERO
        };

        avg_gain =
            (avg_gain * Decimal::from(period as i64 - 1) + gain) / Decimal::from(period as i64);
        avg_loss =
            (avg_loss * Decimal::from(period as i64 - 1) + loss) / Decimal::from(period as i64);

        let rs = if avg_loss == Decimal::ZERO {
            dec!(100)
        } else {
            avg_gain / avg_loss
        };

        rsi_values.push(dec!(100) - (dec!(100) / (dec!(1) + rs)));
    }

    Ok(rsi_values)
}

/// Calculate RSI 14 (standard period)
pub fn calculate_rsi14(prices: &[Decimal]) -> Result<Vec<Decimal>, TechnicalError> {
    calculate_rsi(prices, 14)
}

/// Interpret RSI value
pub fn rsi_signal(rsi: Decimal) -> &'static str {
    if rsi >= dec!(70) {
        "overbought"
    } else if rsi <= dec!(30) {
        "oversold"
    } else {
        "neutral"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_bounds() {
        // RSI should always be between 0 and 100
        let prices: Vec<Decimal> = (0..30).map(|i| Decimal::from(100 + i)).collect();

        let rsi = calculate_rsi(&prices, 14).unwrap();

        for value in rsi.iter().skip(14) {
            assert!(*value >= Decimal::ZERO);
            assert!(*value <= dec!(100));
        }
    }

    #[test]
    fn test_rsi_insufficient_data() {
        let prices = vec![dec!(100), dec!(102), dec!(104)];
        let result = calculate_rsi(&prices, 14);
        assert!(result.is_err());
    }

    #[test]
    fn test_rsi_signal_overbought() {
        assert_eq!(rsi_signal(dec!(75)), "overbought");
        assert_eq!(rsi_signal(dec!(70)), "overbought");
    }

    #[test]
    fn test_rsi_signal_oversold() {
        assert_eq!(rsi_signal(dec!(25)), "oversold");
        assert_eq!(rsi_signal(dec!(30)), "oversold");
    }

    #[test]
    fn test_rsi_signal_neutral() {
        assert_eq!(rsi_signal(dec!(50)), "neutral");
        assert_eq!(rsi_signal(dec!(45)), "neutral");
    }

    #[test]
    fn test_rsi14() {
        let prices: Vec<Decimal> = (0..30).map(|i| Decimal::from(100 + i)).collect();
        let result = calculate_rsi14(&prices);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 30);
    }
}
