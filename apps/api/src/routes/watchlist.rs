//! Watchlist routes

use crate::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use jejakcuan_db::{repositories, StockRow, WatchlistRow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const SYARIAH_BANK_ALLOWLIST: &[&str] = &["BRIS", "BTPS", "PNBS"];

fn is_excluded_non_syariah_bank(stock: &StockRow) -> bool {
    let is_bank = stock
        .sector
        .as_deref()
        .map(|s| s.eq_ignore_ascii_case("Banking") || s.eq_ignore_ascii_case("Financials"))
        .unwrap_or(false)
        && stock
            .subsector
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("Bank") || s.eq_ignore_ascii_case("Banks"))
            .unwrap_or(false);

    if !is_bank {
        return false;
    }

    !SYARIAH_BANK_ALLOWLIST
        .iter()
        .any(|allowed| stock.symbol.eq_ignore_ascii_case(allowed))
}

pub fn watchlist_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_watchlist))
        .route("/", post(add_to_watchlist))
        .route("/:symbol", delete(remove_from_watchlist))
}

async fn get_watchlist(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<WatchlistRow>>, (axum::http::StatusCode, String)> {
    let watchlist = repositories::watchlist::get_watchlist(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(watchlist))
}

#[derive(Debug, Deserialize)]
pub struct AddToWatchlistRequest {
    symbol: String,
}

#[derive(Debug, Serialize)]
pub struct WatchlistError {
    error: String,
    code: String,
    symbol: String,
}

async fn add_to_watchlist(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddToWatchlistRequest>,
) -> Result<Json<WatchlistRow>, (axum::http::StatusCode, Json<WatchlistError>)> {
    let symbol = req.symbol.to_uppercase();

    // First, check if the stock exists in the database
    let stock = repositories::stocks::get_stock_by_symbol(&state.db, &symbol)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(WatchlistError {
                    error: e.to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                    symbol: symbol.clone(),
                }),
            )
        })?;

    let Some(stock) = stock else {
        return Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(WatchlistError {
                error: format!("Stock '{}' not found in database. Please ensure the stock exists before adding to watchlist.", symbol),
                code: "STOCK_NOT_FOUND".to_string(),
                symbol: symbol.clone(),
            })
        ));
    };

    if is_excluded_non_syariah_bank(&stock) {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(WatchlistError {
                error: format!("Stock '{}' is excluded (non-Syariah bank).", symbol),
                code: "EXCLUDED_NON_SYARIAH_BANK".to_string(),
                symbol: symbol.clone(),
            }),
        ));
    }

    // Stock exists, proceed to add to watchlist
    let item = repositories::watchlist::add_to_watchlist(&state.db, &symbol)
        .await
        .map_err(|e| {
            // Check if it's a foreign key constraint error
            let err_str = e.to_string();
            if err_str.contains("foreign key") || err_str.contains("fk_watchlist_symbol") {
                (
                    axum::http::StatusCode::NOT_FOUND,
                    Json(WatchlistError {
                        error: format!(
                            "Stock '{}' not found. Please add the stock to the database first.",
                            symbol
                        ),
                        code: "STOCK_NOT_FOUND".to_string(),
                        symbol: symbol.clone(),
                    }),
                )
            } else {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(WatchlistError {
                        error: e.to_string(),
                        code: "INTERNAL_ERROR".to_string(),
                        symbol: symbol.clone(),
                    }),
                )
            }
        })?;

    Ok(Json(item))
}

async fn remove_from_watchlist(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    repositories::watchlist::remove_from_watchlist(&state.db, &symbol.to_uppercase())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({ "success": true })))
}
