//! Stock repository

use crate::models::{FinancialsRow, StockRow};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Get all active stocks
pub async fn get_all_stocks(pool: &PgPool) -> Result<Vec<StockRow>, sqlx::Error> {
    sqlx::query_as::<_, StockRow>("SELECT * FROM stocks WHERE is_active = true ORDER BY symbol")
        .fetch_all(pool)
        .await
}

/// Get stock by symbol
pub async fn get_stock_by_symbol(
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<StockRow>, sqlx::Error> {
    sqlx::query_as::<_, StockRow>("SELECT * FROM stocks WHERE symbol = $1")
        .bind(symbol)
        .fetch_optional(pool)
        .await
}

/// Insert or update stock
pub async fn upsert_stock(
    pool: &PgPool,
    symbol: &str,
    name: &str,
    sector: Option<&str>,
    subsector: Option<&str>,
) -> Result<StockRow, sqlx::Error> {
    sqlx::query_as::<_, StockRow>(
        r#"
        INSERT INTO stocks (symbol, name, sector, subsector)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (symbol) DO UPDATE SET
            name = EXCLUDED.name,
            sector = EXCLUDED.sector,
            subsector = EXCLUDED.subsector,
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(symbol)
    .bind(name)
    .bind(sector)
    .bind(subsector)
    .fetch_one(pool)
    .await
}

/// Get latest financials for a stock
pub async fn get_financials(
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<FinancialsRow>, sqlx::Error> {
    sqlx::query_as::<_, FinancialsRow>(
        "SELECT * FROM financials WHERE symbol = $1 ORDER BY period_end DESC LIMIT 1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await
}

pub async fn get_latest_financials_created_at(
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
        "SELECT MAX(created_at) FROM financials WHERE symbol = $1",
    )
    .bind(symbol)
    .fetch_one(pool)
    .await
}
