//! Watchlist repository

use crate::models::WatchlistRow;
use sqlx::PgPool;

/// Get all watchlist items
pub async fn get_watchlist(pool: &PgPool) -> Result<Vec<WatchlistRow>, sqlx::Error> {
    sqlx::query_as::<_, WatchlistRow>("SELECT * FROM watchlist ORDER BY sort_order")
        .fetch_all(pool)
        .await
}

/// Add stock to watchlist
pub async fn add_to_watchlist(pool: &PgPool, symbol: &str) -> Result<WatchlistRow, sqlx::Error> {
    sqlx::query_as::<_, WatchlistRow>(
        r#"
        INSERT INTO watchlist (symbol, sort_order)
        VALUES ($1, (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM watchlist))
        ON CONFLICT (symbol) DO NOTHING
        RETURNING *
        "#,
    )
    .bind(symbol)
    .fetch_one(pool)
    .await
}

/// Remove stock from watchlist
pub async fn remove_from_watchlist(pool: &PgPool, symbol: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM watchlist WHERE symbol = $1")
        .bind(symbol)
        .execute(pool)
        .await?;
    Ok(())
}
