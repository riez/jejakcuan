//! Stock repository

use crate::models::StockRow;
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
