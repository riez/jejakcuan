//! Yahoo Finance HTTP client

use super::models::*;
use super::parser;
use crate::error::DataSourceError;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, warn};

const YAHOO_QUOTE_API: &str = "https://query1.finance.yahoo.com/v7/finance/quote";
const YAHOO_CHART_API: &str = "https://query1.finance.yahoo.com/v8/finance/chart";

/// Yahoo Finance API client
#[derive(Debug, Clone)]
pub struct YahooFinanceClient {
    client: Client,
}

impl YahooFinanceClient {
    /// Create a new Yahoo Finance client
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; JejakCuan/1.0)")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Convert IDX symbol to Yahoo Finance format (add .JK suffix)
    pub fn to_yahoo_symbol(symbol: &str) -> String {
        if symbol.ends_with(".JK") {
            symbol.to_string()
        } else {
            format!("{}.JK", symbol)
        }
    }

    /// Convert Yahoo Finance symbol back to IDX format (remove .JK suffix)
    pub fn from_yahoo_symbol(symbol: &str) -> String {
        symbol.trim_end_matches(".JK").to_string()
    }

    /// Get quote for a single stock
    pub async fn get_quote(&self, symbol: &str) -> Result<YahooQuote, DataSourceError> {
        let yahoo_symbol = Self::to_yahoo_symbol(symbol);
        debug!("Fetching quote for {}", yahoo_symbol);

        let url = format!("{}?symbols={}", YAHOO_QUOTE_API, yahoo_symbol);
        let response = self.client.get(&url).send().await?;

        if response.status() == 429 {
            warn!("Rate limited by Yahoo Finance");
            return Err(DataSourceError::RateLimited);
        }

        let data: QuoteResponse = response.json().await?;

        if let Some(error) = data.quote_response.error {
            return Err(DataSourceError::ApiError(error.to_string()));
        }

        let result = data
            .quote_response
            .result
            .first()
            .ok_or_else(|| DataSourceError::SymbolNotFound(symbol.to_string()))?;

        parser::parse_quote(result)
    }

    /// Get quotes for multiple stocks
    pub async fn get_quotes(&self, symbols: &[&str]) -> Result<Vec<YahooQuote>, DataSourceError> {
        let yahoo_symbols: Vec<String> = symbols.iter().map(|s| Self::to_yahoo_symbol(s)).collect();

        debug!("Fetching quotes for {} symbols", yahoo_symbols.len());

        let url = format!("{}?symbols={}", YAHOO_QUOTE_API, yahoo_symbols.join(","));
        let response = self.client.get(&url).send().await?;

        if response.status() == 429 {
            warn!("Rate limited by Yahoo Finance");
            return Err(DataSourceError::RateLimited);
        }

        let data: QuoteResponse = response.json().await?;

        if let Some(error) = data.quote_response.error {
            return Err(DataSourceError::ApiError(error.to_string()));
        }

        data.quote_response
            .result
            .iter()
            .map(parser::parse_quote)
            .collect()
    }

    /// Get historical OHLCV data
    ///
    /// # Arguments
    /// * `symbol` - Stock symbol (without .JK suffix)
    /// * `interval` - Data interval: "1d", "1wk", "1mo"
    /// * `range` - Data range: "1mo", "3mo", "6mo", "1y", "2y", "5y", "max"
    pub async fn get_history(
        &self,
        symbol: &str,
        interval: &str,
        range: &str,
    ) -> Result<Vec<YahooOHLCV>, DataSourceError> {
        let yahoo_symbol = Self::to_yahoo_symbol(symbol);
        debug!(
            "Fetching history for {} (interval={}, range={})",
            yahoo_symbol, interval, range
        );

        let url = format!(
            "{}/{}?interval={}&range={}",
            YAHOO_CHART_API, yahoo_symbol, interval, range
        );
        let response = self.client.get(&url).send().await?;

        if response.status() == 429 {
            warn!("Rate limited by Yahoo Finance");
            return Err(DataSourceError::RateLimited);
        }

        if response.status() == 404 {
            return Err(DataSourceError::SymbolNotFound(symbol.to_string()));
        }

        let data: ChartResponse = response.json().await?;

        if let Some(error) = data.chart.error {
            return Err(DataSourceError::ApiError(error.to_string()));
        }

        parser::parse_chart(data)
    }

    /// Get 1 year of daily history (convenience method)
    pub async fn get_daily_history_1y(
        &self,
        symbol: &str,
    ) -> Result<Vec<YahooOHLCV>, DataSourceError> {
        self.get_history(symbol, "1d", "1y").await
    }

    /// Get list of common IDX stocks
    /// Note: Yahoo Finance doesn't provide a stock list API, so this is hardcoded
    pub fn get_idx_stock_list() -> Vec<&'static str> {
        vec![
            // Blue chips
            "BBCA", "BBRI", "BMRI", "BBNI", "TLKM", "ASII", "UNVR", "HMSP", // Banks
            "BRIS", "BTPS", "MEGA", "BDMN", "BNGA", "PNBN", "NISP", // Telco & Tech
            "EXCL", "ISAT", "TBIG", "TOWR", "MTEL", // Consumer
            "ICBP", "INDF", "MYOR", "KLBF", "SIDO", "UNTR", // Energy & Mining
            "ADRO", "PTBA", "ITMG", "INCO", "ANTM", "TINS", "MDKA", // Property
            "BSDE", "CTRA", "SMRA", "PWON", "LPKR", // Infrastructure
            "JSMR", "WIKA", "PTPP", "WSKT", "ADHI", // Prajogo Pangestu group
            "BRPT", "TPIA", "CUAN", "BREN", "PTRO", // Other notable
            "GOTO", "BUKA", "EMTK", "SCMA", "MNCN", "ACES", "MAPI", "ERAA", "LPPF", "SMGR", "INTP",
            "WTON", "GGRM", "SRIL", "TKIM", "INKP", "PGAS", "AKRA", "MEDC", "JPFA", "CPIN", "MAIN",
            "ARTO", "BJTM", "BJBR",
        ]
    }
}

impl Default for YahooFinanceClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_conversion() {
        assert_eq!(YahooFinanceClient::to_yahoo_symbol("BBCA"), "BBCA.JK");
        assert_eq!(YahooFinanceClient::to_yahoo_symbol("BBCA.JK"), "BBCA.JK");
        assert_eq!(YahooFinanceClient::from_yahoo_symbol("BBCA.JK"), "BBCA");
        assert_eq!(YahooFinanceClient::from_yahoo_symbol("BBCA"), "BBCA");
    }

    #[test]
    fn test_stock_list_not_empty() {
        let list = YahooFinanceClient::get_idx_stock_list();
        assert!(!list.is_empty());
        assert!(list.contains(&"BBCA"));
    }
}
