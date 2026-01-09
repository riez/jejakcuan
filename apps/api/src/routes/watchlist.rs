//! Watchlist routes

use crate::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use jejakcuan_db::{repositories, WatchlistRow};
use serde::Deserialize;
use std::sync::Arc;

pub fn watchlist_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_watchlist))
        .route("/", post(add_to_watchlist))
        .route("/{symbol}", delete(remove_from_watchlist))
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

async fn add_to_watchlist(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddToWatchlistRequest>,
) -> Result<Json<WatchlistRow>, (axum::http::StatusCode, String)> {
    let item = repositories::watchlist::add_to_watchlist(&state.db, &req.symbol.to_uppercase())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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
