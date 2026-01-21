//! Stock-specific caching operations

use crate::{CacheClient, CacheError, CacheKeys, CacheResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cached stock quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedQuote {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: i64,
    pub timestamp: i64,
}

/// Cached technical score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTechnicalScore {
    pub symbol: String,
    pub score: f64,
    pub signals: Vec<String>,
    pub updated_at: i64,
}

/// Cached broker flow summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBrokerFlow {
    pub symbol: String,
    pub date: String,
    pub net_foreign: f64,
    pub net_institutional: f64,
    pub accumulation_score: f64,
    pub top_buyers: Vec<String>,
    pub top_sellers: Vec<String>,
}

/// Stock cache service
pub struct StockCache {
    client: CacheClient,
    quote_ttl: Duration,
    score_ttl: Duration,
}

impl StockCache {
    /// Create new stock cache with default TTLs
    pub fn new(client: CacheClient) -> Self {
        Self {
            client,
            quote_ttl: Duration::from_secs(30), // 30 seconds for quotes
            score_ttl: Duration::from_secs(300), // 5 minutes for scores
        }
    }

    /// Create with custom TTLs
    pub fn with_ttls(client: CacheClient, quote_ttl: Duration, score_ttl: Duration) -> Self {
        Self {
            client,
            quote_ttl,
            score_ttl,
        }
    }

    // Quote operations

    /// Get cached quote
    pub async fn get_quote(&mut self, symbol: &str) -> CacheResult<Option<CachedQuote>> {
        let key = CacheKeys::stock_quote(symbol);
        self.client.get(&key).await
    }

    /// Set quote in cache
    pub async fn set_quote(&mut self, quote: &CachedQuote) -> CacheResult<()> {
        let key = CacheKeys::stock_quote(&quote.symbol);
        self.client.set_with_ttl(&key, quote, self.quote_ttl).await
    }

    /// Get multiple quotes
    pub async fn get_quotes(&mut self, symbols: &[&str]) -> CacheResult<Vec<Option<CachedQuote>>> {
        let keys: Vec<String> = symbols.iter().map(|s| CacheKeys::stock_quote(s)).collect();
        let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
        self.client.mget(&key_refs).await
    }

    // Technical score operations

    /// Get cached technical score
    pub async fn get_technical_score(
        &mut self,
        symbol: &str,
    ) -> CacheResult<Option<CachedTechnicalScore>> {
        let key = CacheKeys::technical_score(symbol);
        self.client.get(&key).await
    }

    /// Set technical score in cache
    pub async fn set_technical_score(&mut self, score: &CachedTechnicalScore) -> CacheResult<()> {
        let key = CacheKeys::technical_score(&score.symbol);
        self.client.set_with_ttl(&key, score, self.score_ttl).await
    }

    // Broker flow operations

    /// Get cached broker flow
    pub async fn get_broker_flow(
        &mut self,
        symbol: &str,
        date: &str,
    ) -> CacheResult<Option<CachedBrokerFlow>> {
        let key = CacheKeys::broker_flow(symbol, date);
        self.client.get(&key).await
    }

    /// Set broker flow in cache
    pub async fn set_broker_flow(&mut self, flow: &CachedBrokerFlow) -> CacheResult<()> {
        let key = CacheKeys::broker_flow(&flow.symbol, &flow.date);
        // Broker flow data is daily, cache for longer
        self.client
            .set_with_ttl(&key, flow, Duration::from_secs(3600))
            .await
    }

    // Utility operations

    /// Invalidate all cached data for a symbol
    pub async fn invalidate_symbol(&mut self, symbol: &str) -> CacheResult<()> {
        let keys = [
            CacheKeys::stock_quote(symbol),
            CacheKeys::technical_score(symbol),
            CacheKeys::fundamental_score(symbol),
            CacheKeys::composite_score(symbol),
        ];

        for key in keys {
            let _ = self.client.delete(&key).await;
        }

        Ok(())
    }

    /// Update leaderboard (top movers, etc.)
    pub async fn update_leaderboard(
        &mut self,
        category: &str,
        symbol: &str,
        score: f64,
    ) -> CacheResult<()> {
        let key = CacheKeys::leaderboard(category);
        self.client.zadd(&key, symbol, score).await
    }

    /// Get leaderboard top N
    pub async fn get_leaderboard(
        &mut self,
        category: &str,
        top_n: usize,
    ) -> CacheResult<Vec<String>> {
        let key = CacheKeys::leaderboard(category);
        self.client.zrevrange(&key, 0, (top_n - 1) as isize).await
    }

    /// Get underlying client for raw operations
    pub fn client(&mut self) -> &mut CacheClient {
        &mut self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_quote_serialization() {
        let quote = CachedQuote {
            symbol: "BBCA".to_string(),
            price: 9500.0,
            change: 100.0,
            change_percent: 1.06,
            volume: 10000000,
            timestamp: 1705315200,
        };

        let json = serde_json::to_string(&quote).unwrap();
        let parsed: CachedQuote = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.symbol, "BBCA");
        assert_eq!(parsed.price, 9500.0);
    }

    #[test]
    fn test_cached_technical_score_serialization() {
        let score = CachedTechnicalScore {
            symbol: "BBRI".to_string(),
            score: 75.5,
            signals: vec!["RSI oversold".to_string(), "MACD bullish".to_string()],
            updated_at: 1705315200,
        };

        let json = serde_json::to_string(&score).unwrap();
        let parsed: CachedTechnicalScore = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.symbol, "BBRI");
        assert_eq!(parsed.signals.len(), 2);
    }

    #[test]
    fn test_cached_broker_flow_serialization() {
        let flow = CachedBrokerFlow {
            symbol: "TLKM".to_string(),
            date: "2024-01-15".to_string(),
            net_foreign: 15_000_000_000.0,
            net_institutional: 5_000_000_000.0,
            accumulation_score: 78.0,
            top_buyers: vec!["BK".to_string(), "CC".to_string()],
            top_sellers: vec!["YP".to_string()],
        };

        let json = serde_json::to_string(&flow).unwrap();
        let parsed: CachedBrokerFlow = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.symbol, "TLKM");
        assert_eq!(parsed.net_foreign, 15_000_000_000.0);
    }
}
