//! MACD (Moving Average Convergence Divergence) calculations

use crate::{calculate_ema, error::TechnicalError};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// MACD result containing line, signal, and histogram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacdResult {
    pub macd_line: Vec<Decimal>,
    pub signal_line: Vec<Decimal>,
    pub histogram: Vec<Decimal>,
}

/// Calculate MACD with default parameters (12, 26, 9)
pub fn calculate_macd(prices: &[Decimal]) -> Result<MacdResult, TechnicalError> {
    calculate_macd_custom(prices, 12, 26, 9)
}

/// Calculate MACD with custom parameters
pub fn calculate_macd_custom(
    prices: &[Decimal],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Result<MacdResult, TechnicalError> {
    if prices.len() < slow_period + signal_period {
        return Err(TechnicalError::InsufficientData {
            required: slow_period + signal_period,
            actual: prices.len(),
        });
    }

    let ema_fast = calculate_ema(prices, fast_period)?;
    let ema_slow = calculate_ema(prices, slow_period)?;

    // MACD line = EMA(fast) - EMA(slow)
    let macd_line: Vec<Decimal> = ema_fast
        .iter()
        .zip(ema_slow.iter())
        .map(|(f, s)| *f - *s)
        .collect();

    // Signal line = EMA of MACD line
    let signal_line = calculate_ema(&macd_line, signal_period)?;

    // Histogram = MACD line - Signal line
    let histogram: Vec<Decimal> = macd_line
        .iter()
        .zip(signal_line.iter())
        .map(|(m, s)| *m - *s)
        .collect();

    Ok(MacdResult {
        macd_line,
        signal_line,
        histogram,
    })
}

/// Interpret MACD signal
pub fn macd_signal(macd: &MacdResult) -> &'static str {
    if macd.histogram.is_empty() {
        return "neutral";
    }

    let last_hist = macd.histogram.last().unwrap();
    let prev_hist = macd.histogram.get(macd.histogram.len().saturating_sub(2));

    match prev_hist {
        Some(prev) => {
            if *last_hist > Decimal::ZERO && *prev <= Decimal::ZERO {
                "bullish_crossover"
            } else if *last_hist < Decimal::ZERO && *prev >= Decimal::ZERO {
                "bearish_crossover"
            } else if *last_hist > Decimal::ZERO {
                "bullish"
            } else {
                "bearish"
            }
        }
        None => {
            if *last_hist > Decimal::ZERO {
                "bullish"
            } else {
                "bearish"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_macd_calculation() {
        let prices: Vec<Decimal> = (0..50).map(|i| Decimal::from(100 + i % 10)).collect();

        let macd = calculate_macd(&prices).unwrap();

        assert_eq!(macd.macd_line.len(), prices.len());
        assert_eq!(macd.signal_line.len(), prices.len());
        assert_eq!(macd.histogram.len(), prices.len());
    }

    #[test]
    fn test_macd_insufficient_data() {
        let prices: Vec<Decimal> = (0..20).map(|i| Decimal::from(100 + i)).collect();
        let result = calculate_macd(&prices);
        assert!(result.is_err());
    }

    #[test]
    fn test_macd_signal_bullish() {
        let macd = MacdResult {
            macd_line: vec![dec!(1)],
            signal_line: vec![dec!(0)],
            histogram: vec![dec!(1)],
        };
        assert_eq!(macd_signal(&macd), "bullish");
    }

    #[test]
    fn test_macd_signal_bearish() {
        let macd = MacdResult {
            macd_line: vec![dec!(-1)],
            signal_line: vec![dec!(0)],
            histogram: vec![dec!(-1)],
        };
        assert_eq!(macd_signal(&macd), "bearish");
    }

    #[test]
    fn test_macd_signal_bullish_crossover() {
        let macd = MacdResult {
            macd_line: vec![dec!(-1), dec!(1)],
            signal_line: vec![dec!(0), dec!(0)],
            histogram: vec![dec!(-1), dec!(1)],
        };
        assert_eq!(macd_signal(&macd), "bullish_crossover");
    }

    #[test]
    fn test_macd_signal_bearish_crossover() {
        let macd = MacdResult {
            macd_line: vec![dec!(1), dec!(-1)],
            signal_line: vec![dec!(0), dec!(0)],
            histogram: vec![dec!(1), dec!(-1)],
        };
        assert_eq!(macd_signal(&macd), "bearish_crossover");
    }

    #[test]
    fn test_macd_signal_empty() {
        let macd = MacdResult {
            macd_line: vec![],
            signal_line: vec![],
            histogram: vec![],
        };
        assert_eq!(macd_signal(&macd), "neutral");
    }
}
