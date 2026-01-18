//! Broker flow analysis for institutional tracking
//!
//! Implements rolling accumulation detection and institutional flow analysis:
//! - Rolling 5-day and 20-day net position calculation
//! - Coordinated institutional buying detection
//! - Accumulation persistence tracking

use super::classification::{get_broker_category, is_foreign_broker, is_institutional_broker};
use super::models::{BrokerAccumulationScore, BrokerCategory, BrokerSummary};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

/// Rolling accumulation window sizes
pub const WINDOW_5_DAYS: usize = 5;
pub const WINDOW_20_DAYS: usize = 20;

/// Threshold for coordinated activity detection
pub const COORDINATED_BROKER_THRESHOLD: usize = 3;

/// Rolling accumulation analysis result
#[derive(Debug, Clone)]
pub struct RollingAccumulation {
    pub symbol: String,
    pub date: NaiveDate,
    pub net_5_day: Decimal,
    pub net_20_day: Decimal,
    pub institutional_net_5_day: Decimal,
    pub institutional_net_20_day: Decimal,
    pub foreign_net_5_day: Decimal,
    pub foreign_net_20_day: Decimal,
    pub accumulation_score: Decimal,
    pub days_accumulated: i32,
    pub is_accumulating: bool,
    pub coordinated_buying: bool,
}

/// Broker position tracking
#[derive(Debug, Clone)]
pub struct BrokerPosition {
    pub broker_code: String,
    pub category: BrokerCategory,
    pub net_value: Decimal,
    pub net_volume: i64,
    pub is_buyer: bool,
}

/// Calculate rolling accumulation metrics from historical broker data
pub fn calculate_rolling_accumulation(
    summaries: &[BrokerSummary],
    window_size: usize,
) -> Option<RollingAccumulation> {
    if summaries.is_empty() {
        return None;
    }

    // Group summaries by date
    let mut by_date: HashMap<NaiveDate, Vec<&BrokerSummary>> = HashMap::new();
    for summary in summaries {
        by_date.entry(summary.date).or_default().push(summary);
    }

    // Sort dates
    let mut dates: Vec<_> = by_date.keys().cloned().collect();
    dates.sort();

    if dates.is_empty() {
        return None;
    }

    let symbol = summaries[0].symbol.clone();
    let latest_date = *dates.last()?;

    // Calculate rolling metrics
    let window_dates: Vec<_> = dates.iter().rev().take(window_size).cloned().collect();

    let mut net_value = Decimal::ZERO;
    let mut institutional_net = Decimal::ZERO;
    let mut foreign_net = Decimal::ZERO;
    let mut days_positive = 0i32;

    for date in &window_dates {
        if let Some(day_summaries) = by_date.get(date) {
            let mut day_net = Decimal::ZERO;
            let mut day_inst_net = Decimal::ZERO;
            let mut day_foreign_net = Decimal::ZERO;

            for summary in day_summaries {
                day_net += summary.net_value;

                let category = get_broker_category(&summary.broker_code);
                if is_institutional_broker(&summary.broker_code) {
                    day_inst_net += summary.net_value * category.weight();
                }

                if is_foreign_broker(&summary.broker_code) {
                    day_foreign_net += summary.net_value;
                }
            }

            net_value += day_net;
            institutional_net += day_inst_net;
            foreign_net += day_foreign_net;

            if day_inst_net > Decimal::ZERO {
                days_positive += 1;
            }
        }
    }

    // Detect coordinated buying
    let coordinated = detect_coordinated_buying(&window_dates, &by_date);

    // Calculate accumulation score (0-100)
    let score = calculate_accumulation_score_internal(
        institutional_net,
        foreign_net,
        days_positive,
        window_size as i32,
        coordinated,
    );

    Some(RollingAccumulation {
        symbol,
        date: latest_date,
        net_5_day: if window_size == 5 {
            net_value
        } else {
            Decimal::ZERO
        },
        net_20_day: if window_size == 20 {
            net_value
        } else {
            Decimal::ZERO
        },
        institutional_net_5_day: if window_size == 5 {
            institutional_net
        } else {
            Decimal::ZERO
        },
        institutional_net_20_day: if window_size == 20 {
            institutional_net
        } else {
            Decimal::ZERO
        },
        foreign_net_5_day: if window_size == 5 {
            foreign_net
        } else {
            Decimal::ZERO
        },
        foreign_net_20_day: if window_size == 20 {
            foreign_net
        } else {
            Decimal::ZERO
        },
        accumulation_score: score,
        days_accumulated: days_positive,
        is_accumulating: score > dec!(60) && days_positive >= (window_size as i32 / 2),
        coordinated_buying: coordinated,
    })
}

/// Calculate both 5-day and 20-day rolling accumulation
pub fn calculate_dual_window_accumulation(
    summaries: &[BrokerSummary],
) -> (Option<RollingAccumulation>, Option<RollingAccumulation>) {
    let window_5 = calculate_rolling_accumulation(summaries, WINDOW_5_DAYS);
    let window_20 = calculate_rolling_accumulation(summaries, WINDOW_20_DAYS);
    (window_5, window_20)
}

/// Detect coordinated buying by multiple institutional brokers
fn detect_coordinated_buying(
    dates: &[NaiveDate],
    by_date: &HashMap<NaiveDate, Vec<&BrokerSummary>>,
) -> bool {
    // Track institutional brokers with net buying across the period
    let mut institutional_buyers: HashMap<String, i32> = HashMap::new();

    for date in dates {
        if let Some(day_summaries) = by_date.get(date) {
            for summary in day_summaries {
                if is_institutional_broker(&summary.broker_code)
                    && summary.net_value > Decimal::ZERO
                {
                    *institutional_buyers
                        .entry(summary.broker_code.clone())
                        .or_default() += 1;
                }
            }
        }
    }

    // Count brokers that are consistently buying (appeared in at least half the days)
    let min_days = dates.len() / 2;
    let consistent_buyers: usize = institutional_buyers
        .values()
        .filter(|&&days| days as usize >= min_days.max(1))
        .count();

    consistent_buyers >= COORDINATED_BROKER_THRESHOLD
}

/// Calculate internal accumulation score
fn calculate_accumulation_score_internal(
    institutional_net: Decimal,
    foreign_net: Decimal,
    days_positive: i32,
    window_size: i32,
    coordinated: bool,
) -> Decimal {
    let mut score = dec!(50); // Base score

    // Institutional net contribution (-25 to +25)
    if institutional_net > Decimal::ZERO {
        score += dec!(25);
    } else if institutional_net < Decimal::ZERO {
        score -= dec!(15);
    }

    // Foreign flow contribution (-15 to +15)
    if foreign_net > Decimal::ZERO {
        score += dec!(15);
    } else if foreign_net < Decimal::ZERO {
        score -= dec!(10);
    }

    // Consistency contribution (0 to +15)
    if window_size > 0 {
        let consistency_ratio = Decimal::from(days_positive) / Decimal::from(window_size);
        score += consistency_ratio * dec!(15);
    }

    // Coordinated buying bonus (+10)
    if coordinated {
        score += dec!(10);
    }

    // Clamp to 0-100
    score.max(Decimal::ZERO).min(dec!(100))
}

/// Get broker positions aggregated by broker code
pub fn aggregate_broker_positions(summaries: &[BrokerSummary]) -> Vec<BrokerPosition> {
    let mut positions: HashMap<String, (Decimal, i64, BrokerCategory)> = HashMap::new();

    for summary in summaries {
        let entry = positions.entry(summary.broker_code.clone()).or_insert((
            Decimal::ZERO,
            0,
            get_broker_category(&summary.broker_code),
        ));

        entry.0 += summary.net_value;
        entry.1 += summary.net_volume;
    }

    positions
        .into_iter()
        .map(|(code, (net_value, net_volume, category))| BrokerPosition {
            broker_code: code,
            category,
            net_value,
            net_volume,
            is_buyer: net_value > Decimal::ZERO,
        })
        .collect()
}

/// Get top institutional accumulators
pub fn get_top_institutional_accumulators(
    summaries: &[BrokerSummary],
    limit: usize,
) -> Vec<BrokerPosition> {
    let mut positions = aggregate_broker_positions(summaries);

    // Filter to institutional brokers only and sort by net value
    positions.retain(|p| {
        matches!(
            p.category,
            BrokerCategory::ForeignInstitutional | BrokerCategory::LocalInstitutional
        ) && p.net_value > Decimal::ZERO
    });

    positions.sort_by(|a, b| b.net_value.cmp(&a.net_value));
    positions.truncate(limit);
    positions
}

/// Calculate accumulation persistence score
pub fn calculate_persistence_score(
    historical_scores: &[BrokerAccumulationScore],
) -> (Decimal, i32) {
    if historical_scores.is_empty() {
        return (Decimal::ZERO, 0);
    }

    let mut consecutive_days = 0i32;
    let mut current_streak = 0i32;

    for score in historical_scores.iter().rev() {
        if score.institutional_buying {
            current_streak += 1;
            if current_streak > consecutive_days {
                consecutive_days = current_streak;
            }
        } else {
            current_streak = 0;
        }
    }

    // Average score over period
    let avg_score: Decimal = historical_scores.iter().map(|s| s.score).sum::<Decimal>()
        / Decimal::from(historical_scores.len() as i32);

    (avg_score, consecutive_days)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_summary(code: &str, net_value: i64, date: NaiveDate) -> BrokerSummary {
        BrokerSummary {
            date,
            symbol: "BBCA".to_string(),
            broker_code: code.to_string(),
            buy_volume: if net_value > 0 { net_value } else { 0 },
            sell_volume: if net_value < 0 { -net_value } else { 0 },
            buy_value: Decimal::from(net_value.max(0)),
            sell_value: Decimal::from((-net_value).max(0)),
            net_volume: net_value,
            net_value: Decimal::from(net_value),
        }
    }

    #[test]
    fn test_rolling_accumulation_empty() {
        let summaries: Vec<BrokerSummary> = vec![];
        let result = calculate_rolling_accumulation(&summaries, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_rolling_accumulation_single_day() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let summaries = vec![
            make_summary("BK", 5000, date),  // Foreign institutional
            make_summary("CC", 3000, date),  // Local institutional
            make_summary("EP", -2000, date), // Retail
        ];

        let result = calculate_rolling_accumulation(&summaries, 5).unwrap();

        assert_eq!(result.symbol, "BBCA");
        assert!(result.accumulation_score > dec!(50)); // Should be bullish
        assert_eq!(result.days_accumulated, 1);
    }

    #[test]
    fn test_rolling_accumulation_multiple_days() {
        let summaries: Vec<BrokerSummary> = (0..5)
            .map(|i| {
                let date = NaiveDate::from_ymd_opt(2024, 1, 10 + i).unwrap();
                make_summary("BK", 1000, date)
            })
            .collect();

        let result = calculate_rolling_accumulation(&summaries, 5).unwrap();

        assert_eq!(result.days_accumulated, 5);
        assert!(result.is_accumulating);
    }

    #[test]
    fn test_coordinated_buying_detection() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let summaries = vec![
            make_summary("BK", 1000, date), // Foreign institutional
            make_summary("KZ", 1000, date), // Foreign institutional
            make_summary("CC", 1000, date), // Local institutional
            make_summary("SQ", 1000, date), // Local institutional
        ];

        let result = calculate_rolling_accumulation(&summaries, 5).unwrap();

        assert!(result.coordinated_buying);
    }

    #[test]
    fn test_aggregate_broker_positions() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let summaries = vec![
            make_summary("BK", 1000, date),
            make_summary("BK", 2000, date),
            make_summary("CC", -500, date),
        ];

        let positions = aggregate_broker_positions(&summaries);

        assert_eq!(positions.len(), 2);

        let bk_pos = positions.iter().find(|p| p.broker_code == "BK").unwrap();
        assert_eq!(bk_pos.net_value, Decimal::from(3000));
        assert!(bk_pos.is_buyer);

        let cc_pos = positions.iter().find(|p| p.broker_code == "CC").unwrap();
        assert_eq!(cc_pos.net_value, Decimal::from(-500));
        assert!(!cc_pos.is_buyer);
    }

    #[test]
    fn test_top_institutional_accumulators() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let summaries = vec![
            make_summary("BK", 5000, date),  // Foreign institutional - top
            make_summary("CC", 3000, date),  // Local institutional - second
            make_summary("EP", 10000, date), // Retail - should be excluded
            make_summary("KZ", 1000, date),  // Foreign institutional - third
        ];

        let top = get_top_institutional_accumulators(&summaries, 3);

        assert!(top.len() <= 3);
        assert!(top.iter().all(|p| {
            matches!(
                p.category,
                BrokerCategory::ForeignInstitutional | BrokerCategory::LocalInstitutional
            )
        }));

        // BK should be first
        assert_eq!(top[0].broker_code, "BK");
    }

    #[test]
    fn test_persistence_score() {
        let scores: Vec<BrokerAccumulationScore> = (0..5)
            .map(|i| BrokerAccumulationScore {
                symbol: "BBCA".to_string(),
                score: Decimal::from(70 + i),
                institutional_buying: true,
                foreign_buying: true,
                concentration_index: dec!(0.2),
                days_accumulated: i,
            })
            .collect();

        let (avg_score, consecutive) = calculate_persistence_score(&scores);

        assert!(avg_score > Decimal::ZERO);
        assert_eq!(consecutive, 5);
    }

    #[test]
    fn test_persistence_score_with_breaks() {
        let scores = vec![
            BrokerAccumulationScore {
                symbol: "BBCA".to_string(),
                score: dec!(70),
                institutional_buying: true,
                foreign_buying: true,
                concentration_index: dec!(0.2),
                days_accumulated: 1,
            },
            BrokerAccumulationScore {
                symbol: "BBCA".to_string(),
                score: dec!(40),
                institutional_buying: false, // Break in streak
                foreign_buying: false,
                concentration_index: dec!(0.1),
                days_accumulated: 0,
            },
            BrokerAccumulationScore {
                symbol: "BBCA".to_string(),
                score: dec!(75),
                institutional_buying: true,
                foreign_buying: true,
                concentration_index: dec!(0.25),
                days_accumulated: 2,
            },
        ];

        let (_, consecutive) = calculate_persistence_score(&scores);

        // Streak should be 1 (only the last day counts after break)
        assert_eq!(consecutive, 1);
    }

    #[test]
    fn test_dual_window_accumulation() {
        let summaries: Vec<BrokerSummary> = (0..20)
            .map(|i| {
                let date = NaiveDate::from_ymd_opt(2024, 1, 1 + i).unwrap();
                make_summary("BK", 1000, date)
            })
            .collect();

        let (window_5, window_20) = calculate_dual_window_accumulation(&summaries);

        assert!(window_5.is_some());
        assert!(window_20.is_some());

        let w5 = window_5.unwrap();
        let w20 = window_20.unwrap();

        // Both should show accumulation
        assert!(w5.is_accumulating);
        assert!(w20.is_accumulating);
    }
}
