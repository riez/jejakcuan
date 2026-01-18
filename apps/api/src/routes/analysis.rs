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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TradingSignal {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

#[derive(Debug, Serialize)]
pub struct SignalAnalysis {
    pub signal: TradingSignal,
    pub conviction_percent: f64,
    pub thesis: String,
    pub target_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub upside_percent: Option<f64>,
    pub downside_percent: Option<f64>,
    pub risk_reward_ratio: Option<f64>,
    pub key_catalysts: Vec<String>,
    pub key_risks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SuspiciousActivity {
    pub detected: bool,
    pub activity_type: String,
    pub description: String,
    pub severity: String,
    pub brokers_involved: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BrokerInfo {
    pub code: String,
    pub name: Option<String>,
    pub avg_price: f64,
    pub category: String,
    pub buy_volume: i64,
    pub sell_volume: i64,
    pub net_volume: i64,
    pub buy_value: f64,
    pub sell_value: f64,
    pub net_value: f64,
}

#[derive(Debug, Serialize)]
pub struct BrokerSummaryResponse {
    pub big_buyers: Vec<BrokerInfo>,
    pub big_sellers: Vec<BrokerInfo>,
    pub net_status: String, // "accumulation", "distribution", "balanced"
    pub price_range: PriceRange,
    pub foreign_net: f64,
    pub domestic_net: f64,
    // Institutional flow analysis (big player movements)
    pub institutional_analysis: Option<InstitutionalFlowAnalysis>,
}

#[derive(Debug, Serialize)]
pub struct InstitutionalFlowAnalysis {
    pub accumulation_score: f64,                // 0-100 score
    pub is_accumulating: bool,                  // Strong accumulation signal
    pub coordinated_buying: bool,               // Multiple institutional buyers acting together
    pub days_accumulated: i32,                  // Consecutive days of net buying
    pub net_5_day: f64,                         // 5-day rolling net flow
    pub net_20_day: f64,                        // 20-day rolling net flow
    pub institutional_net_5_day: f64,           // 5-day institutional net
    pub institutional_net_20_day: f64,          // 20-day institutional net
    pub foreign_net_5_day: f64,                 // 5-day foreign net
    pub foreign_net_20_day: f64,                // 20-day foreign net
    pub top_accumulators: Vec<AccumulatorInfo>, // Top institutional accumulators
    pub signal_strength: String,
    pub signal_description: String,
    pub suspicious_activity: Option<SuspiciousActivity>,
}

#[derive(Debug, Serialize)]
pub struct AccumulatorInfo {
    pub broker_code: String,
    pub broker_name: Option<String>,
    pub category: String,
    pub net_value: f64,
    pub net_volume: i64,
    pub is_foreign: bool,
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
    let technical = get_technical_analysis(&state, &upper_symbol, days)
        .await
        .ok();

    // Get broker flow
    let broker_summary = get_broker_flow_internal(&state, &upper_symbol, 5)
        .await
        .ok();

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

    get_technical_analysis(&state, &upper_symbol, days)
        .await
        .map(Json)
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

    get_broker_flow_internal(&state, &upper_symbol, days)
        .await
        .map(Json)
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
    let rsi = rsi_values.last().copied().unwrap_or(dec!(50));
    let rsi_f64 = rsi.to_f64().unwrap_or(50.0);
    let rsi_sig = rsi_signal(rsi).to_string();

    // Calculate MACD
    let macd_result = calculate_macd(&close_prices).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("MACD calculation error: {}", e),
        )
    })?;
    let macd_value = macd_result
        .macd_line
        .last()
        .copied()
        .unwrap_or(Decimal::ZERO);
    let macd_hist = macd_result
        .histogram
        .last()
        .copied()
        .unwrap_or(Decimal::ZERO);
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
            upper: bollinger
                .upper
                .last()
                .copied()
                .unwrap_or(Decimal::ZERO)
                .to_f64()
                .unwrap_or(0.0),
            middle: bollinger
                .middle
                .last()
                .copied()
                .unwrap_or(Decimal::ZERO)
                .to_f64()
                .unwrap_or(0.0),
            lower: bollinger
                .lower
                .last()
                .copied()
                .unwrap_or(Decimal::ZERO)
                .to_f64()
                .unwrap_or(0.0),
        },
        ichimoku,
        support,
        resistance,
        summary,
    })
}

async fn get_broker_flow_internal(
    state: &AppState,
    symbol: &str,
    days: i32,
) -> Result<BrokerSummaryResponse, (axum::http::StatusCode, String)> {
    let from = Utc::now() - Duration::days(days as i64);
    let to = Utc::now();
    let from_20 = Utc::now() - Duration::days(20);

    let aggregates =
        repositories::broker_summary::get_broker_flow_aggregates(&state.db, symbol, from, to)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let price_range = repositories::broker_summary::get_price_range(&state.db, symbol, from, to)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let daily_summaries =
        repositories::broker_summary::get_daily_broker_summaries(&state.db, symbol, from_20, to)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut foreign_net = 0.0;
    let mut domestic_net = 0.0;
    let mut total_net = 0.0;
    let mut total_traded = 0.0;
    let mut total_volume: i64 = 0;

    for a in &aggregates {
        let buy_value = a.buy_value.to_f64().unwrap_or(0.0);
        let sell_value = a.sell_value.to_f64().unwrap_or(0.0);
        let net_value = a.net_value.to_f64().unwrap_or(0.0);

        total_traded += buy_value + sell_value;
        total_net += net_value;
        total_volume += a.buy_volume + a.sell_volume;

        if a.category == "foreign_institutional" {
            foreign_net += net_value;
        } else {
            domestic_net += net_value;
        }
    }

    let net_status = if total_traded <= 0.0 {
        "balanced"
    } else {
        let net_ratio = (total_net / total_traded).abs();
        if net_ratio < 0.05 {
            "balanced"
        } else if total_net > 0.0 {
            "accumulation"
        } else {
            "distribution"
        }
    };

    let big_buyers: Vec<BrokerInfo> = aggregates
        .iter()
        .filter(|a| a.net_value > Decimal::ZERO)
        .take(5)
        .map(|a| {
            let buy_value = a.buy_value.to_f64().unwrap_or(0.0);
            let sell_value = a.sell_value.to_f64().unwrap_or(0.0);
            let net_value = a.net_value.to_f64().unwrap_or(0.0);
            let avg_price = if a.buy_volume > 0 {
                buy_value / a.buy_volume as f64
            } else {
                0.0
            };

            BrokerInfo {
                code: a.broker_code.clone(),
                name: a.broker_name.clone(),
                avg_price,
                category: a.category.clone(),
                buy_volume: a.buy_volume,
                sell_volume: a.sell_volume,
                net_volume: a.net_volume,
                buy_value,
                sell_value,
                net_value,
            }
        })
        .collect();

    let big_sellers: Vec<BrokerInfo> = aggregates
        .iter()
        .rev()
        .filter(|a| a.net_value < Decimal::ZERO)
        .take(5)
        .map(|a| {
            let buy_value = a.buy_value.to_f64().unwrap_or(0.0);
            let sell_value = a.sell_value.to_f64().unwrap_or(0.0);
            let net_value = a.net_value.to_f64().unwrap_or(0.0);
            let avg_price = if a.sell_volume > 0 {
                sell_value / a.sell_volume as f64
            } else {
                0.0
            };

            BrokerInfo {
                code: a.broker_code.clone(),
                name: a.broker_name.clone(),
                avg_price,
                category: a.category.clone(),
                buy_volume: a.buy_volume,
                sell_volume: a.sell_volume,
                net_volume: a.net_volume,
                buy_value,
                sell_value,
                net_value,
            }
        })
        .collect();

    let avg_daily_volume = total_volume / days.max(1) as i64;
    let suspicious =
        detect_suspicious_activity(&big_buyers, &big_sellers, total_volume, avg_daily_volume);

    let mut institutional_analysis =
        calculate_institutional_flow_analysis(&aggregates, &daily_summaries);

    if let Some(ref mut inst) = institutional_analysis {
        inst.suspicious_activity = suspicious;
    }

    Ok(BrokerSummaryResponse {
        big_buyers,
        big_sellers,
        net_status: net_status.to_string(),
        price_range: PriceRange {
            low: price_range.low.and_then(|d| d.to_f64()).unwrap_or(0.0),
            high: price_range.high.and_then(|d| d.to_f64()).unwrap_or(0.0),
        },
        foreign_net,
        domestic_net,
        institutional_analysis,
    })
}

fn calculate_institutional_flow_analysis(
    aggregates: &[repositories::broker_summary::BrokerFlowAggregateRow],
    daily_summaries: &[repositories::broker_summary::DailyBrokerSummaryRow],
) -> Option<InstitutionalFlowAnalysis> {
    if daily_summaries.is_empty() {
        return None;
    }

    use std::collections::{HashMap, HashSet};

    let mut by_date: HashMap<
        chrono::NaiveDate,
        Vec<&repositories::broker_summary::DailyBrokerSummaryRow>,
    > = HashMap::new();
    for summary in daily_summaries {
        by_date
            .entry(summary.time.date_naive())
            .or_default()
            .push(summary);
    }

    let mut dates: Vec<_> = by_date.keys().cloned().collect();
    dates.sort();

    if dates.is_empty() {
        return None;
    }

    let calculate_window = |window_size: usize| -> (f64, f64, f64, i32, bool) {
        let window_dates: Vec<_> = dates.iter().rev().take(window_size).cloned().collect();
        let mut net_value = 0.0f64;
        let mut institutional_net = 0.0f64;
        let mut foreign_net = 0.0f64;
        let mut days_positive = 0i32;
        let mut institutional_buyers: HashMap<String, i32> = HashMap::new();

        for date in &window_dates {
            if let Some(day_summaries) = by_date.get(date) {
                let mut day_net = 0.0f64;
                let mut day_inst_net = 0.0f64;
                let mut day_foreign_net = 0.0f64;

                for summary in day_summaries {
                    let net = summary.net_value.to_f64().unwrap_or(0.0);
                    day_net += net;

                    let is_institutional = summary.category == "foreign_institutional"
                        || summary.category == "local_institutional";
                    let is_foreign = summary.category == "foreign_institutional";

                    if is_institutional {
                        let weight = if is_foreign { 1.0 } else { 0.8 };
                        day_inst_net += net * weight;

                        if net > 0.0 {
                            *institutional_buyers
                                .entry(summary.broker_code.clone())
                                .or_default() += 1;
                        }
                    }

                    if is_foreign {
                        day_foreign_net += net;
                    }
                }

                net_value += day_net;
                institutional_net += day_inst_net;
                foreign_net += day_foreign_net;

                if day_inst_net > 0.0 {
                    days_positive += 1;
                }
            }
        }

        let min_days = window_dates.len() / 2;
        let consistent_buyers: usize = institutional_buyers
            .values()
            .filter(|&&days| days as usize >= min_days.max(1))
            .count();
        let coordinated = consistent_buyers >= 3;

        (
            net_value,
            institutional_net,
            foreign_net,
            days_positive,
            coordinated,
        )
    };

    let (net_5, inst_5, foreign_5, days_5, coord_5) = calculate_window(5);
    let (net_20, inst_20, foreign_20, days_20, coord_20) = calculate_window(20);

    let mut score = 50.0f64;

    if inst_5 > 0.0 {
        score += 25.0;
    } else if inst_5 < 0.0 {
        score -= 15.0;
    }
    if foreign_5 > 0.0 {
        score += 15.0;
    } else if foreign_5 < 0.0 {
        score -= 10.0;
    }
    if days_5 > 0 {
        score += (days_5 as f64 / 5.0) * 15.0;
    }
    if coord_5 {
        score += 10.0;
    }

    score = score.max(0.0).min(100.0);

    let is_accumulating = score > 60.0 && days_5 >= 3;
    let is_distributing = score < 40.0 && days_5 <= 1;

    let signal_strength = if score >= 75.0 {
        "strong"
    } else if score >= 60.0 {
        "moderate"
    } else if score <= 25.0 {
        "distribution"
    } else if score <= 40.0 {
        "weak"
    } else {
        "neutral"
    };

    let signal_description = if is_accumulating && coord_5 {
        format!(
            "Strong accumulation: {} institutional buyers coordinating over {} days",
            aggregates
                .iter()
                .filter(|a| a.net_value > Decimal::ZERO
                    && (a.category == "foreign_institutional"
                        || a.category == "local_institutional"))
                .count(),
            days_5
        )
    } else if is_accumulating {
        format!(
            "Accumulation detected: institutional net buying for {} days",
            days_5
        )
    } else if is_distributing {
        "Distribution: institutional net selling detected".to_string()
    } else if coord_5 {
        "Coordinated activity: multiple institutional brokers acting together".to_string()
    } else {
        "Mixed signals: no clear accumulation/distribution pattern".to_string()
    };

    let top_accumulators: Vec<AccumulatorInfo> = aggregates
        .iter()
        .filter(|a| {
            a.net_value > Decimal::ZERO
                && (a.category == "foreign_institutional" || a.category == "local_institutional")
        })
        .take(5)
        .map(|a| AccumulatorInfo {
            broker_code: a.broker_code.clone(),
            broker_name: a.broker_name.clone(),
            category: a.category.clone(),
            net_value: a.net_value.to_f64().unwrap_or(0.0),
            net_volume: a.net_volume,
            is_foreign: a.category == "foreign_institutional",
        })
        .collect();

    Some(InstitutionalFlowAnalysis {
        accumulation_score: score,
        is_accumulating,
        coordinated_buying: coord_5 || coord_20,
        days_accumulated: days_5,
        net_5_day: net_5,
        net_20_day: net_20,
        institutional_net_5_day: inst_5,
        institutional_net_20_day: inst_20,
        foreign_net_5_day: foreign_5,
        foreign_net_20_day: foreign_20,
        top_accumulators,
        signal_strength: signal_strength.to_string(),
        signal_description,
        suspicious_activity: None, // Will be populated by detect_suspicious_activity
    })
}

fn calculate_support_resistance(prices: &[jejakcuan_db::StockPriceRow]) -> (Vec<f64>, Vec<f64>) {
    if prices.is_empty() {
        return (vec![], vec![]);
    }

    let lows: Vec<f64> = prices
        .iter()
        .map(|p| p.low.to_f64().unwrap_or(0.0))
        .collect();
    let highs: Vec<f64> = prices
        .iter()
        .map(|p| p.high.to_f64().unwrap_or(0.0))
        .collect();

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
        let is_duplicate = result
            .iter()
            .any(|&existing| (level - existing).abs() / existing.max(1.0) < tolerance);
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
            cloud_range: PriceRange {
                low: 0.0,
                high: 0.0,
            },
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
        strengths.push(format!(
            "Technical indicators favor buying ({} buy vs {} sell)",
            technical.summary.buy, technical.summary.sell
        ));
    } else if technical.summary.sell > technical.summary.buy {
        weaknesses.push(format!(
            "Technical indicators favor selling ({} sell vs {} buy)",
            technical.summary.sell, technical.summary.buy
        ));
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
            strengths.push(format!(
                "Potential {:.1}% upside to resistance at {:.0}",
                upside, nearest_resistance
            ));
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
        format!(
            "Consider entry near support {:.0}, target resistance {:.0}",
            technical
                .support
                .first()
                .copied()
                .unwrap_or(last_price * 0.95),
            technical
                .resistance
                .first()
                .copied()
                .unwrap_or(last_price * 1.1)
        )
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

#[allow(dead_code)]
fn calculate_trading_signal(
    composite_score: f64,
    technical: &TechnicalResponse,
    valuation: &ValuationResponse,
    broker: &BrokerSummaryResponse,
    current_price: f64,
) -> SignalAnalysis {
    let signal = match composite_score {
        c if c >= 75.0 => TradingSignal::StrongBuy,
        c if c >= 60.0 => TradingSignal::Buy,
        c if c >= 45.0 => TradingSignal::Hold,
        c if c >= 30.0 => TradingSignal::Sell,
        _ => TradingSignal::StrongSell,
    };

    let target_price = Some(valuation.fair_price_range.high);

    let stop_loss = technical
        .support
        .first()
        .copied()
        .unwrap_or(current_price * 0.95);

    let upside = target_price.map(|t| ((t - current_price) / current_price) * 100.0);
    let downside = Some(((current_price - stop_loss) / current_price) * 100.0);

    let risk_reward = match (upside, downside) {
        (Some(up), Some(down)) if down.abs() > 0.01 => Some(up / down.abs()),
        _ => None,
    };

    let thesis = generate_thesis(broker, technical, valuation);
    let key_catalysts = extract_catalysts(broker, technical);
    let key_risks = extract_risks(technical, valuation);

    SignalAnalysis {
        signal,
        conviction_percent: composite_score,
        thesis,
        target_price,
        stop_loss: Some(stop_loss),
        upside_percent: upside,
        downside_percent: downside,
        risk_reward_ratio: risk_reward,
        key_catalysts,
        key_risks,
    }
}

fn generate_thesis(
    broker: &BrokerSummaryResponse,
    technical: &TechnicalResponse,
    valuation: &ValuationResponse,
) -> String {
    let mut parts = Vec::new();

    if let Some(inst) = &broker.institutional_analysis {
        if inst.is_accumulating && inst.coordinated_buying {
            parts.push("Strong institutional accumulation detected".to_string());
        } else if inst.is_accumulating {
            parts.push("Institutional accumulation ongoing".to_string());
        }
    }

    if technical.rsi < 30.0 {
        parts.push("oversold on RSI".to_string());
    } else if technical.rsi > 70.0 {
        parts.push("overbought on RSI".to_string());
    }

    let margin_of_safety =
        ((valuation.fair_price_range.high - technical.last_price) / technical.last_price) * 100.0;
    if margin_of_safety > 20.0 {
        parts.push(format!("{:.0}% margin of safety", margin_of_safety));
    }

    if parts.is_empty() {
        "Mixed signals - wait for clearer setup".to_string()
    } else {
        parts.join(" with ")
    }
}

fn extract_catalysts(broker: &BrokerSummaryResponse, technical: &TechnicalResponse) -> Vec<String> {
    let mut catalysts = Vec::new();

    if let Some(inst) = &broker.institutional_analysis {
        if inst.coordinated_buying {
            catalysts.push("Multiple institutions accumulating".to_string());
        }
        if inst.foreign_net_5_day > 0.0 {
            catalysts.push("Foreign buying interest".to_string());
        }
    }

    if technical.macd_signal == "bullish" {
        catalysts.push("MACD bullish crossover".to_string());
    }

    catalysts
}

fn extract_risks(technical: &TechnicalResponse, valuation: &ValuationResponse) -> Vec<String> {
    let mut risks = Vec::new();

    if technical.rsi > 70.0 {
        risks.push("Overbought conditions".to_string());
    }

    let is_expensive = valuation.fair_price_range.low > technical.last_price * 0.9;
    if is_expensive {
        risks.push("Valuation stretched".to_string());
    }

    risks
}

fn detect_suspicious_activity(
    big_buyers: &[BrokerInfo],
    big_sellers: &[BrokerInfo],
    total_volume: i64,
    avg_daily_volume: i64,
) -> Option<SuspiciousActivity> {
    use std::collections::HashSet;

    let buy_codes: HashSet<_> = big_buyers.iter().map(|b| b.code.as_str()).collect();
    let sell_codes: HashSet<_> = big_sellers.iter().map(|b| b.code.as_str()).collect();

    let both_sides: Vec<&str> = buy_codes.intersection(&sell_codes).copied().collect();

    if !both_sides.is_empty() {
        return Some(SuspiciousActivity {
            detected: true,
            activity_type: "wash_trading_signal".to_string(),
            description: format!(
                "Broker(s) {} appear on both buy and sell sides - possible wash trading",
                both_sides.join(", ")
            ),
            severity: "medium".to_string(),
            brokers_involved: both_sides.iter().map(|s| s.to_string()).collect(),
        });
    }

    if avg_daily_volume > 0 && total_volume > avg_daily_volume * 3 {
        return Some(SuspiciousActivity {
            detected: true,
            activity_type: "unusual_volume".to_string(),
            description: format!(
                "Volume {}x above average - unusual activity",
                total_volume / avg_daily_volume.max(1)
            ),
            severity: "low".to_string(),
            brokers_involved: vec![],
        });
    }

    None
}
