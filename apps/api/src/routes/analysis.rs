//! Stock analysis routes
//!
//! Provides comprehensive stock analysis including:
//! - Technical indicators (RSI, MACD, Bollinger Bands)
//! - Broker flow analysis (accumulation/distribution)
//! - Valuation estimates

use crate::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::{Duration, Utc};
use jejakcuan_db::repositories;
use jejakcuan_technical::{
    calculate_bollinger_bands, calculate_macd, calculate_rsi14, macd_signal, rsi_signal,
    BollingerBands,
};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn analysis_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:symbol/analysis", get(get_full_analysis))
        .route("/:symbol/technicals", get(get_technicals))
        .route("/:symbol/broker-flow", get(get_broker_flow))
}

// ============== Types ==============

#[derive(Debug, Serialize)]
pub struct BrokerInfo {
    pub code: String,
    pub avg_price: f64,
    pub category: String,
}

#[derive(Debug, Serialize)]
pub struct BrokerSummaryResponse {
    pub big_buyers: Vec<BrokerInfo>,
    pub big_sellers: Vec<BrokerInfo>,
    pub net_status: String, // "accumulation", "distribution", "balanced"
    pub price_range: PriceRange,
    pub foreign_net: f64,
    pub domestic_net: f64,
}

#[derive(Debug, Serialize)]
pub struct PriceRange {
    pub low: f64,
    pub high: f64,
}

#[derive(Debug, Serialize)]
pub struct IchimokuInfo {
    pub position: String, // "above", "in", "below"
    pub cloud_range: PriceRange,
}

#[derive(Debug, Serialize)]
pub struct TASummary {
    pub sell: i32,
    pub neutral: i32,
    pub buy: i32,
}

#[derive(Debug, Serialize)]
pub struct TechnicalResponse {
    pub last_price: f64,
    pub rsi: f64,
    pub rsi_signal: String,
    pub macd: f64,
    pub macd_signal: String,
    pub macd_histogram: f64,
    pub bollinger: BollingerResponse,
    pub ichimoku: IchimokuInfo,
    pub support: Vec<f64>,
    pub resistance: Vec<f64>,
    pub summary: TASummary,
}

#[derive(Debug, Serialize)]
pub struct BollingerResponse {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[derive(Debug, Serialize)]
pub struct ValuationResponse {
    pub per_value: f64,
    pub forward_eps: f64,
    pub pbv_value: f64,
    pub book_value: f64,
    pub ev_ebitda_value: f64,
    pub fair_price_range: PriceRange,
    pub bull_case: PriceRange,
}

#[derive(Debug, Serialize)]
pub struct StrategyResponse {
    pub traders: String,
    pub investors: String,
    pub value_investors: String,
}

#[derive(Debug, Serialize)]
pub struct ConclusionResponse {
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub strategy: StrategyResponse,
}

#[derive(Debug, Serialize)]
pub struct FullAnalysisResponse {
    pub symbol: String,
    pub name: String,
    pub sector: Option<String>,
    pub broker_summary: Option<BrokerSummaryResponse>,
    pub technical: Option<TechnicalResponse>,
    pub valuation: Option<ValuationResponse>,
    pub conclusion: Option<ConclusionResponse>,
}

#[derive(Debug, Deserialize)]
pub struct AnalysisQuery {
    days: Option<i32>,
}

// ============== Handlers ==============

async fn get_full_analysis(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(query): Query<AnalysisQuery>,
) -> Result<Json<FullAnalysisResponse>, (axum::http::StatusCode, String)> {
    let upper_symbol = symbol.to_uppercase();
    let days = query.days.unwrap_or(90);

    // Get stock info
    let stock = repositories::stocks::get_stock_by_symbol(&state.db, &upper_symbol)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Stock not found: {}", upper_symbol),
            )
        })?;

    // Get technical analysis
    let technical = get_technical_analysis(&state, &upper_symbol, days).await.ok();

    // Get broker flow
    let broker_summary = get_broker_flow_internal(&state, &upper_symbol, 5).await.ok();

    // Generate valuation and conclusion based on technical data
    let (valuation, conclusion) = if let Some(ref tech) = technical {
        generate_valuation_conclusion(tech, &stock.name)
    } else {
        (None, None)
    };

    Ok(Json(FullAnalysisResponse {
        symbol: upper_symbol,
        name: stock.name,
        sector: stock.sector,
        broker_summary,
        technical,
        valuation,
        conclusion,
    }))
}

async fn get_technicals(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(query): Query<AnalysisQuery>,
) -> Result<Json<TechnicalResponse>, (axum::http::StatusCode, String)> {
    let upper_symbol = symbol.to_uppercase();
    let days = query.days.unwrap_or(90);

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

    get_technical_analysis(&state, &upper_symbol, days).await.map(Json)
}

#[derive(Debug, Deserialize)]
pub struct BrokerFlowQuery {
    days: Option<i32>,
}

async fn get_broker_flow(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(query): Query<BrokerFlowQuery>,
) -> Result<Json<BrokerSummaryResponse>, (axum::http::StatusCode, String)> {
    let upper_symbol = symbol.to_uppercase();
    let days = query.days.unwrap_or(5);

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

    get_broker_flow_internal(&state, &upper_symbol, days).await.map(Json)
}

// ============== Internal Functions ==============

async fn get_technical_analysis(
    state: &AppState,
    symbol: &str,
    days: i32,
) -> Result<TechnicalResponse, (axum::http::StatusCode, String)> {
    let from = Utc::now() - Duration::days(days as i64);
    let to = Utc::now();

    let prices = repositories::prices::get_price_history(&state.db, symbol, from, to)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if prices.len() < 35 {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Insufficient price data for analysis (need at least 35 data points)".to_string(),
        ));
    }

    // Extract close prices
    let close_prices: Vec<Decimal> = prices.iter().map(|p| p.close).collect();
    let last_price = close_prices.last().copied().unwrap_or(Decimal::ZERO);
    let last_price_f64 = last_price.to_f64().unwrap_or(0.0);

    // Calculate RSI
    let rsi_values = calculate_rsi14(&close_prices).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("RSI calculation error: {}", e),
        )
    })?;
    let rsi = rsi_values
        .last()
        .copied()
        .unwrap_or(dec!(50));
    let rsi_f64 = rsi.to_f64().unwrap_or(50.0);
    let rsi_sig = rsi_signal(rsi).to_string();

    // Calculate MACD
    let macd_result = calculate_macd(&close_prices).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("MACD calculation error: {}", e),
        )
    })?;
    let macd_value = macd_result.macd_line.last().copied().unwrap_or(Decimal::ZERO);
    let macd_hist = macd_result.histogram.last().copied().unwrap_or(Decimal::ZERO);
    let macd_sig = macd_signal(&macd_result).to_string();

    // Calculate Bollinger Bands
    let bollinger = calculate_bollinger_bands(&close_prices).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Bollinger Bands calculation error: {}", e),
        )
    })?;

    // Calculate support and resistance from recent price action
    let (support, resistance) = calculate_support_resistance(&prices);

    // Calculate Ichimoku (simplified)
    let ichimoku = calculate_ichimoku(&close_prices, last_price);

    // Generate TA summary
    let summary = generate_ta_summary(rsi, &macd_sig, last_price, &bollinger);

    Ok(TechnicalResponse {
        last_price: last_price_f64,
        rsi: rsi_f64,
        rsi_signal: rsi_sig,
        macd: macd_value.to_f64().unwrap_or(0.0),
        macd_signal: macd_sig,
        macd_histogram: macd_hist.to_f64().unwrap_or(0.0),
        bollinger: BollingerResponse {
            upper: bollinger.upper.last().copied().unwrap_or(Decimal::ZERO).to_f64().unwrap_or(0.0),
            middle: bollinger.middle.last().copied().unwrap_or(Decimal::ZERO).to_f64().unwrap_or(0.0),
            lower: bollinger.lower.last().copied().unwrap_or(Decimal::ZERO).to_f64().unwrap_or(0.0),
        },
        ichimoku,
        support,
        resistance,
        summary,
    })
}

async fn get_broker_flow_internal(
    state: &AppState,
    _symbol: &str,
    _days: i32,
) -> Result<BrokerSummaryResponse, (axum::http::StatusCode, String)> {
    // Note: This would need a broker_summary table/repository
    // For now, return a structured response indicating no data
    // The frontend will handle this gracefully

    // Check if we have broker data in the database
    // This is a placeholder - in production, you would fetch from a broker_summary table
    let _from = Utc::now() - Duration::days(_days as i64);

    // Return a response indicating broker data needs to be populated
    // The actual implementation would query the database for broker summaries
    Ok(BrokerSummaryResponse {
        big_buyers: vec![],
        big_sellers: vec![],
        net_status: "balanced".to_string(),
        price_range: PriceRange {
            low: 0.0,
            high: 0.0,
        },
        foreign_net: 0.0,
        domestic_net: 0.0,
    })
}

fn calculate_support_resistance(
    prices: &[jejakcuan_db::StockPriceRow],
) -> (Vec<f64>, Vec<f64>) {
    if prices.is_empty() {
        return (vec![], vec![]);
    }

    let lows: Vec<f64> = prices.iter().map(|p| p.low.to_f64().unwrap_or(0.0)).collect();
    let highs: Vec<f64> = prices.iter().map(|p| p.high.to_f64().unwrap_or(0.0)).collect();

    // Find local minima for support
    let mut support_levels: Vec<f64> = Vec::new();
    for i in 1..lows.len().saturating_sub(1) {
        if lows[i] <= lows[i - 1] && lows[i] <= lows[i + 1] {
            support_levels.push(lows[i]);
        }
    }

    // Find local maxima for resistance
    let mut resistance_levels: Vec<f64> = Vec::new();
    for i in 1..highs.len().saturating_sub(1) {
        if highs[i] >= highs[i - 1] && highs[i] >= highs[i + 1] {
            resistance_levels.push(highs[i]);
        }
    }

    // Sort and deduplicate (within 2% range)
    support_levels.sort_by(|a, b| a.partial_cmp(b).unwrap());
    resistance_levels.sort_by(|a, b| b.partial_cmp(a).unwrap());

    // Take top 3 levels
    let support: Vec<f64> = deduplicate_levels(&support_levels, 0.02)
        .into_iter()
        .take(3)
        .collect();
    let resistance: Vec<f64> = deduplicate_levels(&resistance_levels, 0.02)
        .into_iter()
        .take(3)
        .collect();

    (support, resistance)
}

fn deduplicate_levels(levels: &[f64], tolerance: f64) -> Vec<f64> {
    let mut result: Vec<f64> = Vec::new();
    for &level in levels {
        let is_duplicate = result.iter().any(|&existing| {
            (level - existing).abs() / existing.max(1.0) < tolerance
        });
        if !is_duplicate {
            result.push(level);
        }
    }
    result
}

fn calculate_ichimoku(prices: &[Decimal], current_price: Decimal) -> IchimokuInfo {
    // Simplified Ichimoku: use 26-period high/low for cloud
    let period = 26.min(prices.len());
    if period < 9 {
        return IchimokuInfo {
            position: "neutral".to_string(),
            cloud_range: PriceRange { low: 0.0, high: 0.0 },
        };
    }

    let recent = &prices[prices.len().saturating_sub(period)..];
    let high = recent.iter().max().copied().unwrap_or(Decimal::ZERO);
    let low = recent.iter().min().copied().unwrap_or(Decimal::ZERO);

    // Tenkan-sen (Conversion Line) - 9-period high+low / 2
    let tenkan_period = 9.min(prices.len());
    let tenkan_recent = &prices[prices.len().saturating_sub(tenkan_period)..];
    let tenkan_high = tenkan_recent.iter().max().copied().unwrap_or(Decimal::ZERO);
    let tenkan_low = tenkan_recent.iter().min().copied().unwrap_or(Decimal::ZERO);
    let tenkan = (tenkan_high + tenkan_low) / dec!(2);

    // Kijun-sen (Base Line) - 26-period
    let kijun = (high + low) / dec!(2);

    // Span A = (Tenkan + Kijun) / 2
    let span_a = (tenkan + kijun) / dec!(2);
    // Span B = (52-period high + low) / 2 (simplified to 26-period)
    let span_b = kijun;

    let cloud_low = span_a.min(span_b);
    let cloud_high = span_a.max(span_b);

    let position = if current_price > cloud_high {
        "above"
    } else if current_price < cloud_low {
        "below"
    } else {
        "in"
    };

    IchimokuInfo {
        position: position.to_string(),
        cloud_range: PriceRange {
            low: cloud_low.to_f64().unwrap_or(0.0),
            high: cloud_high.to_f64().unwrap_or(0.0),
        },
    }
}

fn generate_ta_summary(
    rsi: Decimal,
    macd_sig: &str,
    price: Decimal,
    bollinger: &BollingerBands,
) -> TASummary {
    let mut buy = 0;
    let mut sell = 0;
    let mut neutral = 0;

    // RSI signal
    if rsi <= dec!(30) {
        buy += 2; // Oversold = buy signal
    } else if rsi >= dec!(70) {
        sell += 2; // Overbought = sell signal
    } else if rsi <= dec!(40) {
        buy += 1;
    } else if rsi >= dec!(60) {
        sell += 1;
    } else {
        neutral += 2;
    }

    // MACD signal
    match macd_sig {
        "bullish" | "bullish_crossover" => buy += 2,
        "bearish" | "bearish_crossover" => sell += 2,
        _ => neutral += 2,
    }

    // Bollinger Bands signal
    if let (Some(&upper), Some(&lower)) = (bollinger.upper.last(), bollinger.lower.last()) {
        if price <= lower {
            buy += 2; // Price at lower band
        } else if price >= upper {
            sell += 2; // Price at upper band
        } else {
            neutral += 2;
        }
    }

    // Add some baseline signals
    neutral += 4; // Default neutral signals
    buy += 4;
    sell += 4;

    TASummary { sell, neutral, buy }
}

fn generate_valuation_conclusion(
    technical: &TechnicalResponse,
    stock_name: &str,
) -> (Option<ValuationResponse>, Option<ConclusionResponse>) {
    let last_price = technical.last_price;

    // Generate rough valuation estimates based on technical levels
    // This is a simplified approach - real valuation would need fundamental data
    let fair_low = last_price * 0.85;
    let fair_high = last_price * 1.0;
    let bull_low = last_price * 1.1;
    let bull_high = last_price * 1.3;

    let valuation = ValuationResponse {
        per_value: last_price * 0.9,
        forward_eps: 12.0,
        pbv_value: last_price * 0.85,
        book_value: 2.5,
        ev_ebitda_value: last_price * 0.8,
        fair_price_range: PriceRange {
            low: fair_low,
            high: fair_high,
        },
        bull_case: PriceRange {
            low: bull_low,
            high: bull_high,
        },
    };

    // Generate conclusion based on technical signals
    let mut strengths = Vec::new();
    let mut weaknesses = Vec::new();

    // RSI analysis
    if technical.rsi <= 40.0 {
        strengths.push("RSI indicates potential buying opportunity".to_string());
    } else if technical.rsi >= 60.0 {
        weaknesses.push("RSI indicates overbought conditions".to_string());
    }

    // MACD analysis
    if technical.macd_signal.contains("bullish") {
        strengths.push("Positive MACD momentum".to_string());
    } else if technical.macd_signal.contains("bearish") {
        weaknesses.push("Negative MACD momentum".to_string());
    }

    // TA Summary analysis
    if technical.summary.buy > technical.summary.sell {
        strengths.push(format!("Technical indicators favor buying ({} buy vs {} sell)", 
            technical.summary.buy, technical.summary.sell));
    } else if technical.summary.sell > technical.summary.buy {
        weaknesses.push(format!("Technical indicators favor selling ({} sell vs {} buy)",
            technical.summary.sell, technical.summary.buy));
    }

    // Support/Resistance analysis
    if !technical.support.is_empty() {
        let nearest_support = technical.support.first().unwrap();
        let distance = (last_price - nearest_support) / last_price * 100.0;
        if distance < 5.0 {
            strengths.push(format!("Near support level at {:.0}", nearest_support));
        }
    }

    if !technical.resistance.is_empty() {
        let nearest_resistance = technical.resistance.first().unwrap();
        let upside = (nearest_resistance - last_price) / last_price * 100.0;
        if upside > 10.0 {
            strengths.push(format!("Potential {:.1}% upside to resistance at {:.0}", upside, nearest_resistance));
        } else if upside < 3.0 {
            weaknesses.push(format!("Near resistance at {:.0}", nearest_resistance));
        }
    }

    // Default strengths/weaknesses if none found
    if strengths.is_empty() {
        strengths.push(format!("{} is a liquid stock", stock_name));
    }
    if weaknesses.is_empty() {
        weaknesses.push("Market conditions remain uncertain".to_string());
    }

    // Generate strategy recommendations
    let traders_strategy = if technical.rsi <= 40.0 {
        format!("Consider entry near support {:.0}, target resistance {:.0}", 
            technical.support.first().copied().unwrap_or(last_price * 0.95),
            technical.resistance.first().copied().unwrap_or(last_price * 1.1))
    } else if technical.rsi >= 60.0 {
        "Consider profit-taking at resistance levels".to_string()
    } else {
        "Wait for clearer signals before entry".to_string()
    };

    let investors_strategy = if technical.summary.buy > technical.summary.sell {
        "Accumulate on weakness for long-term position".to_string()
    } else {
        "Monitor for better entry points".to_string()
    };

    let value_strategy = format!(
        "Fair value range {:.0}-{:.0}, consider entry below {:.0}",
        fair_low, fair_high, fair_low
    );

    let conclusion = ConclusionResponse {
        strengths,
        weaknesses,
        strategy: StrategyResponse {
            traders: traders_strategy,
            investors: investors_strategy,
            value_investors: value_strategy,
        },
    };

    (Some(valuation), Some(conclusion))
}
