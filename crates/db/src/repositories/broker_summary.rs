//! Broker summary repository

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, FromRow)]
pub struct BrokerFlowAggregateRow {
    pub broker_code: String,
    pub broker_name: Option<String>,
    pub category: String,
    pub buy_volume: i64,
    pub sell_volume: i64,
    pub buy_value: Decimal,
    pub sell_value: Decimal,
    pub net_volume: i64,
    pub net_value: Decimal,
}

#[derive(Debug, Clone, FromRow)]
pub struct PriceRangeRow {
    pub low: Option<Decimal>,
    pub high: Option<Decimal>,
}

pub async fn get_broker_flow_aggregates(
    pool: &PgPool,
    symbol: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<BrokerFlowAggregateRow>, sqlx::Error> {
    sqlx::query_as::<_, BrokerFlowAggregateRow>(
        r#"
        SELECT
            bs.broker_code,
            b.name AS broker_name,
            COALESCE(b.category, 'unknown') AS category,
            SUM(bs.buy_volume)::bigint AS buy_volume,
            SUM(bs.sell_volume)::bigint AS sell_volume,
            SUM(bs.buy_value) AS buy_value,
            SUM(bs.sell_value) AS sell_value,
            SUM(bs.net_volume)::bigint AS net_volume,
            SUM(bs.net_value) AS net_value
        FROM broker_summary bs
        LEFT JOIN brokers b ON b.code = bs.broker_code
        WHERE bs.symbol = $1 AND bs.time >= $2 AND bs.time <= $3
        GROUP BY bs.broker_code, b.name, b.category
        ORDER BY net_value DESC
        "#,
    )
    .bind(symbol)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}

pub async fn get_price_range(
    pool: &PgPool,
    symbol: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<PriceRangeRow, sqlx::Error> {
    sqlx::query_as::<_, PriceRangeRow>(
        r#"
        SELECT
            MIN(low) AS low,
            MAX(high) AS high
        FROM stock_prices
        WHERE symbol = $1 AND time >= $2 AND time <= $3
        "#,
    )
    .bind(symbol)
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await
}

pub async fn get_latest_broker_summary_time(
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
        "SELECT MAX(time) FROM broker_summary WHERE symbol = $1",
    )
    .bind(symbol)
    .fetch_one(pool)
    .await
}
