//! Watchlist routes

use crate::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use jejakcuan_db::{repositories, WatchlistRow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    let stock_exists = repositories::stocks::get_stock_by_symbol(&state.db, &symbol)
        .await
        .map_err(|e| (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(WatchlistError {
                error: e.to_string(),
                code: "INTERNAL_ERROR".to_string(),
                symbol: symbol.clone(),
            })
        ))?;
    
    if stock_exists.is_none() {
        return Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(WatchlistError {
                error: format!("Stock '{}' not found in database. Please ensure the stock exists before adding to watchlist.", symbol),
                code: "STOCK_NOT_FOUND".to_string(),
                symbol: symbol.clone(),
            })
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
                        error: format!("Stock '{}' not found. Please add the stock to the database first.", symbol),
                        code: "STOCK_NOT_FOUND".to_string(),
                        symbol: symbol.clone(),
                    })
                )
            } else {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(WatchlistError {
                        error: e.to_string(),
                        code: "INTERNAL_ERROR".to_string(),
                        symbol: symbol.clone(),
                    })
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
