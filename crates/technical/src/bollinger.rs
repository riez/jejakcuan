//! Bollinger Bands calculations

use crate::error::TechnicalError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Bollinger Bands result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BollingerBands {
    pub upper: Vec<Decimal>,
    pub middle: Vec<Decimal>,
    pub lower: Vec<Decimal>,
}

/// Calculate Bollinger Bands (default: 20 period, 2 std dev)
pub fn calculate_bollinger_bands(prices: &[Decimal]) -> Result<BollingerBands, TechnicalError> {
    calculate_bollinger_bands_custom(prices, 20, dec!(2))
}

/// Calculate Bollinger Bands with custom parameters
pub fn calculate_bollinger_bands_custom(
    prices: &[Decimal],
    period: usize,
    num_std_dev: Decimal,
) -> Result<BollingerBands, TechnicalError> {
    if prices.len() < period {
        return Err(TechnicalError::InsufficientData {
            required: period,
            actual: prices.len(),
        });
    }

    let mut upper = vec![Decimal::ZERO; period - 1];
    let mut middle = vec![Decimal::ZERO; period - 1];
    let mut lower = vec![Decimal::ZERO; period - 1];

    for i in (period - 1)..prices.len() {
        let window = &prices[i + 1 - period..=i];

        // Calculate SMA (middle band)
        let sma: Decimal = window.iter().sum::<Decimal>() / Decimal::from(period as i64);

        // Calculate standard deviation
        let variance: Decimal = window
            .iter()
            .map(|p| (*p - sma) * (*p - sma))
            .sum::<Decimal>()
            / Decimal::from(period as i64);

        // Approximate square root using Newton's method
        let std_dev = sqrt_decimal(variance);

        middle.push(sma);
        upper.push(sma + (std_dev * num_std_dev));
        lower.push(sma - (std_dev * num_std_dev));
    }

    Ok(BollingerBands {
        upper,
        middle,
        lower,
    })
}

/// Approximate square root for Decimal using Newton's method
fn sqrt_decimal(n: Decimal) -> Decimal {
    if n <= Decimal::ZERO {
        return Decimal::ZERO;
    }

    let mut x = n;
    let two = dec!(2);

    for _ in 0..20 {
        let next = (x + n / x) / two;
        if (next - x).abs() < dec!(0.0000001) {
            return next;
        }
        x = next;
    }

    x
}

/// Calculate %B (where price is relative to bands)
/// %B = (Price - Lower) / (Upper - Lower)
pub fn percent_b(price: Decimal, upper: Decimal, lower: Decimal) -> Decimal {
    let range = upper - lower;
    if range == Decimal::ZERO {
        return dec!(0.5);
    }
    (price - lower) / range
}

/// Interpret Bollinger Band position
pub fn bollinger_signal(price: Decimal, bands: &BollingerBands) -> &'static str {
    if bands.upper.is_empty() {
        return "neutral";
    }

    let upper = *bands.upper.last().unwrap();
    let lower = *bands.lower.last().unwrap();

    if price >= upper {
        "overbought"
    } else if price <= lower {
        "oversold"
    } else {
        "neutral"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bollinger_bands() {
        let prices: Vec<Decimal> = (0..30).map(|i| Decimal::from(100 + (i % 5))).collect();

        let bb = calculate_bollinger_bands(&prices).unwrap();

        assert_eq!(bb.upper.len(), prices.len());
        assert_eq!(bb.middle.len(), prices.len());
        assert_eq!(bb.lower.len(), prices.len());

        // Upper should be > middle > lower
        for i in 19..prices.len() {
            assert!(bb.upper[i] >= bb.middle[i]);
            assert!(bb.middle[i] >= bb.lower[i]);
        }
    }

    #[test]
    fn test_bollinger_insufficient_data() {
        let prices = vec![dec!(100), dec!(102), dec!(104)];
        let result = calculate_bollinger_bands(&prices);
        assert!(result.is_err());
    }

    #[test]
    fn test_sqrt_decimal() {
        let result = sqrt_decimal(dec!(4));
        assert!((result - dec!(2)).abs() < dec!(0.0001));

        let result = sqrt_decimal(dec!(9));
        assert!((result - dec!(3)).abs() < dec!(0.0001));

        let result = sqrt_decimal(dec!(0));
        assert_eq!(result, Decimal::ZERO);
    }

    #[test]
    fn test_percent_b() {
        // Price at middle should be 0.5
        let pct = percent_b(dec!(100), dec!(110), dec!(90));
        assert_eq!(pct, dec!(0.5));

        // Price at upper should be 1.0
        let pct = percent_b(dec!(110), dec!(110), dec!(90));
        assert_eq!(pct, dec!(1));

        // Price at lower should be 0.0
        let pct = percent_b(dec!(90), dec!(110), dec!(90));
        assert_eq!(pct, Decimal::ZERO);
    }

    #[test]
    fn test_percent_b_zero_range() {
        let pct = percent_b(dec!(100), dec!(100), dec!(100));
        assert_eq!(pct, dec!(0.5));
    }

    #[test]
    fn test_bollinger_signal_overbought() {
        let bands = BollingerBands {
            upper: vec![dec!(110)],
            middle: vec![dec!(100)],
            lower: vec![dec!(90)],
        };
        assert_eq!(bollinger_signal(dec!(115), &bands), "overbought");
        assert_eq!(bollinger_signal(dec!(110), &bands), "overbought");
    }

    #[test]
    fn test_bollinger_signal_oversold() {
        let bands = BollingerBands {
            upper: vec![dec!(110)],
            middle: vec![dec!(100)],
            lower: vec![dec!(90)],
        };
        assert_eq!(bollinger_signal(dec!(85), &bands), "oversold");
        assert_eq!(bollinger_signal(dec!(90), &bands), "oversold");
    }

    #[test]
    fn test_bollinger_signal_neutral() {
        let bands = BollingerBands {
            upper: vec![dec!(110)],
            middle: vec![dec!(100)],
            lower: vec![dec!(90)],
        };
        assert_eq!(bollinger_signal(dec!(100), &bands), "neutral");
    }

    #[test]
    fn test_bollinger_signal_empty() {
        let bands = BollingerBands {
            upper: vec![],
            middle: vec![],
            lower: vec![],
        };
        assert_eq!(bollinger_signal(dec!(100), &bands), "neutral");
    }
}
