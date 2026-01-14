//! Broker summary scraper for bandarmology analysis
//!
//! Data sources:
//! - IDX broker summary data (idxdata3.co.id)
//! - Third-party APIs (Stockbit, etc.)
//!
//! Compliance:
//! - Only accesses publicly available data
//! - Respects rate limits
//! - Does not access individual client data

use super::classification::{get_broker_category, is_foreign_broker};
use super::models::{BrokerAccumulationScore, BrokerActivity, BrokerCategory, BrokerSummary};
use crate::error::DataSourceError;
use chrono::NaiveDate;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use scraper::{Html, Selector};
use std::time::Duration;
use tracing::{debug, info, warn};

const IDX_DATA_URL: &str = "https://idxdata3.co.id";
const RATE_LIMIT_DELAY_MS: u64 = 500;

/// Broker summary scraper client
#[derive(Debug, Clone)]
pub struct BrokerScraper {
    client: Client,
    rate_limit_delay: Duration,
}

impl BrokerScraper {
    /// Create new broker scraper
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            rate_limit_delay: Duration::from_millis(RATE_LIMIT_DELAY_MS),
        }
    }

    /// Create scraper with custom rate limit
    pub fn with_rate_limit(mut self, delay_ms: u64) -> Self {
        self.rate_limit_delay = Duration::from_millis(delay_ms);
        self
    }

    /// Apply rate limiting
    async fn rate_limit(&self) {
        tokio::time::sleep(self.rate_limit_delay).await;
    }

    /// Fetch broker summary for a stock from IDX data
    pub async fn get_broker_summary(
        &self,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Vec<BrokerSummary>, DataSourceError> {
        debug!("Fetching broker summary for {} on {}", symbol, date);

        // Try IDX data first
        match self.fetch_idx_broker_data(symbol, date).await {
            Ok(summaries) if !summaries.is_empty() => {
                info!("Fetched {} broker summaries for {} from IDX", summaries.len(), symbol);
                return Ok(summaries);
            }
            Ok(_) => {
                debug!("No IDX data available for {} on {}", symbol, date);
            }
            Err(e) => {
                warn!("IDX fetch failed for {}: {}", symbol, e);
            }
        }

        // Return empty if no data available
        Ok(vec![])
    }

    /// Fetch broker data from IDX data service
    async fn fetch_idx_broker_data(
        &self,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Vec<BrokerSummary>, DataSourceError> {
        // IDX data URL pattern for broker summary
        let date_str = date.format("%Y%m%d").to_string();
        let url = format!(
            "{}/Download_Data/Broker/{}/{}.TXT",
            IDX_DATA_URL,
            symbol.to_uppercase(),
            date_str
        );

        self.rate_limit().await;

        let response = self.client.get(&url).send().await.map_err(|e| {
            DataSourceError::ApiError(format!("Failed to fetch IDX data: {}", e))
        })?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let text = response.text().await.map_err(|e| {
            DataSourceError::InvalidResponse(format!("Failed to read response: {}", e))
        })?;

        // Parse IDX broker data format (pipe-delimited)
        self.parse_idx_broker_text(&text, symbol, date)
    }

    /// Parse IDX broker summary text format
    fn parse_idx_broker_text(
        &self,
        text: &str,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Vec<BrokerSummary>, DataSourceError> {
        let mut summaries = Vec::new();

        for line in text.lines() {
            let fields: Vec<&str> = line.split('|').collect();

            // IDX format: Date|Symbol|BrokerCode|BuyVol|BuyVal|SellVol|SellVal
            if fields.len() >= 7 {
                let broker_code = fields[2].trim().to_string();
                let buy_volume: i64 = fields[3].trim().parse().unwrap_or(0);
                let buy_value: Decimal = fields[4].trim().parse().unwrap_or(Decimal::ZERO);
                let sell_volume: i64 = fields[5].trim().parse().unwrap_or(0);
                let sell_value: Decimal = fields[6].trim().parse().unwrap_or(Decimal::ZERO);

                if !broker_code.is_empty() && (buy_volume > 0 || sell_volume > 0) {
                    summaries.push(BrokerSummary {
                        date,
                        symbol: symbol.to_string(),
                        broker_code,
                        buy_volume,
                        sell_volume,
                        buy_value,
                        sell_value,
                        net_volume: buy_volume - sell_volume,
                        net_value: buy_value - sell_value,
                    });
                }
            }
        }

        Ok(summaries)
    }

    /// Fetch broker summary from HTML page (alternative source)
    pub async fn get_broker_summary_html(
        &self,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Vec<BrokerSummary>, DataSourceError> {
        // Alternative URL pattern for HTML broker data
        let url = format!(
            "https://www.idx.co.id/id/data-pasar/ringkasan-perdagangan/ringkasan-broker/?kodeEmiten={}",
            symbol.to_uppercase()
        );

        self.rate_limit().await;

        let response = self.client.get(&url).send().await.map_err(|e| {
            DataSourceError::ApiError(format!("Failed to fetch broker HTML: {}", e))
        })?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let html = response.text().await.map_err(|e| {
            DataSourceError::InvalidResponse(format!("Failed to read response: {}", e))
        })?;

        self.parse_broker_html(&html, symbol, date)
    }

    /// Parse broker summary from HTML table
    fn parse_broker_html(
        &self,
        html: &str,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Vec<BrokerSummary>, DataSourceError> {
        let document = Html::parse_document(html);

        let table_selector = Selector::parse("table.broker-summary, #broker-table, table")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid selector".into()))?;
        let row_selector = Selector::parse("tbody tr")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid row selector".into()))?;
        let cell_selector = Selector::parse("td")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid cell selector".into()))?;

        let mut summaries = Vec::new();

        for table in document.select(&table_selector) {
            let text = table.text().collect::<String>().to_lowercase();

            // Look for broker-related tables
            if text.contains("broker") || text.contains("buy") || text.contains("sell") {
                for row in table.select(&row_selector) {
                    let cells: Vec<_> = row.select(&cell_selector).collect();

                    // Expected format: BrokerCode | BuyVol | BuyVal | SellVol | SellVal
                    if cells.len() >= 5 {
                        let broker_code = cells[0].text().collect::<String>().trim().to_string();

                        if broker_code.len() == 2 && broker_code.chars().all(|c| c.is_alphanumeric()) {
                            let buy_volume = parse_number(&cells[1].text().collect::<String>());
                            let buy_value = parse_decimal(&cells[2].text().collect::<String>());
                            let sell_volume = parse_number(&cells[3].text().collect::<String>());
                            let sell_value = parse_decimal(&cells[4].text().collect::<String>());

                            if buy_volume > 0 || sell_volume > 0 {
                                summaries.push(BrokerSummary {
                                    date,
                                    symbol: symbol.to_string(),
                                    broker_code,
                                    buy_volume,
                                    sell_volume,
                                    buy_value,
                                    sell_value,
                                    net_volume: buy_volume - sell_volume,
                                    net_value: buy_value - sell_value,
                                });
                            }
                        }
                    }
                }

                if !summaries.is_empty() {
                    break;
                }
            }
        }

        Ok(summaries)
    }

    /// Get multiple days of broker data for analysis
    pub async fn get_broker_summary_range(
        &self,
        symbol: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<BrokerSummary>, DataSourceError> {
        let mut all_summaries = Vec::new();
        let mut current_date = start_date;

        while current_date <= end_date {
            match self.get_broker_summary(symbol, current_date).await {
                Ok(summaries) => all_summaries.extend(summaries),
                Err(e) => warn!("Failed to fetch {} for {}: {}", symbol, current_date, e),
            }
            current_date = current_date.succ_opt().unwrap_or(end_date);
        }

        Ok(all_summaries)
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

/// Parse number from text (handles thousand separators)
fn parse_number(text: &str) -> i64 {
    let cleaned: String = text
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-')
        .collect();

    cleaned.parse().unwrap_or(0)
}

/// Parse decimal from text
fn parse_decimal(text: &str) -> Decimal {
    let cleaned: String = text
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '-')
        .collect();

    let normalized = cleaned.replace(',', ".");
    normalized.parse().unwrap_or(Decimal::ZERO)
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
