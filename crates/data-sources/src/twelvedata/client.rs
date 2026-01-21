//! TwelveData REST API client implementation

use super::models::*;
use crate::error::DataSourceError;
use chrono::NaiveDate;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, warn};

const BASE_URL: &str = "https://api.twelvedata.com";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RETRIES: u32 = 3;

/// TwelveData REST API client
#[derive(Debug, Clone)]
pub struct TwelveDataClient {
    client: Client,
    api_key: String,
}

impl TwelveDataClient {
    /// Create a new TwelveData client
    pub fn new(api_key: String) -> Result<Self, DataSourceError> {
        if api_key.is_empty() {
            return Err(DataSourceError::InvalidResponse(
                "TwelveData API key is required".to_string(),
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

    /// Create client from TWELVEDATA_API_KEY environment variable
    pub fn from_env() -> Result<Self, DataSourceError> {
        let api_key = std::env::var("TWELVEDATA_API_KEY").map_err(|_| {
            DataSourceError::InvalidResponse(
                "TWELVEDATA_API_KEY environment variable not set".into(),
            )
        })?;
        Self::new(api_key)
    }

    /// Get API key for WebSocket connection
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Execute GET request with retry logic
    async fn get_with_retry<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T, DataSourceError> {
        let url = format!("{}/{}", BASE_URL, endpoint);
        let mut last_error = None;
        let mut backoff_ms = 1000u64;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                debug!("Retry attempt {} for {}", attempt, endpoint);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                backoff_ms *= 2;
            }

            let mut request = self.client.get(&url);
            for (key, value) in params {
                request = request.query(&[(key, value)]);
            }
            request = request.query(&[("apikey", self.api_key.as_str())]);

            match request.send().await {
                Ok(response) => {
                    let status = response.status();

                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        warn!("Rate limited by TwelveData API");
                        last_error = Some(DataSourceError::RateLimited);
                        continue;
                    }

                    if status.is_server_error() {
                        warn!("Server error from TwelveData: {}", status);
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
                    warn!("Network error: {}", e);
                    last_error = Some(DataSourceError::HttpError(e));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| DataSourceError::ApiError("Max retries exceeded".into())))
    }

    /// Get time series data
    pub async fn time_series(
        &self,
        symbol: &str,
        interval: Interval,
        output_size: Option<i32>,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<TimeSeriesResponse, DataSourceError> {
        let mut params = vec![("symbol", symbol), ("interval", interval.as_str())];

        let output_str;
        if let Some(size) = output_size {
            output_str = size.to_string();
            params.push(("outputsize", &output_str));
        }

        let start_str;
        if let Some(date) = start_date {
            start_str = date.to_string();
            params.push(("start_date", &start_str));
        }

        let end_str;
        if let Some(date) = end_date {
            end_str = date.to_string();
            params.push(("end_date", &end_str));
        }

        self.get_with_retry("time_series", &params).await
    }

    /// Get latest price
    pub async fn price(&self, symbol: &str) -> Result<LatestPrice, DataSourceError> {
        let params = vec![("symbol", symbol)];
        self.get_with_retry("price", &params).await
    }

    /// Get quote
    pub async fn quote(&self, symbol: &str) -> Result<Quote, DataSourceError> {
        let params = vec![("symbol", symbol)];
        self.get_with_retry("quote", &params).await
    }

    /// Get multiple quotes
    pub async fn quotes(&self, symbols: &[&str]) -> Result<Vec<Quote>, DataSourceError> {
        let symbols_str = symbols.join(",");
        let params = vec![("symbol", symbols_str.as_str())];
        self.get_with_retry("quote", &params).await
    }

    /// Get market movers
    pub async fn market_movers(
        &self,
        direction: &str,
        output_size: Option<i32>,
    ) -> Result<MarketMoversResponse, DataSourceError> {
        let mut params = vec![("direction", direction)];
        let output_str;
        if let Some(size) = output_size {
            output_str = size.to_string();
            params.push(("outputsize", &output_str));
        }
        self.get_with_retry("market_movers", &params).await
    }

    /// Get stock list for exchange
    pub async fn stocks(&self, exchange: &str) -> Result<Vec<StockInfo>, DataSourceError> {
        let params = vec![("exchange", exchange)];
        self.get_with_retry("stocks", &params).await
    }

    /// Search symbols
    pub async fn symbol_search(&self, query: &str) -> Result<Vec<StockInfo>, DataSourceError> {
        let params = vec![("symbol", query)];
        self.get_with_retry("symbol_search", &params).await
    }

    /// Get exchanges list
    pub async fn exchanges(&self) -> Result<Vec<ExchangeInfo>, DataSourceError> {
        self.get_with_retry("exchanges", &[]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = TwelveDataClient::new("test_api_key".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_empty_api_key() {
        let client = TwelveDataClient::new("".to_string());
        assert!(client.is_err());
    }

    #[test]
    fn test_interval_display() {
        assert_eq!(Interval::Min1.as_str(), "1min");
        assert_eq!(Interval::Hour4.as_str(), "4h");
        assert_eq!(Interval::Day1.as_str(), "1day");
        assert_eq!(Interval::Month1.as_str(), "1month");
    }
}
