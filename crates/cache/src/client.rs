//! Redis client wrapper

use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, RedisError};
use std::time::Duration;

/// Cache error types
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Redis connection error: {0}")]
    Connection(#[from] RedisError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Key not found: {0}")]
    NotFound(String),
}

pub type CacheResult<T> = Result<T, CacheError>;

/// Redis cache client with connection pooling
pub struct CacheClient {
    conn: ConnectionManager,
    default_ttl: Duration,
}

impl CacheClient {
    /// Create new cache client
    pub async fn new(redis_url: &str) -> CacheResult<Self> {
        let client = Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;

        Ok(Self {
            conn,
            default_ttl: Duration::from_secs(300), // 5 minutes default
        })
    }

    /// Create with custom TTL
    pub async fn with_ttl(redis_url: &str, ttl: Duration) -> CacheResult<Self> {
        let client = Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;

        Ok(Self {
            conn,
            default_ttl: ttl,
        })
    }

    /// Get a value from cache
    pub async fn get<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> CacheResult<Option<T>> {
        let value: Option<String> = self.conn.get(key).await?;

        match value {
            Some(json) => {
                let parsed = serde_json::from_str(&json)?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Set a value with default TTL
    pub async fn set<T: serde::Serialize>(&mut self, key: &str, value: &T) -> CacheResult<()> {
        let json = serde_json::to_string(value)?;
        self.conn
            .set_ex(key, json, self.default_ttl.as_secs())
            .await?;
        Ok(())
    }

    /// Set a value with custom TTL
    pub async fn set_with_ttl<T: serde::Serialize>(
        &mut self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> CacheResult<()> {
        let json = serde_json::to_string(value)?;
        self.conn.set_ex(key, json, ttl.as_secs()).await?;
        Ok(())
    }

    /// Delete a key
    pub async fn delete(&mut self, key: &str) -> CacheResult<bool> {
        let deleted: i32 = self.conn.del(key).await?;
        Ok(deleted > 0)
    }

    /// Check if key exists
    pub async fn exists(&mut self, key: &str) -> CacheResult<bool> {
        let exists: bool = self.conn.exists(key).await?;
        Ok(exists)
    }

    /// Get TTL of a key
    pub async fn ttl(&mut self, key: &str) -> CacheResult<i64> {
        let ttl: i64 = self.conn.ttl(key).await?;
        Ok(ttl)
    }

    /// Set multiple values at once
    pub async fn mset<T: serde::Serialize>(
        &mut self,
        pairs: &[(&str, &T)],
    ) -> CacheResult<()> {
        let serialized: Vec<(&str, String)> = pairs
            .iter()
            .map(|(k, v)| (*k, serde_json::to_string(v).unwrap_or_default()))
            .collect();

        let refs: Vec<(&str, &str)> = serialized.iter().map(|(k, v)| (*k, v.as_str())).collect();
        
        redis::cmd("MSET")
            .arg(&refs[..])
            .query_async(&mut self.conn)
            .await?;

        // Set TTL for each key
        for (key, _) in pairs {
            self.conn
                .expire(*key, self.default_ttl.as_secs() as i64)
                .await?;
        }

        Ok(())
    }

    /// Get multiple values at once
    pub async fn mget<T: serde::de::DeserializeOwned>(
        &mut self,
        keys: &[&str],
    ) -> CacheResult<Vec<Option<T>>> {
        let values: Vec<Option<String>> = self.conn.mget(keys).await?;

        let results: Vec<Option<T>> = values
            .into_iter()
            .map(|v| v.and_then(|json| serde_json::from_str(&json).ok()))
            .collect();

        Ok(results)
    }

    /// Increment a counter
    pub async fn incr(&mut self, key: &str) -> CacheResult<i64> {
        let val: i64 = self.conn.incr(key, 1).await?;
        Ok(val)
    }

    /// Add to a sorted set (for leaderboards, rankings)
    pub async fn zadd(&mut self, key: &str, member: &str, score: f64) -> CacheResult<()> {
        self.conn.zadd(key, member, score).await?;
        Ok(())
    }

    /// Get top N from sorted set
    pub async fn zrevrange(&mut self, key: &str, start: isize, stop: isize) -> CacheResult<Vec<String>> {
        let members: Vec<String> = self.conn.zrevrange(key, start, stop).await?;
        Ok(members)
    }

    /// Publish message to channel
    pub async fn publish(&mut self, channel: &str, message: &str) -> CacheResult<()> {
        self.conn.publish(channel, message).await?;
        Ok(())
    }

    /// Get underlying connection for raw operations
    pub fn connection(&mut self) -> &mut ConnectionManager {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running Redis instance
    // Run with: cargo test -p jejakcuan-cache -- --ignored

    #[tokio::test]
    #[ignore]
    async fn test_set_get() {
        let mut cache = CacheClient::new("redis://localhost:6379").await.unwrap();

        cache.set("test_key", &"test_value").await.unwrap();
        let value: Option<String> = cache.get("test_key").await.unwrap();

        assert_eq!(value, Some("test_value".to_string()));
        cache.delete("test_key").await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_exists() {
        let mut cache = CacheClient::new("redis://localhost:6379").await.unwrap();

        cache.set("exists_test", &123).await.unwrap();
        assert!(cache.exists("exists_test").await.unwrap());
        assert!(!cache.exists("nonexistent").await.unwrap());

        cache.delete("exists_test").await.unwrap();
    }
}
