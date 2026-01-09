//! Price data repository

use crate::models::StockPriceRow;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

/// Price data for insertion
pub struct InsertPrice<'a> {
    pub time: DateTime<Utc>,
    pub symbol: &'a str,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
}

/// Get latest price for a stock
pub async fn get_latest_price(
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<StockPriceRow>, sqlx::Error> {
    sqlx::query_as::<_, StockPriceRow>(
        "SELECT * FROM stock_prices WHERE symbol = $1 ORDER BY time DESC LIMIT 1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await
}

/// Get price history for a stock
pub async fn get_price_history(
    pool: &PgPool,
    symbol: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<StockPriceRow>, sqlx::Error> {
    sqlx::query_as::<_, StockPriceRow>(
        "SELECT * FROM stock_prices WHERE symbol = $1 AND time >= $2 AND time <= $3 ORDER BY time",
    )
    .bind(symbol)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}

/// Insert price data
pub async fn insert_price(pool: &PgPool, price: &InsertPrice<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO stock_prices (time, symbol, open, high, low, close, volume)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(price.time)
    .bind(price.symbol)
    .bind(price.open)
    .bind(price.high)
    .bind(price.low)
    .bind(price.close)
    .bind(price.volume)
    .execute(pool)
    .await?;
    Ok(())
}
