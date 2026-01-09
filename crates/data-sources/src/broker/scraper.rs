//! Broker summary scraper
//!
//! Note: This scraper targets publicly available broker summary data.
//! In production, consider using official APIs if available.

use super::classification::{get_broker_category, is_foreign_broker};
use super::models::{BrokerAccumulationScore, BrokerActivity, BrokerCategory, BrokerSummary};
use crate::error::DataSourceError;
use chrono::NaiveDate;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::time::Duration;
use tracing::debug;

/// Broker summary scraper client
#[derive(Debug, Clone)]
pub struct BrokerScraper {
    client: Client,
}

impl BrokerScraper {
    /// Create new broker scraper
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; JejakCuan/1.0)")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Fetch broker summary for a stock from mock/placeholder data
    ///
    /// In production, this would scrape from IDX or third-party sources.
    /// For now, returns placeholder data structure.
    pub async fn get_broker_summary(
        &self,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Vec<BrokerSummary>, DataSourceError> {
        debug!("Fetching broker summary for {} on {}", symbol, date);

        // Placeholder: In production, implement actual scraping
        // The IDX website structure changes frequently, so this would need
        // regular maintenance. Alternative: use Stockbit API if available.

        // Return empty for now - will be populated when data source is configured
        Ok(vec![])
    }

    /// Get the HTTP client (for testing or custom requests)
    #[allow(dead_code)]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Calculate broker activity summary
    pub fn calculate_activity(summaries: &[BrokerSummary]) -> BrokerActivity {
        if summaries.is_empty() {
            return BrokerActivity {
                symbol: String::new(),
                date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                top_buyers: vec![],
                top_sellers: vec![],
                foreign_net: Decimal::ZERO,
                domestic_net: Decimal::ZERO,
            };
        }

        let symbol = summaries[0].symbol.clone();
        let date = summaries[0].date;

        // Sort by net value
        let mut sorted = summaries.to_vec();
        sorted.sort_by(|a, b| b.net_value.cmp(&a.net_value));

        let top_buyers: Vec<_> = sorted
            .iter()
            .filter(|s| s.net_value > Decimal::ZERO)
            .take(5)
            .cloned()
            .collect();

        let top_sellers: Vec<_> = sorted
            .iter()
            .filter(|s| s.net_value < Decimal::ZERO)
            .rev()
            .take(5)
            .cloned()
            .collect();

        // Calculate foreign vs domestic
        let foreign_net: Decimal = summaries
            .iter()
            .filter(|s| is_foreign_broker(&s.broker_code))
            .map(|s| s.net_value)
            .sum();

        let domestic_net: Decimal = summaries
            .iter()
            .filter(|s| !is_foreign_broker(&s.broker_code))
            .map(|s| s.net_value)
            .sum();

        BrokerActivity {
            symbol,
            date,
            top_buyers,
            top_sellers,
            foreign_net,
            domestic_net,
        }
    }

    /// Calculate Herfindahl-Hirschman Index (HHI) for broker concentration
    /// HHI > 0.15-0.20 indicates concentrated accumulation
    pub fn calculate_hhi(summaries: &[BrokerSummary]) -> Decimal {
        if summaries.is_empty() {
            return Decimal::ZERO;
        }

        let total_volume: i64 = summaries.iter().map(|s| s.buy_volume + s.sell_volume).sum();

        if total_volume == 0 {
            return Decimal::ZERO;
        }

        let total_dec = Decimal::from(total_volume);

        summaries
            .iter()
            .map(|s| {
                let share = Decimal::from(s.buy_volume + s.sell_volume) / total_dec;
                share * share
            })
            .sum()
    }

    /// Calculate broker accumulation score
    ///
    /// Factors:
    /// - Net buying by institutional brokers (weighted by category)
    /// - Foreign flow direction
    /// - Concentration (HHI)
    /// - Consistency over time
    pub fn calculate_accumulation_score(
        summaries: &[BrokerSummary],
        historical_days: i32,
    ) -> BrokerAccumulationScore {
        if summaries.is_empty() {
            return BrokerAccumulationScore {
                symbol: String::new(),
                score: dec!(50),
                institutional_buying: false,
                foreign_buying: false,
                concentration_index: Decimal::ZERO,
                days_accumulated: 0,
            };
        }

        let symbol = summaries[0].symbol.clone();

        // Calculate weighted institutional net
        let mut institutional_net = Decimal::ZERO;
        let mut foreign_net = Decimal::ZERO;
        let mut total_net = Decimal::ZERO;

        for summary in summaries {
            let category = get_broker_category(&summary.broker_code);
            let weight = category.weight();

            institutional_net += summary.net_value * weight;
            total_net += summary.net_value.abs();

            if matches!(category, BrokerCategory::ForeignInstitutional) {
                foreign_net += summary.net_value;
            }
        }

        let hhi = Self::calculate_hhi(summaries);

        // Base score from institutional buying
        let mut score = dec!(50);

        if total_net > Decimal::ZERO {
            let institutional_ratio = institutional_net / total_net;
            score += institutional_ratio * dec!(30); // -30 to +30 contribution
        }

        // Bonus for foreign buying
        if foreign_net > Decimal::ZERO {
            score += dec!(10);
        } else if foreign_net < Decimal::ZERO {
            score -= dec!(5);
        }

        // Bonus for concentrated accumulation (high HHI with positive flow)
        if hhi > dec!(0.15) && institutional_net > Decimal::ZERO {
            score += dec!(10);
        }

        // Clamp to 0-100
        score = score.max(Decimal::ZERO).min(dec!(100));

        BrokerAccumulationScore {
            symbol,
            score,
            institutional_buying: institutional_net > Decimal::ZERO,
            foreign_buying: foreign_net > Decimal::ZERO,
            concentration_index: hhi,
            days_accumulated: historical_days,
        }
    }
}

impl Default for BrokerScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_summary(code: &str, net_value: i64) -> BrokerSummary {
        BrokerSummary {
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
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
    fn test_hhi_calculation() {
        let summaries = vec![
            make_summary("BK", 1000),
            make_summary("CC", 1000),
            make_summary("EP", 1000),
            make_summary("AI", 1000),
        ];

        let hhi = BrokerScraper::calculate_hhi(&summaries);
        // Equal distribution: 4 brokers each with 25% = 0.25^2 * 4 = 0.25
        assert!(hhi > dec!(0.2) && hhi < dec!(0.3));
    }

    #[test]
    fn test_accumulation_score_institutional() {
        let summaries = vec![
            make_summary("BK", 5000),  // Foreign institutional buying
            make_summary("CC", 3000),  // Local institutional buying
            make_summary("EP", -2000), // Retail selling
        ];

        let score = BrokerScraper::calculate_accumulation_score(&summaries, 1);

        assert!(score.institutional_buying);
        assert!(score.foreign_buying);
        assert!(score.score > dec!(60)); // Should be bullish
    }

    #[test]
    fn test_empty_summaries() {
        let summaries: Vec<BrokerSummary> = vec![];

        let hhi = BrokerScraper::calculate_hhi(&summaries);
        assert_eq!(hhi, Decimal::ZERO);

        let activity = BrokerScraper::calculate_activity(&summaries);
        assert!(activity.top_buyers.is_empty());
        assert!(activity.top_sellers.is_empty());

        let score = BrokerScraper::calculate_accumulation_score(&summaries, 0);
        assert_eq!(score.score, dec!(50));
    }

    #[test]
    fn test_calculate_activity() {
        let summaries = vec![
            make_summary("BK", 5000),
            make_summary("CC", 3000),
            make_summary("EP", -2000),
            make_summary("AI", -1000),
        ];

        let activity = BrokerScraper::calculate_activity(&summaries);

        assert_eq!(activity.symbol, "BBCA");
        assert_eq!(activity.top_buyers.len(), 2);
        assert_eq!(activity.top_sellers.len(), 2);
        assert!(activity.foreign_net > Decimal::ZERO);
    }
}
