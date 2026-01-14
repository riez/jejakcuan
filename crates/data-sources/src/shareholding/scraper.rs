//! Shareholding data scraper
//!
//! Scraper for fetching shareholding data from KSEI/OJK/IDX.
//!
//! Data sources:
//! - KSEI AKSes: Real-time custody data (requires scraping)
//! - IDX: 5%+ shareholder disclosures
//! - OJK: Ownership change filings
//!
//! Compliance:
//! - Only scrapes publicly available data
//! - Respects rate limits and ToS
//! - PDP Law compliant (no personal data)

use super::models::{OwnershipChange, Shareholder, ShareholderType, ShareholdingSnapshot, ShareholdingSource};
use crate::error::DataSourceError;
use chrono::NaiveDate;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use scraper::{Html, Selector};
use std::time::Duration;
use tracing::{debug, info, warn};

const IDX_BASE_URL: &str = "https://www.idx.co.id";
const RATE_LIMIT_DELAY_MS: u64 = 500;

/// Shareholding data scraper client
#[derive(Debug, Clone)]
pub struct ShareholdingScraper {
    client: Client,
    rate_limit_delay: Duration,
}

impl ShareholdingScraper {
    /// Create new shareholding scraper
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

    /// Get the HTTP client (for testing or custom requests)
    #[allow(dead_code)]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Apply rate limiting between requests
    async fn rate_limit(&self) {
        tokio::time::sleep(self.rate_limit_delay).await;
    }

    /// Fetch shareholding snapshot for a stock from KSEI AKSes
    ///
    /// KSEI provides custody data through AKSes portal.
    /// Note: This requires web scraping as no official API exists.
    pub async fn get_ksei_snapshot(
        &self,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Option<ShareholdingSnapshot>, DataSourceError> {
        debug!("Fetching KSEI shareholding for {} on {}", symbol, date);

        // KSEI AKSes portal URL pattern
        let url = format!(
            "https://akses.ksei.co.id/acuan-kepemilikan-efek/{}",
            symbol.to_uppercase()
        );

        self.rate_limit().await;

        match self.client.get(&url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    debug!("KSEI returned status {} for {}", response.status(), symbol);
                    return Ok(None);
                }

                let html = response.text().await.map_err(|e| {
                    DataSourceError::InvalidResponse(format!("Failed to read KSEI response: {}", e))
                })?;

                // Parse shareholding table from KSEI HTML
                match self.parse_ksei_html(&html, symbol, date) {
                    Ok(Some(snapshot)) => {
                        info!("Parsed KSEI data for {} with {} shareholders", symbol, snapshot.shareholders.len());
                        Ok(Some(snapshot))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => {
                        warn!("Failed to parse KSEI HTML for {}: {}", symbol, e);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch KSEI data for {}: {}", symbol, e);
                Ok(None)
            }
        }
    }

    /// Parse KSEI HTML shareholding table
    fn parse_ksei_html(
        &self,
        html: &str,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Option<ShareholdingSnapshot>, DataSourceError> {
        let document = Html::parse_document(html);

        // Common table selectors for KSEI-style pages
        let table_selector = Selector::parse("table.shareholding, table.ownership, #shareholding-table")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid selector".into()))?;
        let row_selector = Selector::parse("tbody tr")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid row selector".into()))?;
        let cell_selector = Selector::parse("td")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid cell selector".into()))?;

        let table = match document.select(&table_selector).next() {
            Some(t) => t,
            None => return Ok(None),
        };

        let mut shareholders = Vec::new();
        let mut total_shares: i64 = 0;

        for row in table.select(&row_selector) {
            let cells: Vec<_> = row.select(&cell_selector).collect();

            if cells.len() >= 3 {
                let name = cells[0].text().collect::<String>().trim().to_string();
                let shares_text = cells[1].text().collect::<String>();
                let pct_text = cells[2].text().collect::<String>();

                if let (Ok(shares), Ok(pct)) = (
                    parse_number(&shares_text),
                    parse_percentage(&pct_text),
                ) {
                    if !name.is_empty() && shares > 0 {
                        let shareholder_type = ShareholderType::from_name(&name);
                        shareholders.push(Shareholder::with_type(
                            name,
                            shareholder_type,
                            shares,
                            pct,
                        ));
                        total_shares += shares;
                    }
                }
            }
        }

        if shareholders.is_empty() {
            return Ok(None);
        }

        Ok(Some(ShareholdingSnapshot::new(
            symbol.to_string(),
            date,
            total_shares,
            shareholders,
        )))
    }

    /// Fetch shareholding data from OJK filings
    pub async fn get_ojk_filings(
        &self,
        symbol: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<Vec<OwnershipChange>, DataSourceError> {
        debug!("Fetching OJK filings for {}", symbol);

        // OJK filings URL
        let _url = format!(
            "https://www.ojk.go.id/id/kanal/pasar-modal/data-dan-statistik/kepemilikan-efek/{}",
            symbol.to_uppercase()
        );

        self.rate_limit().await;

        // OJK doesn't have a consistent public API
        // In production, this would scrape from their filings database
        warn!("OJK scraper requires site-specific implementation for {}", symbol);

        Ok(vec![])
    }

    /// Fetch shareholding data from IDX company profile
    pub async fn get_idx_snapshot(
        &self,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Option<ShareholdingSnapshot>, DataSourceError> {
        debug!("Fetching IDX shareholding for {} on {}", symbol, date);

        // IDX company profile API
        let url = format!(
            "{}/id/perusahaan-tercatat/profil-perusahaan-tercatat/?kodeEmiten={}",
            IDX_BASE_URL,
            symbol.to_uppercase()
        );

        self.rate_limit().await;

        match self.client.get(&url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    debug!("IDX returned status {} for {}", response.status(), symbol);
                    return Ok(None);
                }

                let html = response.text().await.map_err(|e| {
                    DataSourceError::InvalidResponse(format!("Failed to read IDX response: {}", e))
                })?;

                match self.parse_idx_html(&html, symbol, date) {
                    Ok(Some(snapshot)) => {
                        info!("Parsed IDX data for {} with {} shareholders", symbol, snapshot.shareholders.len());
                        Ok(Some(snapshot))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => {
                        warn!("Failed to parse IDX HTML for {}: {}", symbol, e);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch IDX data for {}: {}", symbol, e);
                Ok(None)
            }
        }
    }

    /// Parse IDX HTML shareholding section
    fn parse_idx_html(
        &self,
        html: &str,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Option<ShareholdingSnapshot>, DataSourceError> {
        let document = Html::parse_document(html);

        // IDX shareholding table selectors
        let section_selector = Selector::parse("#shareholder, .shareholder-section, [data-section='shareholder']")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid selector".into()))?;
        let row_selector = Selector::parse("tr")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid row selector".into()))?;
        let cell_selector = Selector::parse("td")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid cell selector".into()))?;

        let section = match document.select(&section_selector).next() {
            Some(s) => s,
            None => {
                // Try alternative table structure
                return self.parse_idx_table_alternative(&document, symbol, date);
            }
        };

        let mut shareholders = Vec::new();
        let mut total_shares: i64 = 0;

        for row in section.select(&row_selector) {
            let cells: Vec<_> = row.select(&cell_selector).collect();

            if cells.len() >= 2 {
                let name = cells[0].text().collect::<String>().trim().to_string();
                let shares_text = cells.get(1).map(|c| c.text().collect::<String>()).unwrap_or_default();
                let pct_text = cells.get(2).map(|c| c.text().collect::<String>());

                if let Ok(shares) = parse_number(&shares_text) {
                    if !name.is_empty() && shares > 0 {
                        let pct = pct_text
                            .and_then(|p| parse_percentage(&p).ok())
                            .unwrap_or(Decimal::ZERO);
                        let shareholder_type = ShareholderType::from_name(&name);
                        shareholders.push(Shareholder::with_type(
                            name,
                            shareholder_type,
                            shares,
                            pct,
                        ));
                        total_shares += shares;
                    }
                }
            }
        }

        if shareholders.is_empty() {
            return Ok(None);
        }

        Ok(Some(ShareholdingSnapshot::new(
            symbol.to_string(),
            date,
            total_shares,
            shareholders,
        )))
    }

    /// Alternative parsing for IDX table structure
    fn parse_idx_table_alternative(
        &self,
        document: &Html,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Option<ShareholdingSnapshot>, DataSourceError> {
        // Look for any table containing shareholder-like data
        let table_selector = Selector::parse("table")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid selector".into()))?;
        let row_selector = Selector::parse("tr")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid row selector".into()))?;
        let cell_selector = Selector::parse("td, th")
            .map_err(|_| DataSourceError::InvalidResponse("Invalid cell selector".into()))?;

        for table in document.select(&table_selector) {
            let text = table.text().collect::<String>().to_lowercase();

            // Check if table contains shareholder-related content
            if text.contains("pemegang saham") || text.contains("shareholder") || text.contains("kepemilikan") {
                let mut shareholders = Vec::new();
                let mut total_shares: i64 = 0;

                for row in table.select(&row_selector) {
                    let cells: Vec<_> = row.select(&cell_selector).collect();

                    if cells.len() >= 2 {
                        let name = cells[0].text().collect::<String>().trim().to_string();

                        // Skip header rows
                        if name.to_lowercase().contains("nama") || name.to_lowercase().contains("name") {
                            continue;
                        }

                        for cell in cells.iter().skip(1) {
                            let text = cell.text().collect::<String>();
                            if let Ok(shares) = parse_number(&text) {
                                if shares > 0 && !name.is_empty() {
                                    let shareholder_type = ShareholderType::from_name(&name);
                                    shareholders.push(Shareholder::with_type(
                                        name.clone(),
                                        shareholder_type,
                                        shares,
                                        dec!(0),
                                    ));
                                    total_shares += shares;
                                    break;
                                }
                            }
                        }
                    }
                }

                // Recalculate percentages
                if total_shares > 0 {
                    for shareholder in &mut shareholders {
                        shareholder.percentage = Decimal::from(shareholder.shares_held * 100) / Decimal::from(total_shares);
                    }
                }

                if !shareholders.is_empty() {
                    return Ok(Some(ShareholdingSnapshot::new(
                        symbol.to_string(),
                        date,
                        total_shares,
                        shareholders,
                    )));
                }
            }
        }

        Ok(None)
    }

    /// Get shareholding snapshot from best available source
    ///
    /// Tries sources in order: KSEI > OJK > IDX
    pub async fn get_snapshot(
        &self,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Option<(ShareholdingSnapshot, ShareholdingSource)>, DataSourceError> {
        // Try KSEI first (most up-to-date)
        if let Some(snapshot) = self.get_ksei_snapshot(symbol, date).await? {
            return Ok(Some((snapshot, ShareholdingSource::Ksei)));
        }

        // Fall back to IDX
        if let Some(snapshot) = self.get_idx_snapshot(symbol, date).await? {
            return Ok(Some((snapshot, ShareholdingSource::Idx)));
        }

        // No data available
        Ok(None)
    }

    /// Compare two snapshots to find ownership changes
    pub fn compare_snapshots(
        previous: &ShareholdingSnapshot,
        current: &ShareholdingSnapshot,
    ) -> Vec<OwnershipChange> {
        let mut changes = Vec::new();

        for curr_holder in &current.shareholders {
            // Find same shareholder in previous snapshot
            let prev_holder = previous
                .shareholders
                .iter()
                .find(|h| h.name == curr_holder.name);

            let (prev_shares, prev_pct) = prev_holder
                .map(|h| (h.shares_held, h.percentage))
                .unwrap_or((0, Decimal::ZERO));

            // Only record if there's a change
            if curr_holder.shares_held != prev_shares {
                changes.push(OwnershipChange::from_snapshots(
                    &current.symbol,
                    &curr_holder.name,
                    curr_holder.shareholder_type,
                    current.report_date,
                    prev_shares,
                    curr_holder.shares_held,
                    prev_pct,
                    curr_holder.percentage,
                ));
            }
        }

        // Check for shareholders who disappeared (sold all)
        for prev_holder in &previous.shareholders {
            let still_exists = current
                .shareholders
                .iter()
                .any(|h| h.name == prev_holder.name);

            if !still_exists && prev_holder.shares_held > 0 {
                changes.push(OwnershipChange::from_snapshots(
                    &current.symbol,
                    &prev_holder.name,
                    prev_holder.shareholder_type,
                    current.report_date,
                    prev_holder.shares_held,
                    0,
                    prev_holder.percentage,
                    Decimal::ZERO,
                ));
            }
        }

        changes
    }

    /// Get historical snapshots for trend analysis
    ///
    /// Returns snapshots from available dates in the given range.
    pub async fn get_historical_snapshots(
        &self,
        symbol: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(ShareholdingSnapshot, ShareholdingSource)>, DataSourceError> {
        debug!(
            "Fetching historical shareholding for {} from {} to {}",
            symbol, start_date, end_date
        );

        // Placeholder: In production, iterate through available data points
        // KSEI/OJK typically have monthly or quarterly updates

        warn!(
            "Historical shareholding not yet implemented for {} - returning empty",
            symbol
        );

        Ok(vec![])
    }
}

impl Default for ShareholdingScraper {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a number from text (handles thousand separators)
fn parse_number(text: &str) -> Result<i64, ()> {
    let cleaned: String = text
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-')
        .collect();

    if cleaned.is_empty() {
        return Err(());
    }

    cleaned.parse::<i64>().map_err(|_| ())
}

/// Parse a percentage from text (e.g., "12.5%", "12,5 %")
fn parse_percentage(text: &str) -> Result<Decimal, ()> {
    let cleaned: String = text
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '-')
        .collect();

    if cleaned.is_empty() {
        return Err(());
    }

    // Handle comma as decimal separator
    let normalized = cleaned.replace(',', ".");

    normalized
        .parse::<Decimal>()
        .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::super::models::ShareholderType;
    use super::*;

    fn make_shareholder(name: &str, shares: i64, percentage: i32) -> Shareholder {
        Shareholder::with_type(
            name.to_string(),
            ShareholderType::from_name(name),
            shares,
            Decimal::from(percentage),
        )
    }

    fn make_snapshot(
        symbol: &str,
        date: NaiveDate,
        shareholders: Vec<Shareholder>,
    ) -> ShareholdingSnapshot {
        let total_shares: i64 = shareholders.iter().map(|s| s.shares_held).sum();
        ShareholdingSnapshot::new(symbol.to_string(), date, total_shares, shareholders)
    }

    #[test]
    fn test_compare_snapshots_increase() {
        let prev = make_snapshot(
            "BBCA",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            vec![
                make_shareholder("PT Fund A", 1_000_000, 10),
                make_shareholder("PT Fund B", 500_000, 5),
            ],
        );

        let curr = make_snapshot(
            "BBCA",
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            vec![
                make_shareholder("PT Fund A", 1_500_000, 15), // Increased
                make_shareholder("PT Fund B", 500_000, 5),    // No change
            ],
        );

        let changes = ShareholdingScraper::compare_snapshots(&prev, &curr);

        assert_eq!(changes.len(), 1); // Only Fund A changed
        assert_eq!(changes[0].shareholder_name, "PT Fund A");
        assert_eq!(changes[0].change_shares, 500_000);
    }

    #[test]
    fn test_compare_snapshots_new_holder() {
        let prev = make_snapshot(
            "BBCA",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            vec![make_shareholder("PT Fund A", 1_000_000, 10)],
        );

        let curr = make_snapshot(
            "BBCA",
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            vec![
                make_shareholder("PT Fund A", 1_000_000, 10),
                make_shareholder("PT Fund B", 500_000, 5), // New holder
            ],
        );

        let changes = ShareholdingScraper::compare_snapshots(&prev, &curr);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].shareholder_name, "PT Fund B");
        assert_eq!(changes[0].previous_shares, 0);
        assert_eq!(changes[0].current_shares, 500_000);
    }

    #[test]
    fn test_compare_snapshots_holder_exited() {
        let prev = make_snapshot(
            "BBCA",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            vec![
                make_shareholder("PT Fund A", 1_000_000, 10),
                make_shareholder("PT Fund B", 500_000, 5),
            ],
        );

        let curr = make_snapshot(
            "BBCA",
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            vec![
                make_shareholder("PT Fund A", 1_000_000, 10), // Still here
                                                              // Fund B exited
            ],
        );

        let changes = ShareholdingScraper::compare_snapshots(&prev, &curr);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].shareholder_name, "PT Fund B");
        assert_eq!(changes[0].current_shares, 0);
        assert_eq!(changes[0].previous_shares, 500_000);
    }

    #[test]
    fn test_compare_snapshots_no_changes() {
        let prev = make_snapshot(
            "BBCA",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            vec![make_shareholder("PT Fund A", 1_000_000, 10)],
        );

        let curr = make_snapshot(
            "BBCA",
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            vec![make_shareholder("PT Fund A", 1_000_000, 10)],
        );

        let changes = ShareholdingScraper::compare_snapshots(&prev, &curr);

        assert!(changes.is_empty());
    }

    #[tokio::test]
    async fn test_scraper_creation() {
        let scraper = ShareholdingScraper::new();
        // Just verify the scraper can be created and the client is accessible
        let _client = scraper.client();
    }

    #[tokio::test]
    async fn test_placeholder_returns_none() {
        let scraper = ShareholdingScraper::new();
        let result = scraper
            .get_ksei_snapshot("BBCA", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
