//! Sectors.app API client implementation
//!
//! Provides a robust REST client with:
//! - Automatic retry with exponential backoff
//! - Rate limiting awareness
//! - Comprehensive error handling

use super::models::*;
use crate::error::DataSourceError;
use reqwest::{Client, StatusCode};
use std::time::Duration;
use tracing::{debug, warn};

const BASE_URL_V1: &str = "https://api.sectors.app/v1";
const BASE_URL_V2: &str = "https://api.sectors.app/v2";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 1000;

/// Sectors.app API client
#[derive(Debug, Clone)]
pub struct SectorsClient {
    client: Client,
    api_key: String,
}

impl SectorsClient {
    /// Create a new Sectors.app client with the provided API key
    pub fn new(api_key: String) -> Result<Self, DataSourceError> {
        if api_key.is_empty() {
            return Err(DataSourceError::InvalidResponse(
                "Sectors.app API key is required".to_string(),
            ));
        }

        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .user_agent("JejakCuan/1.0")
            .build()
            .map_err(|e| {
                DataSourceError::ApiError(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self { client, api_key })
    }

    /// Create client from SECTORS_API_KEY environment variable
    pub fn from_env() -> Result<Self, DataSourceError> {
        let api_key = std::env::var("SECTORS_API_KEY").map_err(|_| {
            DataSourceError::InvalidResponse("SECTORS_API_KEY environment variable not set".into())
        })?;
        Self::new(api_key)
    }

    /// Execute a GET request with retry logic
    async fn get_with_retry<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        params: &[(&str, String)],
    ) -> Result<T, DataSourceError> {
        let mut last_error = None;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                debug!(
                    "Retry attempt {} for {} after {}ms",
                    attempt, url, backoff_ms
                );
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                backoff_ms *= 2; // Exponential backoff
            }

            let result = self
                .client
                .get(url)
                .header("Authorization", &self.api_key)
                .query(params)
                .send()
                .await;

            match result {
                Ok(response) => {
                    let status = response.status();

                    if status == StatusCode::TOO_MANY_REQUESTS {
                        warn!("Rate limited by Sectors.app API, will retry");
                        last_error = Some(DataSourceError::RateLimited);
                        continue;
                    }

                    if status.is_server_error() {
                        warn!("Server error from Sectors.app: {}", status);
                        last_error = Some(DataSourceError::ApiError(format!(
                            "Server error: {}",
                            status
                        )));
                        continue;
                    }

                    if !status.is_success() {
                        let error_text = response.text().await.unwrap_or_default();
                        return Err(DataSourceError::InvalidResponse(format!(
                            "API error {}: {}",
                            status, error_text
                        )));
                    }

                    return response.json::<T>().await.map_err(|e| {
                        DataSourceError::InvalidResponse(format!("Failed to parse response: {}", e))
                    });
                }
                Err(e) => {
                    warn!("Network error calling Sectors.app: {}", e);
                    last_error = Some(DataSourceError::HttpError(e));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| DataSourceError::ApiError("Max retries exceeded".into())))
    }

    // ============================================
    // V2 API Endpoints
    // ============================================

    /// Search companies using the V2 screener API
    pub async fn search_companies(
        &self,
        query: CompanyQuery,
    ) -> Result<CompaniesResponse, DataSourceError> {
        let url = format!("{}/companies/", BASE_URL_V2);
        let params = query.to_params();
        self.get_with_retry(&url, &params).await
    }

    /// Get companies by sector
    pub async fn get_companies_by_sector(
        &self,
        sector: &str,
        limit: i32,
    ) -> Result<CompaniesResponse, DataSourceError> {
        let query = CompanyQuery::new()
            .where_clause(&format!("sector = '{}'", sector))
            .order_by("-market_cap")
            .limit(limit);
        self.search_companies(query).await
    }

    /// Get companies by subsector
    pub async fn get_companies_by_subsector(
        &self,
        subsector: &str,
        limit: i32,
    ) -> Result<CompaniesResponse, DataSourceError> {
        let query = CompanyQuery::new()
            .where_clause(&format!("sub_sector = '{}'", subsector))
            .order_by("-market_cap")
            .limit(limit);
        self.search_companies(query).await
    }

    /// Get top companies by market cap
    pub async fn get_top_by_market_cap(
        &self,
        limit: i32,
    ) -> Result<CompaniesResponse, DataSourceError> {
        let query = CompanyQuery::new().order_by("-market_cap").limit(limit);
        self.search_companies(query).await
    }

    /// Get companies with high ROE
    pub async fn get_high_roe_companies(
        &self,
        min_roe: f64,
        limit: i32,
    ) -> Result<CompaniesResponse, DataSourceError> {
        let query = CompanyQuery::new()
            .where_clause(&format!("roe_ttm > {}", min_roe))
            .order_by("-roe_ttm")
            .limit(limit);
        self.search_companies(query).await
    }

    /// Get dividend-paying companies
    pub async fn get_dividend_stocks(
        &self,
        min_yield: f64,
        limit: i32,
    ) -> Result<CompaniesResponse, DataSourceError> {
        let query = CompanyQuery::new()
            .where_clause(&format!("yield_ttm > {}", min_yield))
            .order_by("-yield_ttm")
            .limit(limit);
        self.search_companies(query).await
    }

    // ============================================
    // V1 API Endpoints
    // ============================================

    /// Get list of all subsectors
    pub async fn get_subsectors(&self) -> Result<Vec<Subsector>, DataSourceError> {
        let url = format!("{}/subsectors/", BASE_URL_V1);
        self.get_with_retry(&url, &[]).await
    }

    /// Get list of all industries
    pub async fn get_industries(&self) -> Result<Vec<Industry>, DataSourceError> {
        let url = format!("{}/industries/", BASE_URL_V1);
        self.get_with_retry(&url, &[]).await
    }

    /// Get daily transaction data for a stock
    pub async fn get_daily_transaction(
        &self,
        symbol: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<DailyTransaction>, DataSourceError> {
        let url = format!("{}/daily/{}/", BASE_URL_V1, symbol);
        let params = vec![
            ("start", start_date.to_string()),
            ("end", end_date.to_string()),
        ];
        self.get_with_retry(&url, &params).await
    }

    /// Get companies list by index (e.g., LQ45, IDX30)
    pub async fn get_companies_by_index(
        &self,
        index: &str,
    ) -> Result<Vec<String>, DataSourceError> {
        let url = format!("{}/index/{}/", BASE_URL_V1, index);
        self.get_with_retry(&url, &[]).await
    }

    /// Get full company report with historical financials
    pub async fn get_company_report(&self, symbol: &str) -> Result<CompanyReport, DataSourceError> {
        let url = format!("{}/companies/{}/", BASE_URL_V1, symbol);
        self.get_with_retry(&url, &[]).await
    }

    /// Get company historical financials only
    pub async fn get_historical_financials(
        &self,
        symbol: &str,
    ) -> Result<Vec<HistoricalFinancial>, DataSourceError> {
        let report = self.get_company_report(symbol).await?;
        Ok(report
            .financials
            .map(|f| f.historical_financials)
            .unwrap_or_default())
    }

    /// Check if client is properly configured
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = SectorsClient::new("test_api_key".to_string());
        assert!(client.is_ok());
        assert!(client.unwrap().is_configured());
    }

    #[test]
    fn test_empty_api_key() {
        let client = SectorsClient::new("".to_string());
        assert!(client.is_err());
    }

    #[test]
    fn test_company_query_natural() {
        let query = CompanyQuery::new().natural_query("top 10 banks by market cap");
        let params = query.to_params();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].0, "q");
    }

    #[test]
    fn test_company_query_structured() {
        let query = CompanyQuery::new()
            .where_clause("sector = 'Financials'")
            .order_by("-market_cap")
            .limit(50)
            .offset(10);

        let params = query.to_params();
        assert!(params.iter().any(|(k, _)| *k == "where"));
        assert!(params.iter().any(|(k, _)| *k == "order_by"));
        assert!(params.iter().any(|(k, _)| *k == "limit"));
        assert!(params.iter().any(|(k, _)| *k == "offset"));
    }

    #[test]
    fn test_limit_max() {
        let query = CompanyQuery::new().limit(500);
        let params = query.to_params();
        let limit = params.iter().find(|(k, _)| *k == "limit").unwrap();
        assert_eq!(limit.1, "200"); // Should be capped at 200
    }
}
