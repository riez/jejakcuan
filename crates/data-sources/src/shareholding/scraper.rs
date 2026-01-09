//! Shareholding data scraper
//!
//! Scraper for fetching shareholding data from KSEI/OJK.
//!
//! Note: This scraper targets publicly available shareholding data.
//! In production, consider using official APIs if available.

use super::models::{OwnershipChange, Shareholder, ShareholdingSnapshot, ShareholdingSource};
use crate::error::DataSourceError;
use chrono::NaiveDate;
use reqwest::Client;
use rust_decimal::Decimal;
use std::time::Duration;
use tracing::{debug, warn};

/// Shareholding data scraper client
#[derive(Debug, Clone)]
pub struct ShareholdingScraper {
    client: Client,
}

impl ShareholdingScraper {
    /// Create new shareholding scraper
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; JejakCuan/1.0)")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Get the HTTP client (for testing or custom requests)
    #[allow(dead_code)]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Fetch shareholding snapshot for a stock from KSEI
    ///
    /// In production, this would scrape from KSEI's public data.
    /// For now, returns placeholder data structure.
    pub async fn get_ksei_snapshot(
        &self,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Option<ShareholdingSnapshot>, DataSourceError> {
        debug!("Fetching KSEI shareholding for {} on {}", symbol, date);

        // Placeholder: In production, implement actual scraping from KSEI
        // KSEI provides custody data through AKSes (Acuan Kepemilikan Sekuritas)
        // https://akses.ksei.co.id/

        warn!(
            "KSEI scraper not yet implemented for {} - returning None",
            symbol
        );

        // Return None for now - will be populated when data source is configured
        Ok(None)
    }

    /// Fetch shareholding data from OJK filings
    ///
    /// OJK requires companies to report ownership changes.
    /// This would scrape from OJK's public database.
    pub async fn get_ojk_filings(
        &self,
        symbol: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<Vec<OwnershipChange>, DataSourceError> {
        debug!("Fetching OJK filings for {}", symbol);

        // Placeholder: In production, implement actual scraping from OJK
        // OJK filings are available at https://www.ojk.go.id/

        warn!(
            "OJK scraper not yet implemented for {} - returning empty",
            symbol
        );

        // Return empty for now
        Ok(vec![])
    }

    /// Fetch shareholding data from IDX (annual report disclosures)
    ///
    /// IDX publishes annual reports that contain major shareholder info.
    pub async fn get_idx_snapshot(
        &self,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<Option<ShareholdingSnapshot>, DataSourceError> {
        debug!("Fetching IDX shareholding for {} on {}", symbol, date);

        // Placeholder: In production, implement actual scraping from IDX
        // IDX annual reports: https://www.idx.co.id/id/perusahaan-tercatat/laporan-keuangan-dan-tahunan

        warn!(
            "IDX scraper not yet implemented for {} - returning None",
            symbol
        );

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

/// Parse shareholder data from HTML table (utility function)
///
/// This is a placeholder that would be implemented with actual HTML parsing
#[allow(dead_code)]
fn parse_shareholder_table(html: &str) -> Result<Vec<Shareholder>, DataSourceError> {
    // Placeholder: Use scraper crate to parse HTML
    // Example pattern for KSEI/OJK tables:
    //
    // let document = scraper::Html::parse_document(html);
    // let row_selector = scraper::Selector::parse("table.shareholders tr").unwrap();
    // ...

    if html.is_empty() {
        return Ok(vec![]);
    }

    Err(DataSourceError::InvalidResponse(
        "HTML parsing not yet implemented".to_string(),
    ))
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
