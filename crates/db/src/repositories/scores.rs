//! Score repository

use crate::models::StockScoreRow;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

/// Score data for insertion
#[derive(Debug, Clone)]
pub struct InsertStockScore {
    pub time: DateTime<Utc>,
    pub symbol: String,
    pub composite_score: Decimal,
    pub technical_score: Decimal,
    pub fundamental_score: Decimal,
    pub sentiment_score: Decimal,
    pub ml_score: Decimal,
    pub technical_breakdown: Option<serde_json::Value>,
    pub fundamental_breakdown: Option<serde_json::Value>,
    pub sentiment_breakdown: Option<serde_json::Value>,
    pub ml_breakdown: Option<serde_json::Value>,
}

/// Get latest scores for all stocks
pub async fn get_latest_scores(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<StockScoreRow>, sqlx::Error> {
    sqlx::query_as::<_, StockScoreRow>(
        r#"
        SELECT *
        FROM (
            SELECT DISTINCT ON (symbol) *
            FROM stock_scores
            ORDER BY symbol, time DESC
        ) latest
        ORDER BY composite_score DESC, time DESC
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

/// Insert a computed score snapshot
pub async fn insert_stock_score(
    pool: &PgPool,
    score: &InsertStockScore,
) -> Result<StockScoreRow, sqlx::Error> {
    sqlx::query_as::<_, StockScoreRow>(
        r#"
        INSERT INTO stock_scores (
            time,
            symbol,
            composite_score,
            technical_score,
            fundamental_score,
            sentiment_score,
            ml_score,
            technical_breakdown,
            fundamental_breakdown,
            sentiment_breakdown,
            ml_breakdown
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        RETURNING *
        "#,
    )
    .bind(score.time)
    .bind(&score.symbol)
    .bind(score.composite_score)
    .bind(score.technical_score)
    .bind(score.fundamental_score)
    .bind(score.sentiment_score)
    .bind(score.ml_score)
    .bind(score.technical_breakdown.clone())
    .bind(score.fundamental_breakdown.clone())
    .bind(score.sentiment_breakdown.clone())
    .bind(score.ml_breakdown.clone())
    .fetch_one(pool)
    .await
}
