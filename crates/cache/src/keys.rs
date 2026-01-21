//! Cache key generators for consistent key naming

/// Cache key prefixes
pub mod prefix {
    pub const STOCK_QUOTE: &str = "stock:quote";
    pub const STOCK_PRICE: &str = "stock:price";
    pub const TECHNICAL_SCORE: &str = "score:technical";
    pub const FUNDAMENTAL_SCORE: &str = "score:fundamental";
    pub const COMPOSITE_SCORE: &str = "score:composite";
    pub const BROKER_FLOW: &str = "broker:flow";
    pub const ALERT: &str = "alert";
    pub const WATCHLIST: &str = "watchlist";
    pub const USER_SESSION: &str = "session";
    pub const RATE_LIMIT: &str = "ratelimit";
    pub const LEADERBOARD: &str = "leaderboard";
}

/// Generate cache keys for various entities
pub struct CacheKeys;

impl CacheKeys {
    /// Stock quote key: stock:quote:{symbol}
    pub fn stock_quote(symbol: &str) -> String {
        format!("{}:{}", prefix::STOCK_QUOTE, symbol.to_uppercase())
    }

    /// Stock price history key: stock:price:{symbol}:{timeframe}
    pub fn stock_price(symbol: &str, timeframe: &str) -> String {
        format!(
            "{}:{}:{}",
            prefix::STOCK_PRICE,
            symbol.to_uppercase(),
            timeframe
        )
    }

    /// Technical score key: score:technical:{symbol}
    pub fn technical_score(symbol: &str) -> String {
        format!("{}:{}", prefix::TECHNICAL_SCORE, symbol.to_uppercase())
    }

    /// Fundamental score key: score:fundamental:{symbol}
    pub fn fundamental_score(symbol: &str) -> String {
        format!("{}:{}", prefix::FUNDAMENTAL_SCORE, symbol.to_uppercase())
    }

    /// Composite score key: score:composite:{symbol}
    pub fn composite_score(symbol: &str) -> String {
        format!("{}:{}", prefix::COMPOSITE_SCORE, symbol.to_uppercase())
    }

    /// Broker flow key: broker:flow:{symbol}:{date}
    pub fn broker_flow(symbol: &str, date: &str) -> String {
        format!("{}:{}:{}", prefix::BROKER_FLOW, symbol.to_uppercase(), date)
    }

    /// Alert key: alert:{alert_id}
    pub fn alert(alert_id: &str) -> String {
        format!("{}:{}", prefix::ALERT, alert_id)
    }

    /// User alerts key: alert:user:{user_id}
    pub fn user_alerts(user_id: &str) -> String {
        format!("{}:user:{}", prefix::ALERT, user_id)
    }

    /// Watchlist key: watchlist:{user_id}
    pub fn watchlist(user_id: &str) -> String {
        format!("{}:{}", prefix::WATCHLIST, user_id)
    }

    /// User session key: session:{session_id}
    pub fn user_session(session_id: &str) -> String {
        format!("{}:{}", prefix::USER_SESSION, session_id)
    }

    /// Rate limit key: ratelimit:{resource}:{identifier}
    pub fn rate_limit(resource: &str, identifier: &str) -> String {
        format!("{}:{}:{}", prefix::RATE_LIMIT, resource, identifier)
    }

    /// Leaderboard key: leaderboard:{category}
    pub fn leaderboard(category: &str) -> String {
        format!("{}:{}", prefix::LEADERBOARD, category)
    }

    /// Pattern for wildcard matching
    pub fn pattern(prefix: &str, symbol: Option<&str>) -> String {
        match symbol {
            Some(s) => format!("{}:{}:*", prefix, s.to_uppercase()),
            None => format!("{}:*", prefix),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_quote_key() {
        assert_eq!(CacheKeys::stock_quote("bbca"), "stock:quote:BBCA");
        assert_eq!(CacheKeys::stock_quote("BBRI"), "stock:quote:BBRI");
    }

    #[test]
    fn test_stock_price_key() {
        assert_eq!(
            CacheKeys::stock_price("bbca", "daily"),
            "stock:price:BBCA:daily"
        );
    }

    #[test]
    fn test_technical_score_key() {
        assert_eq!(CacheKeys::technical_score("tlkm"), "score:technical:TLKM");
    }

    #[test]
    fn test_broker_flow_key() {
        assert_eq!(
            CacheKeys::broker_flow("asii", "2024-01-15"),
            "broker:flow:ASII:2024-01-15"
        );
    }

    #[test]
    fn test_rate_limit_key() {
        assert_eq!(
            CacheKeys::rate_limit("api", "192.168.1.1"),
            "ratelimit:api:192.168.1.1"
        );
    }

    #[test]
    fn test_pattern_generation() {
        assert_eq!(
            CacheKeys::pattern(prefix::STOCK_QUOTE, Some("bbca")),
            "stock:quote:BBCA:*"
        );
        assert_eq!(
            CacheKeys::pattern(prefix::STOCK_QUOTE, None),
            "stock:quote:*"
        );
    }
}
