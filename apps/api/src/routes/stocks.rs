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
        .route("/scores/top", get(get_top_scores))
        .route("/:symbol", get(get_stock))
        .route("/:symbol/prices", get(get_stock_prices))
        .route("/:symbol/score", get(get_stock_score))
        .route("/:symbol/fundamentals", get(get_stock_fundamentals))
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
    tracing::debug!("get_stock called with symbol: {}", symbol);
    let upper_symbol = symbol.to_uppercase();
    tracing::debug!("Looking up stock: {}", upper_symbol);
    
    let stock = repositories::stocks::get_stock_by_symbol(&state.db, &upper_symbol)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    
    tracing::debug!("Stock query result: {:?}", stock.is_some());
    
    stock.ok_or_else(|| {
        tracing::debug!("Stock not found: {}", upper_symbol);
        (
            axum::http::StatusCode::NOT_FOUND,
            format!("Stock not found: {}", upper_symbol),
        )
    }).map(Json)
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

#[derive(Debug, Serialize)]
pub struct FundamentalData {
    pub symbol: String,
    pub pe_ratio: Option<f64>,
    pub pb_ratio: Option<f64>,
    pub ps_ratio: Option<f64>,
    pub ev_ebitda: Option<f64>,
    pub roe: Option<f64>,
    pub roa: Option<f64>,
    pub profit_margin: Option<f64>,
    pub debt_to_equity: Option<f64>,
    pub current_ratio: Option<f64>,
    pub dcf_intrinsic_value: Option<f64>,
    pub dcf_margin_of_safety: Option<f64>,
    pub sector_avg_pe: Option<f64>,
    pub sector_avg_pb: Option<f64>,
}

async fn get_stock_fundamentals(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<Json<Option<FundamentalData>>, (axum::http::StatusCode, String)> {
    // Verify stock exists first
    let _stock = repositories::stocks::get_stock_by_symbol(&state.db, &symbol.to_uppercase())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                "Stock not found".to_string(),
            )
        })?;

    // TODO: Implement actual fundamental data retrieval
    // For now, return None (no fundamental data available)
    Ok(Json(None))
}
