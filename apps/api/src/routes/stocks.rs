//! Stock-related routes

use crate::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use jejakcuan_db::{repositories, StockPriceRow, StockRow, StockScoreRow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn stock_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_stocks))
        .route("/{symbol}", get(get_stock))
        .route("/{symbol}/prices", get(get_stock_prices))
        .route("/{symbol}/score", get(get_stock_score))
        .route("/scores/top", get(get_top_scores))
}

#[derive(Debug, Deserialize)]
pub struct ListStocksQuery {
    sector: Option<String>,
    limit: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct StockListResponse {
    stocks: Vec<StockRow>,
    count: usize,
}

async fn list_stocks(
    _user: AuthUser, // Require authentication
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListStocksQuery>,
) -> Result<Json<StockListResponse>, (axum::http::StatusCode, String)> {
    let stocks = repositories::stocks::get_all_stocks(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Filter by sector if provided
    let filtered: Vec<StockRow> = if let Some(sector) = query.sector {
        stocks
            .into_iter()
            .filter(|s| {
                s.sector
                    .as_ref()
                    .map(|sec| sec.eq_ignore_ascii_case(&sector))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        stocks
    };

    // Apply limit
    let limited: Vec<StockRow> = if let Some(limit) = query.limit {
        filtered.into_iter().take(limit as usize).collect()
    } else {
        filtered
    };

    let count = limited.len();
    Ok(Json(StockListResponse {
        stocks: limited,
        count,
    }))
}

async fn get_stock(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<Json<StockRow>, (axum::http::StatusCode, String)> {
    let stock = repositories::stocks::get_stock_by_symbol(&state.db, &symbol.to_uppercase())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                "Stock not found".to_string(),
            )
        })?;

    Ok(Json(stock))
}

#[derive(Debug, Deserialize)]
pub struct PriceHistoryQuery {
    days: Option<i32>,
}

async fn get_stock_prices(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(query): Query<PriceHistoryQuery>,
) -> Result<Json<Vec<StockPriceRow>>, (axum::http::StatusCode, String)> {
    let days = query.days.unwrap_or(30);
    let from = chrono::Utc::now() - chrono::Duration::days(days as i64);
    let to = chrono::Utc::now();

    let prices =
        repositories::prices::get_price_history(&state.db, &symbol.to_uppercase(), from, to)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(prices))
}

async fn get_stock_score(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<Json<Option<StockScoreRow>>, (axum::http::StatusCode, String)> {
    let score = repositories::scores::get_stock_score(&state.db, &symbol.to_uppercase())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(score))
}

#[derive(Debug, Deserialize)]
pub struct TopScoresQuery {
    limit: Option<i32>,
}

async fn get_top_scores(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<TopScoresQuery>,
) -> Result<Json<Vec<StockScoreRow>>, (axum::http::StatusCode, String)> {
    let limit = query.limit.unwrap_or(50);
    let scores = repositories::scores::get_latest_scores(&state.db, limit)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(scores))
}
