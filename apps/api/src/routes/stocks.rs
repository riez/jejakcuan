//! Stock-related routes

use crate::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use futures_util::StreamExt;
use jejakcuan_core::{
    calculate_composite_score, FundamentalInput, FundamentalScoreEngine, ScoreWeights,
    TechnicalScoreEngine, TechnicalScoreInput,
};
use jejakcuan_db::{repositories, StockPriceRow, StockRow, StockScoreRow};
use jejakcuan_technical::{
    calculate_ema20, calculate_ema50, calculate_macd, calculate_ohlc_imbalance_proxy,
    calculate_rsi14, calculate_trend_normalized,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

pub fn stock_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_stocks))
        .route("/scores/top", get(get_top_scores))
        .route("/scores/recompute", post(recompute_scores))
        .route("/:symbol", get(get_stock))
        .route("/:symbol/prices", get(get_stock_prices))
        .route("/:symbol/score", get(get_stock_score))
        .route("/:symbol/fundamentals", get(get_stock_fundamentals))
        .route("/:symbol/freshness", get(get_stock_freshness))
}

const SCORE_STALE_HOURS: i64 = 24;

const SYARIAH_BANK_ALLOWLIST: &[&str] = &["BRIS", "BTPS", "PNBS"];

const ORDERFLOW_TREND_WINDOW: usize = 20;

fn compute_orderflow_inputs_from_ohlcv(
    highs: &[Decimal],
    lows: &[Decimal],
    closes: &[Decimal],
    volumes: &[i64],
) -> (Option<Decimal>, Option<Decimal>) {
    let min_len = highs
        .len()
        .min(lows.len())
        .min(closes.len())
        .min(volumes.len());

    if min_len < 2 {
        return (None, None);
    }

    let last_idx = min_len - 1;
    let obi = calculate_ohlc_imbalance_proxy(
        highs[last_idx],
        lows[last_idx],
        closes[last_idx],
        volumes[last_idx],
    );

    let start_idx = min_len.saturating_sub(ORDERFLOW_TREND_WINDOW);

    let mut series = Vec::with_capacity(min_len - start_idx);
    for idx in start_idx..min_len {
        series.push(calculate_ohlc_imbalance_proxy(
            highs[idx],
            lows[idx],
            closes[idx],
            volumes[idx],
        ));
    }

    let ofi_trend = calculate_trend_normalized(&series);

    (Some(obi), Some(ofi_trend))
}

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

    // Exclude non-Syariah bank stocks from the default universe
    let filtered: Vec<StockRow> = filtered
        .into_iter()
        .filter(|s| !is_excluded_non_syariah_bank(s))
        .collect();

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

    stock
        .ok_or_else(|| {
            tracing::debug!("Stock not found: {}", upper_symbol);
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Stock not found: {}", upper_symbol),
            )
        })
        .map(Json)
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
    let upper_symbol = symbol.to_uppercase();

    // Verify stock exists
    repositories::stocks::get_stock_by_symbol(&state.db, &upper_symbol)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                "Stock not found".to_string(),
            )
        })?;

    let now = Utc::now();
    let existing = repositories::scores::get_stock_score(&state.db, &upper_symbol)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(ref score) = existing {
        if now - score.time < Duration::hours(SCORE_STALE_HOURS) {
            return Ok(Json(Some(score.clone())));
        }
    }

    // Compute and persist a fresh score snapshot if missing or stale
    let inserted = compute_and_insert_score(&state.db, &upper_symbol)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(Some(inserted)))
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

    // Exclude non-Syariah bank stocks from signal/screener rankings.
    let excluded: HashSet<String> = repositories::stocks::get_all_stocks(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter()
        .filter(|s| is_excluded_non_syariah_bank(s))
        .map(|s| s.symbol)
        .collect();

    let fetch_limit = limit + excluded.len() as i32;
    let scores = repositories::scores::get_latest_scores(&state.db, fetch_limit)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let filtered: Vec<StockScoreRow> = scores
        .into_iter()
        .filter(|s| !excluded.contains(&s.symbol))
        .take(limit as usize)
        .collect();

    Ok(Json(filtered))
}

#[derive(Debug, Serialize)]
pub struct RecomputeScoresResponse {
    pub computed: usize,
    pub skipped: usize,
    pub errors: usize,
}

async fn recompute_scores(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<RecomputeScoresResponse>, (axum::http::StatusCode, String)> {
    let stocks = repositories::stocks::get_all_stocks(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let pool = state.db.clone();
    let now = Utc::now();

    let results = futures_util::stream::iter(stocks.into_iter().map(|stock| {
        let pool = pool.clone();
        async move {
            let existing = repositories::scores::get_stock_score(&pool, &stock.symbol).await?;
            if let Some(score) = existing {
                if now - score.time < Duration::hours(SCORE_STALE_HOURS) {
                    return Ok::<_, sqlx::Error>(None);
                }
            }

            let inserted = compute_and_insert_score(&pool, &stock.symbol).await?;
            Ok::<_, sqlx::Error>(Some(inserted))
        }
    }))
    .buffer_unordered(8)
    .collect::<Vec<Result<Option<StockScoreRow>, sqlx::Error>>>()
    .await;

    let mut computed = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    for r in results {
        match r {
            Ok(Some(_)) => computed += 1,
            Ok(None) => skipped += 1,
            Err(_) => errors += 1,
        }
    }

    Ok(Json(RecomputeScoresResponse {
        computed,
        skipped,
        errors,
    }))
}

async fn compute_and_insert_score(
    pool: &sqlx::PgPool,
    symbol: &str,
) -> Result<StockScoreRow, sqlx::Error> {
    let now = Utc::now();

    // Prices: use a sufficiently long lookback to compute EMA50/RSI/MACD.
    let from = now - Duration::days(200);
    let prices = repositories::prices::get_price_history(pool, symbol, from, now).await?;

    let close_prices: Vec<Decimal> = prices.iter().map(|p| p.close).collect();
    let volumes: Vec<i64> = prices.iter().map(|p| p.volume).collect();
    let highs: Vec<Decimal> = prices.iter().map(|p| p.high).collect();
    let lows: Vec<Decimal> = prices.iter().map(|p| p.low).collect();

    let current_price = close_prices.last().copied().unwrap_or(Decimal::ZERO);

    let ema20 = calculate_ema20(&close_prices)
        .ok()
        .and_then(|v| v.last().copied());
    let ema50 = calculate_ema50(&close_prices)
        .ok()
        .and_then(|v| v.last().copied());

    let rsi = calculate_rsi14(&close_prices)
        .ok()
        .and_then(|v| v.last().copied());
    let macd_histogram = calculate_macd(&close_prices)
        .ok()
        .and_then(|m| m.histogram.last().copied());

    // Broker flow (last 5 days) used as a key technical input.
    let broker_from = now - Duration::days(5);
    let broker_to = now;
    let aggregates = repositories::broker_summary::get_broker_flow_aggregates(
        pool,
        symbol,
        broker_from,
        broker_to,
    )
    .await
    .unwrap_or_default();

    let mut total_net = 0.0f64;
    let mut total_traded = 0.0f64;
    let mut foreign_net = 0.0f64;
    let mut institutional_buying = false;

    for a in &aggregates {
        let buy_value = a.buy_value.to_f64().unwrap_or(0.0);
        let sell_value = a.sell_value.to_f64().unwrap_or(0.0);
        let net_value = a.net_value.to_f64().unwrap_or(0.0);

        total_traded += buy_value + sell_value;
        total_net += net_value;

        if a.category == "foreign_institutional" {
            foreign_net += net_value;
        }
        if a.category.contains("institutional") && net_value > 0.0 {
            institutional_buying = true;
        }
    }

    let broker_score = if total_traded <= 0.0 {
        None
    } else {
        let net_ratio = (total_net / total_traded).abs();
        let s = if net_ratio < 0.05 {
            50.0
        } else if total_net > 0.0 {
            80.0
        } else {
            20.0
        };
        Decimal::from_f64(s)
    };

    let (obi, ofi_trend) =
        compute_orderflow_inputs_from_ohlcv(&highs, &lows, &close_prices, &volumes);

    let technical_engine = TechnicalScoreEngine::new();
    let technical_input = TechnicalScoreInput {
        current_price,
        prices: close_prices,
        volumes,
        highs,
        lows,
        obi,
        ofi_trend,
        broker_score,
        institutional_buying,
        foreign_buying: foreign_net > 0.0,
        ema20,
        ema50,
        rsi,
        macd_histogram,
    };
    let technical_breakdown = technical_engine.calculate(&technical_input);

    let financials = repositories::stocks::get_financials(pool, symbol).await?;
    let fundamental_engine = FundamentalScoreEngine::new();
    let fundamental_input = if let Some(f) = financials {
        FundamentalInput {
            pe_ratio: f.pe_ratio,
            sector_pe: None,
            pb_ratio: f.pb_ratio,
            sector_pb: None,
            ev_ebitda: f.ev_ebitda,
            sector_ev_ebitda: None,
            dcf_margin: None,
            roe: f.roe.map(|v| v * dec!(100)),
            roa: f.roa.map(|v| v * dec!(100)),
            profit_margin: None,
            debt_to_equity: None,
            current_ratio: None,
        }
    } else {
        FundamentalInput::default()
    };
    let fundamental_breakdown = fundamental_engine.calculate(&fundamental_input);

    // Default neutral components until sentiment/ML pipelines are wired.
    let sentiment_score = 50.0;
    let ml_score = 50.0;

    let weights = ScoreWeights::default();
    let technical_f64 = technical_breakdown.total_score.to_f64().unwrap_or(50.0);
    let fundamental_f64 = fundamental_breakdown.total_score.to_f64().unwrap_or(50.0);
    let composite_f64 = calculate_composite_score(
        technical_f64,
        fundamental_f64,
        sentiment_score,
        ml_score,
        &weights,
    );

    let insert = repositories::scores::InsertStockScore {
        time: now,
        symbol: symbol.to_string(),
        composite_score: Decimal::from_f64(composite_f64).unwrap_or(dec!(50)),
        technical_score: technical_breakdown.total_score,
        fundamental_score: fundamental_breakdown.total_score,
        sentiment_score: Decimal::from_f64(sentiment_score).unwrap_or(dec!(50)),
        ml_score: Decimal::from_f64(ml_score).unwrap_or(dec!(50)),
        technical_breakdown: serde_json::to_value(&technical_breakdown).ok(),
        fundamental_breakdown: serde_json::to_value(&fundamental_breakdown).ok(),
        sentiment_breakdown: None,
        ml_breakdown: None,
    };

    repositories::scores::insert_stock_score(pool, &insert).await
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

#[derive(Debug, Serialize)]
pub struct StockFreshnessResponse {
    pub symbol: String,
    pub prices_as_of: Option<chrono::DateTime<chrono::Utc>>,
    pub broker_flow_as_of: Option<chrono::DateTime<chrono::Utc>>,
    pub financials_as_of: Option<chrono::DateTime<chrono::Utc>>,
    pub scores_as_of: Option<chrono::DateTime<chrono::Utc>>,
}

async fn get_stock_fundamentals(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<Json<Option<FundamentalData>>, (axum::http::StatusCode, String)> {
    let upper_symbol = symbol.to_uppercase();

    // Verify stock exists first
    let _stock = repositories::stocks::get_stock_by_symbol(&state.db, &upper_symbol)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                "Stock not found".to_string(),
            )
        })?;

    // Get latest financials from database
    let financials = repositories::stocks::get_financials(&state.db, &upper_symbol)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = financials.map(|f| {
        use rust_decimal::prelude::ToPrimitive;
        FundamentalData {
            symbol: f.symbol,
            pe_ratio: f.pe_ratio.and_then(|v| v.to_f64()),
            pb_ratio: f.pb_ratio.and_then(|v| v.to_f64()),
            ps_ratio: None,
            ev_ebitda: f.ev_ebitda.and_then(|v| v.to_f64()),
            // Convert ROE/ROA from decimal (0.21) to percentage (21.0)
            roe: f.roe.and_then(|v| v.to_f64().map(|x| x * 100.0)),
            roa: f.roa.and_then(|v| v.to_f64().map(|x| x * 100.0)),
            profit_margin: None,
            debt_to_equity: None,
            current_ratio: None,
            dcf_intrinsic_value: None,
            dcf_margin_of_safety: None,
            sector_avg_pe: None,
            sector_avg_pb: None,
        }
    });

    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::prelude::FromPrimitive;

    #[test]
    fn test_compute_orderflow_inputs_from_ohlcv_happy_path() {
        let highs = vec![dec!(110), dec!(110), dec!(110), dec!(110)];
        let lows = vec![dec!(100), dec!(100), dec!(100), dec!(100)];
        let closes = vec![dec!(101), dec!(103), dec!(106), dec!(109)];
        let volumes = vec![1000, 1000, 1000, 1000];

        let (obi, ofi_trend) =
            compute_orderflow_inputs_from_ohlcv(&highs, &lows, &closes, &volumes);

        assert!(obi.is_some());
        assert!(obi.unwrap() > Decimal::ZERO);

        assert!(ofi_trend.is_some());
        assert!(ofi_trend.unwrap() > Decimal::ZERO);
    }

    #[test]
    fn test_compute_orderflow_inputs_from_ohlcv_insufficient_data() {
        let highs = vec![dec!(110)];
        let lows = vec![dec!(100)];
        let closes = vec![dec!(105)];
        let volumes = vec![1000];

        let (obi, ofi_trend) =
            compute_orderflow_inputs_from_ohlcv(&highs, &lows, &closes, &volumes);

        assert!(obi.is_none());
        assert!(ofi_trend.is_none());
    }

    #[test]
    fn test_compute_orderflow_inputs_from_ohlcv_mismatched_lengths_uses_min_len() {
        let highs = vec![dec!(110), dec!(110), dec!(110)];
        let lows = vec![dec!(100), dec!(100), dec!(100)];
        let closes = vec![dec!(101), dec!(109), dec!(105)];
        let volumes = vec![1000, 1000];

        let (obi, ofi_trend) =
            compute_orderflow_inputs_from_ohlcv(&highs, &lows, &closes, &volumes);

        let expected_obi = calculate_ohlc_imbalance_proxy(dec!(110), dec!(100), dec!(109), 1000);
        let expected_trend = calculate_trend_normalized(&vec![
            calculate_ohlc_imbalance_proxy(dec!(110), dec!(100), dec!(101), 1000),
            calculate_ohlc_imbalance_proxy(dec!(110), dec!(100), dec!(109), 1000),
        ]);

        assert_eq!(obi, Some(expected_obi));
        assert_eq!(ofi_trend, Some(expected_trend));
    }

    #[test]
    fn test_compute_orderflow_inputs_from_ohlcv_trend_window() {
        let highs = vec![dec!(110); 25];
        let lows = vec![dec!(100); 25];
        let mut closes = vec![dec!(101); 25];
        for i in 5..25 {
            closes[i] = dec!(100) + Decimal::from_i32((i - 5) as i32).unwrap();
        }
        let volumes = vec![1000; 25];

        let (obi, ofi_trend) =
            compute_orderflow_inputs_from_ohlcv(&highs, &lows, &closes, &volumes);

        let expected_obi = calculate_ohlc_imbalance_proxy(dec!(110), dec!(100), closes[24], 1000);
        let mut expected_series = Vec::new();
        for idx in 5..25 {
            expected_series.push(calculate_ohlc_imbalance_proxy(
                dec!(110),
                dec!(100),
                closes[idx],
                1000,
            ));
        }
        let expected_trend = calculate_trend_normalized(&expected_series);

        assert_eq!(obi, Some(expected_obi));
        assert_eq!(ofi_trend, Some(expected_trend));
    }
}

async fn get_stock_freshness(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<Json<StockFreshnessResponse>, (axum::http::StatusCode, String)> {
    let upper_symbol = symbol.to_uppercase();

    // Verify stock exists
    repositories::stocks::get_stock_by_symbol(&state.db, &upper_symbol)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Stock not found: {}", upper_symbol),
            )
        })?;

    let prices_as_of = repositories::prices::get_latest_price(&state.db, &upper_symbol)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(|p| p.time);

    let broker_flow_as_of =
        repositories::broker_summary::get_latest_broker_summary_time(&state.db, &upper_symbol)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let financials_as_of =
        repositories::stocks::get_latest_financials_created_at(&state.db, &upper_symbol)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let scores_as_of = repositories::scores::get_stock_score(&state.db, &upper_symbol)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(|s| s.time);

    Ok(Json(StockFreshnessResponse {
        symbol: upper_symbol,
        prices_as_of,
        broker_flow_as_of,
        financials_as_of,
        scores_as_of,
    }))
}
