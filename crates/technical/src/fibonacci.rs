//! Fibonacci Retracement calculations

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Fibonacci retracement levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FibonacciLevels {
    pub high: Decimal,
    pub low: Decimal,
    pub level_0: Decimal,    // 0% (high)
    pub level_236: Decimal,  // 23.6%
    pub level_382: Decimal,  // 38.2%
    pub level_500: Decimal,  // 50%
    pub level_618: Decimal,  // 61.8%
    pub level_786: Decimal,  // 78.6%
    pub level_1000: Decimal, // 100% (low)
}

/// Calculate Fibonacci retracement levels from high and low
pub fn calculate_fibonacci_levels(high: Decimal, low: Decimal) -> FibonacciLevels {
    let diff = high - low;

    FibonacciLevels {
        high,
        low,
        level_0: high,
        level_236: high - (diff * dec!(0.236)),
        level_382: high - (diff * dec!(0.382)),
        level_500: high - (diff * dec!(0.5)),
        level_618: high - (diff * dec!(0.618)),
        level_786: high - (diff * dec!(0.786)),
        level_1000: low,
    }
}

/// Find the nearest Fibonacci level to current price
pub fn nearest_fibonacci_level(
    price: Decimal,
    levels: &FibonacciLevels,
) -> (Decimal, &'static str, Decimal) {
    let all_levels = [
        (levels.level_0, "0%"),
        (levels.level_236, "23.6%"),
        (levels.level_382, "38.2%"),
        (levels.level_500, "50%"),
        (levels.level_618, "61.8%"),
        (levels.level_786, "78.6%"),
        (levels.level_1000, "100%"),
    ];

    let mut nearest = all_levels[0];
    let mut min_distance = (price - all_levels[0].0).abs();

    for level in &all_levels[1..] {
        let distance = (price - level.0).abs();
        if distance < min_distance {
            min_distance = distance;
            nearest = *level;
        }
    }

    (nearest.0, nearest.1, min_distance)
}

/// Calculate Fibonacci score based on proximity to key levels (38.2%, 50%, 61.8%)
/// Returns score 0-100 where 100 means price is exactly at a key support level
pub fn fibonacci_support_score(price: Decimal, levels: &FibonacciLevels) -> Decimal {
    let key_levels = [levels.level_382, levels.level_500, levels.level_618];
    let range = levels.high - levels.low;

    if range == Decimal::ZERO {
        return dec!(50);
    }

    // Find minimum distance to any key level
    let min_distance = key_levels
        .iter()
        .map(|level| (price - *level).abs())
        .min()
        .unwrap_or(range);

    // Score based on proximity (closer = higher score)
    // If within 2% of range, score is 80-100
    // If within 5% of range, score is 60-80
    // Otherwise, lower score
    let proximity_pct = min_distance / range * dec!(100);

    if proximity_pct <= dec!(2) {
        dec!(100) - (proximity_pct * dec!(10))
    } else if proximity_pct <= dec!(5) {
        dec!(80) - ((proximity_pct - dec!(2)) * dec!(6.67))
    } else if proximity_pct <= dec!(10) {
        dec!(60) - ((proximity_pct - dec!(5)) * dec!(4))
    } else {
        dec!(40) - (proximity_pct - dec!(10)).min(dec!(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_levels() {
        let levels = calculate_fibonacci_levels(dec!(100), dec!(50));

        assert_eq!(levels.level_0, dec!(100));
        assert_eq!(levels.level_500, dec!(75));
        assert_eq!(levels.level_1000, dec!(50));
    }

    #[test]
    fn test_nearest_level() {
        let levels = calculate_fibonacci_levels(dec!(100), dec!(50));
        let (level, name, _) = nearest_fibonacci_level(dec!(74), &levels);

        assert_eq!(name, "50%");
        assert_eq!(level, dec!(75));
    }

    #[test]
    fn test_fibonacci_support_score_at_key_level() {
        let levels = calculate_fibonacci_levels(dec!(100), dec!(50));
        // Price exactly at 50% level (75)
        let score = fibonacci_support_score(dec!(75), &levels);
        assert_eq!(score, dec!(100));
    }

    #[test]
    fn test_fibonacci_support_score_away_from_key_levels() {
        let levels = calculate_fibonacci_levels(dec!(100), dec!(50));
        // Price at the high (far from key levels)
        let score = fibonacci_support_score(dec!(100), &levels);
        assert!(score < dec!(60));
    }

    #[test]
    fn test_fibonacci_zero_range() {
        let levels = calculate_fibonacci_levels(dec!(100), dec!(100));
        let score = fibonacci_support_score(dec!(100), &levels);
        assert_eq!(score, dec!(50));
    }
}
