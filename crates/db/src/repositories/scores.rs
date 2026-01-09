//! Score repository

use crate::models::StockScoreRow;
use sqlx::PgPool;

/// Get latest scores for all stocks
pub async fn get_latest_scores(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<StockScoreRow>, sqlx::Error> {
    sqlx::query_as::<_, StockScoreRow>(
        r#"
        SELECT DISTINCT ON (symbol) *
        FROM stock_scores
        ORDER BY symbol, time DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Get latest score for a stock
pub async fn get_stock_score(
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<StockScoreRow>, sqlx::Error> {
    sqlx::query_as::<_, StockScoreRow>(
        "SELECT * FROM stock_scores WHERE symbol = $1 ORDER BY time DESC LIMIT 1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await
}
