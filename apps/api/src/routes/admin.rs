//! Admin routes for data monitoring and system status
//!
//! Provides granular data source management with individual control
//! over each data provider within categories.

use crate::auth::AuthUser;
use crate::routes::jobs::Job;
use crate::AppState;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Legacy endpoints (backward compatible)
        .route("/data-status", get(get_data_status))
        .route("/data-status/:source_id", get(get_source_status))
        .route("/data-status/:source_id/refresh", post(refresh_source))
        // New granular data source endpoints
        .route("/data-sources", get(list_data_sources))
        .route("/data-sources/:source_id", get(get_data_source))
        .route(
            "/data-sources/:source_id/trigger",
            post(trigger_data_source),
        )
        .route(
            "/data-sources/category/:category/trigger",
            post(trigger_category),
        )
        .route("/data-sources/:source_id/config", get(get_source_config))
        // Job management endpoints
        .route("/jobs", get(list_jobs))
        .route("/jobs/:job_id", get(get_job))
        .route("/jobs/source/:source_id", get(get_source_jobs))
}

// ============================================================================
// Data Source Registry - Granular Source Definitions
// ============================================================================

/// Category of data source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSourceCategory {
    Broker,
    Prices,
    Fundamentals,
    Scores,
}

impl DataSourceCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Broker => "broker",
            Self::Prices => "prices",
            Self::Fundamentals => "fundamentals",
            Self::Scores => "scores",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Broker => "Broker Summary",
            Self::Prices => "Price Data",
            Self::Fundamentals => "Fundamentals",
            Self::Scores => "Computed Scores",
        }
    }
}

impl std::str::FromStr for DataSourceCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "broker" => Ok(Self::Broker),
            "prices" => Ok(Self::Prices),
            "fundamentals" => Ok(Self::Fundamentals),
            "scores" => Ok(Self::Scores),
            _ => Err(format!("Unknown category: {}", s)),
        }
    }
}

/// Type of data source implementation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    PythonScraper,
    RustClient,
    Computed,
}

/// Configuration field requirement
#[derive(Debug, Clone, Serialize)]
pub struct ConfigField {
    pub name: String,
    pub description: String,
    pub env_var: String,
    pub required: bool,
    pub is_secret: bool,
}

/// Definition of a data source
#[derive(Debug, Clone)]
pub struct DataSourceDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub category: DataSourceCategory,
    pub source_type: SourceType,
    pub description: &'static str,
    pub config_fields: Vec<ConfigField>,
    pub trigger_command: Option<&'static str>,
    pub db_table: Option<&'static str>,
    pub freshness_threshold_hours: i64,
}

/// Get the registry of all available data sources
fn get_data_source_registry() -> Vec<DataSourceDefinition> {
    vec![
        // =========================
        // BROKER CATEGORY
        // =========================
        DataSourceDefinition {
            id: "stockbit_broker",
            name: "Stockbit Broker Flow",
            category: DataSourceCategory::Broker,
            source_type: SourceType::PythonScraper,
            description: "Broker buy/sell activity from Stockbit API",
            config_fields: vec![],
            trigger_command: Some("python -m jejakcuan_ml.scrapers.cli broker --days 30"),
            db_table: Some("broker_summary"),
            freshness_threshold_hours: 24,
        },
        DataSourceDefinition {
            id: "indopremier_broker",
            name: "Indopremier Broker Summary",
            category: DataSourceCategory::Broker,
            source_type: SourceType::PythonScraper,
            description: "Broker summary from Indopremier HTML scraping",
            config_fields: vec![],
            trigger_command: Some("python -m jejakcuan_ml.scrapers.cli broker --days 30"),
            db_table: Some("broker_summary"),
            freshness_threshold_hours: 24,
        },
        DataSourceDefinition {
            id: "idx_broker",
            name: "IDX Broker Data",
            category: DataSourceCategory::Broker,
            source_type: SourceType::PythonScraper,
            description: "Official broker data from Indonesia Stock Exchange",
            config_fields: vec![],
            trigger_command: Some("python -m jejakcuan_ml.scrapers.cli broker --days 30"),
            db_table: Some("broker_summary"),
            freshness_threshold_hours: 24,
        },
        // =========================
        // PRICES CATEGORY
        // =========================
        DataSourceDefinition {
            id: "yahoo_finance",
            name: "Yahoo Finance",
            category: DataSourceCategory::Prices,
            source_type: SourceType::PythonScraper,
            description: "Historical OHLCV data via yfinance library",
            config_fields: vec![],
            trigger_command: Some("python -m jejakcuan_ml.scrapers.cli price --days 60"),
            db_table: Some("stock_prices"),
            freshness_threshold_hours: 24,
        },
        DataSourceDefinition {
            id: "twelvedata",
            name: "TwelveData API",
            category: DataSourceCategory::Prices,
            source_type: SourceType::RustClient,
            description: "Real-time quotes and time series from TwelveData",
            config_fields: vec![ConfigField {
                name: "API Key".to_string(),
                description: "TwelveData API key for authentication".to_string(),
                env_var: "TWELVEDATA_API_KEY".to_string(),
                required: true,
                is_secret: true,
            }],
            trigger_command: None, // Triggered via Rust client
            db_table: Some("stock_prices"),
            freshness_threshold_hours: 24,
        },
        // =========================
        // FUNDAMENTALS CATEGORY
        // =========================
        DataSourceDefinition {
            id: "yfinance_fundamentals",
            name: "Yahoo Finance Fundamentals",
            category: DataSourceCategory::Fundamentals,
            source_type: SourceType::PythonScraper,
            description: "Financial ratios and statistics from Yahoo Finance",
            config_fields: vec![],
            trigger_command: Some("python -m jejakcuan_ml.scrapers.cli idx"),
            db_table: Some("financials"),
            freshness_threshold_hours: 168, // 7 days
        },
        DataSourceDefinition {
            id: "sectors_app",
            name: "Sectors.app",
            category: DataSourceCategory::Fundamentals,
            source_type: SourceType::RustClient,
            description: "Indonesian market financials from Sectors.app API",
            config_fields: vec![ConfigField {
                name: "API Key".to_string(),
                description: "Sectors.app API key".to_string(),
                env_var: "SECTORS_API_KEY".to_string(),
                required: true,
                is_secret: true,
            }],
            trigger_command: None, // Triggered via Rust client
            db_table: Some("financials"),
            freshness_threshold_hours: 168, // 7 days
        },
        DataSourceDefinition {
            id: "idx_fundamentals",
            name: "IDX Financial Reports",
            category: DataSourceCategory::Fundamentals,
            source_type: SourceType::PythonScraper,
            description: "Official financial statements from IDX",
            config_fields: vec![],
            trigger_command: Some("python -m jejakcuan_ml.scrapers.cli idx"),
            db_table: Some("financials"),
            freshness_threshold_hours: 168,
        },
        // =========================
        // SCORES CATEGORY
        // =========================
        DataSourceDefinition {
            id: "technical_score",
            name: "Technical Score",
            category: DataSourceCategory::Scores,
            source_type: SourceType::Computed,
            description: "Technical analysis score from price indicators (RSI, MACD, etc.)",
            config_fields: vec![],
            trigger_command: None, // Computed via API
            db_table: Some("stock_scores"),
            freshness_threshold_hours: 24,
        },
        DataSourceDefinition {
            id: "fundamental_score",
            name: "Fundamental Score",
            category: DataSourceCategory::Scores,
            source_type: SourceType::Computed,
            description: "Fundamental analysis score from financial ratios",
            config_fields: vec![],
            trigger_command: None,
            db_table: Some("stock_scores"),
            freshness_threshold_hours: 24,
        },
        DataSourceDefinition {
            id: "sentiment_score",
            name: "Sentiment Score",
            category: DataSourceCategory::Scores,
            source_type: SourceType::Computed,
            description: "Market sentiment from broker flow analysis",
            config_fields: vec![],
            trigger_command: None,
            db_table: Some("stock_scores"),
            freshness_threshold_hours: 24,
        },
        DataSourceDefinition {
            id: "ml_score",
            name: "ML Prediction Score",
            category: DataSourceCategory::Scores,
            source_type: SourceType::PythonScraper,
            description: "Machine learning price prediction confidence",
            config_fields: vec![],
            trigger_command: Some("python -m jejakcuan_ml.prediction.score"),
            db_table: Some("stock_scores"),
            freshness_threshold_hours: 24,
        },
    ]
}

// ============================================================================
// API Response Types
// ============================================================================

/// Status of a granular data source
#[derive(Debug, Clone, Serialize)]
pub struct GranularDataSource {
    pub id: String,
    pub name: String,
    pub category: DataSourceCategory,
    pub category_name: String,
    pub source_type: SourceType,
    pub description: String,
    pub status: DataSourceState,
    pub config_status: ConfigStatus,
    pub last_update: Option<DateTime<Utc>>,
    pub record_count: i64,
    pub freshness_hours: Option<i64>,
    pub can_trigger: bool,
    pub trigger_command: Option<String>,
}

/// State of a data source
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSourceState {
    Fresh,
    Stale,
    Outdated,
    NoData,
    NotConfigured,
    Running,
    Error,
}

/// Configuration status
#[derive(Debug, Clone, Serialize)]
pub struct ConfigStatus {
    pub is_configured: bool,
    pub missing_fields: Vec<String>,
    pub config_fields: Vec<ConfigFieldStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigFieldStatus {
    pub name: String,
    pub description: String,
    pub env_var: String,
    pub required: bool,
    pub is_set: bool,
}

/// Response for listing all data sources
#[derive(Debug, Serialize)]
pub struct DataSourcesResponse {
    pub timestamp: DateTime<Utc>,
    pub overall_status: String,
    pub sources: Vec<GranularDataSource>,
    pub by_category: HashMap<String, Vec<GranularDataSource>>,
    pub summary: DataSourcesSummary,
}

#[derive(Debug, Serialize)]
pub struct DataSourcesSummary {
    pub total_sources: usize,
    pub configured_sources: usize,
    pub fresh_sources: usize,
    pub stale_sources: usize,
    pub categories: Vec<CategorySummary>,
}

#[derive(Debug, Serialize)]
pub struct CategorySummary {
    pub category: String,
    pub display_name: String,
    pub total: usize,
    pub fresh: usize,
    pub stale: usize,
    pub not_configured: usize,
}

/// Trigger response
#[derive(Debug, Serialize)]
pub struct TriggerResponse {
    pub source_id: String,
    pub status: String,
    pub message: String,
    pub command: Option<String>,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<Job>,
}

/// Category trigger response
#[derive(Debug, Serialize)]
pub struct CategoryTriggerResponse {
    pub category: String,
    pub triggered: Vec<TriggerResponse>,
    pub skipped: Vec<SkippedSource>,
}

#[derive(Debug, Serialize)]
pub struct SkippedSource {
    pub source_id: String,
    pub reason: String,
}

/// Config response
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub source_id: String,
    pub source_name: String,
    pub fields: Vec<ConfigFieldStatus>,
    pub is_configured: bool,
}

// ============================================================================
// Legacy Types (backward compatible)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct DataSourceStatus {
    pub id: String,
    pub name: String,
    pub source_type: String,
    pub last_update: Option<DateTime<Utc>>,
    pub record_count: i64,
    pub status: String,
    pub freshness_hours: Option<i64>,
    pub can_refresh: bool,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub source_id: String,
    pub status: String,
    pub message: String,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DataStatusResponse {
    pub timestamp: DateTime<Utc>,
    pub overall_status: String,
    pub sources: Vec<DataSourceStatus>,
    pub summary: DataSummary,
}

#[derive(Debug, Serialize)]
pub struct DataSummary {
    pub total_stocks: i64,
    pub stocks_with_prices: i64,
    pub stocks_with_scores: i64,
    pub stocks_with_broker_data: i64,
    pub oldest_price_data: Option<DateTime<Utc>>,
    pub newest_price_data: Option<DateTime<Utc>>,
}

fn determine_status(
    last_update: Option<DateTime<Utc>>,
    threshold_hours: i64,
) -> (String, Option<i64>) {
    match last_update {
        Some(ts) => {
            let hours_ago = (Utc::now() - ts).num_hours();
            let status = if hours_ago <= threshold_hours {
                "fresh".to_string()
            } else if hours_ago <= threshold_hours * 3 {
                "stale".to_string()
            } else {
                "outdated".to_string()
            };
            (status, Some(hours_ago))
        }
        None => ("no_data".to_string(), None),
    }
}

async fn get_data_status(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<DataStatusResponse>, (axum::http::StatusCode, String)> {
    let pool = &state.db;

    let total_stocks: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM stocks WHERE is_active = true")
        .fetch_one(pool)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let stocks_with_prices: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT symbol) FROM stock_prices WHERE time > NOW() - INTERVAL '30 days'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let stocks_with_scores: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT symbol) FROM stock_scores WHERE time > NOW() - INTERVAL '7 days'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let stocks_with_broker: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT symbol) FROM broker_summary WHERE time > NOW() - INTERVAL '30 days'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let latest_price: Option<(DateTime<Utc>,)> =
        sqlx::query_as("SELECT MAX(time) FROM stock_prices")
            .fetch_optional(pool)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let oldest_price: Option<(DateTime<Utc>,)> =
        sqlx::query_as("SELECT MIN(time) FROM stock_prices")
            .fetch_optional(pool)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let latest_broker: Option<(DateTime<Utc>,)> =
        sqlx::query_as("SELECT MAX(time) FROM broker_summary")
            .fetch_optional(pool)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let latest_score: Option<(DateTime<Utc>,)> =
        sqlx::query_as("SELECT MAX(time) FROM stock_scores")
            .fetch_optional(pool)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let price_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM stock_prices")
        .fetch_one(pool)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let broker_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM broker_summary")
        .fetch_one(pool)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let score_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM stock_scores")
        .fetch_one(pool)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let financials_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM financials")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    let latest_financials: Option<(DateTime<Utc>,)> =
        sqlx::query_as("SELECT MAX(created_at) FROM financials")
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    let (price_status, price_hours) = determine_status(latest_price.map(|r| r.0), 24);
    let (broker_status, broker_hours) = determine_status(latest_broker.map(|r| r.0), 24);
    let (score_status, score_hours) = determine_status(latest_score.map(|r| r.0), 24);
    let (fundamentals_status, fundamentals_hours) =
        determine_status(latest_financials.map(|r| r.0), 168);

    let sources = vec![
        DataSourceStatus {
            id: "prices".to_string(),
            name: "Yahoo Finance Prices".to_string(),
            source_type: "OHLCV Historical Data".to_string(),
            last_update: latest_price.map(|r| r.0),
            record_count: price_count.0,
            status: price_status.clone(),
            freshness_hours: price_hours,
            can_refresh: true,
        },
        DataSourceStatus {
            id: "broker".to_string(),
            name: "Broker Summary".to_string(),
            source_type: "Stockbit/IDX Broker Flow".to_string(),
            last_update: latest_broker.map(|r| r.0),
            record_count: broker_count.0,
            status: broker_status.clone(),
            freshness_hours: broker_hours,
            can_refresh: true,
        },
        DataSourceStatus {
            id: "scores".to_string(),
            name: "Computed Scores".to_string(),
            source_type: "Technical/Fundamental/ML Scores".to_string(),
            last_update: latest_score.map(|r| r.0),
            record_count: score_count.0,
            status: score_status.clone(),
            freshness_hours: score_hours,
            can_refresh: true,
        },
        DataSourceStatus {
            id: "fundamentals".to_string(),
            name: "Fundamentals".to_string(),
            source_type: "yfinance/Sectors.app Financials".to_string(),
            last_update: latest_financials.map(|r| r.0),
            record_count: financials_count.0,
            status: fundamentals_status.clone(),
            freshness_hours: fundamentals_hours,
            can_refresh: true,
        },
    ];

    let all_fresh = sources.iter().all(|s| s.status == "fresh");
    let any_outdated = sources
        .iter()
        .any(|s| s.status == "outdated" || s.status == "no_data");

    let overall_status = if all_fresh {
        "healthy".to_string()
    } else if any_outdated {
        "degraded".to_string()
    } else {
        "warning".to_string()
    };

    Ok(Json(DataStatusResponse {
        timestamp: Utc::now(),
        overall_status,
        sources,
        summary: DataSummary {
            total_stocks: total_stocks.0,
            stocks_with_prices: stocks_with_prices.0,
            stocks_with_scores: stocks_with_scores.0,
            stocks_with_broker_data: stocks_with_broker.0,
            oldest_price_data: oldest_price.map(|r| r.0),
            newest_price_data: latest_price.map(|r| r.0),
        },
    }))
}

async fn get_source_status(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<String>,
) -> Result<Json<DataSourceStatus>, (axum::http::StatusCode, String)> {
    let pool = &state.db;

    let source = match source_id.as_str() {
        "prices" => {
            let latest: Option<(DateTime<Utc>,)> =
                sqlx::query_as("SELECT MAX(time) FROM stock_prices")
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM stock_prices")
                .fetch_one(pool)
                .await
                .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let (status, hours) = determine_status(latest.map(|r| r.0), 24);
            DataSourceStatus {
                id: "prices".to_string(),
                name: "Yahoo Finance Prices".to_string(),
                source_type: "OHLCV Historical Data".to_string(),
                last_update: latest.map(|r| r.0),
                record_count: count.0,
                status,
                freshness_hours: hours,
                can_refresh: true,
            }
        }
        "broker" => {
            let latest: Option<(DateTime<Utc>,)> =
                sqlx::query_as("SELECT MAX(time) FROM broker_summary")
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM broker_summary")
                .fetch_one(pool)
                .await
                .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let (status, hours) = determine_status(latest.map(|r| r.0), 24);
            DataSourceStatus {
                id: "broker".to_string(),
                name: "Broker Summary".to_string(),
                source_type: "Stockbit/IDX Broker Flow".to_string(),
                last_update: latest.map(|r| r.0),
                record_count: count.0,
                status,
                freshness_hours: hours,
                can_refresh: true,
            }
        }
        "scores" => {
            let latest: Option<(DateTime<Utc>,)> =
                sqlx::query_as("SELECT MAX(time) FROM stock_scores")
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM stock_scores")
                .fetch_one(pool)
                .await
                .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let (status, hours) = determine_status(latest.map(|r| r.0), 24);
            DataSourceStatus {
                id: "scores".to_string(),
                name: "Computed Scores".to_string(),
                source_type: "Technical/Fundamental/ML Scores".to_string(),
                last_update: latest.map(|r| r.0),
                record_count: count.0,
                status,
                freshness_hours: hours,
                can_refresh: true,
            }
        }
        "fundamentals" => {
            let latest: Option<(DateTime<Utc>,)> =
                sqlx::query_as("SELECT MAX(created_at) FROM financials")
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten();
            let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM financials")
                .fetch_one(pool)
                .await
                .unwrap_or((0,));
            let (status, hours) = determine_status(latest.map(|r| r.0), 168);
            DataSourceStatus {
                id: "fundamentals".to_string(),
                name: "Fundamentals".to_string(),
                source_type: "yfinance/Sectors.app Financials".to_string(),
                last_update: latest.map(|r| r.0),
                record_count: count.0,
                status,
                freshness_hours: hours,
                can_refresh: true,
            }
        }
        _ => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                format!("Unknown source: {}", source_id),
            ))
        }
    };

    Ok(Json(source))
}

async fn refresh_source(
    _user: AuthUser,
    Path(source_id): Path<String>,
) -> Result<Json<RefreshResponse>, (axum::http::StatusCode, String)> {
    let message = match source_id.as_str() {
        "prices" => "Price data refresh queued. Run: python -m jejakcuan_ml.scrapers.price_history",
        "broker" => {
            "Broker data refresh queued. Run: python -m jejakcuan_ml.scrapers.broker_summary"
        }
        "scores" => "Score recomputation queued. Use POST /api/stocks/scores/recompute endpoint",
        "fundamentals" => {
            "Fundamentals refresh queued. Run: python -m jejakcuan_ml.scrapers.fundamentals"
        }
        _ => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                format!("Unknown source: {}", source_id),
            ))
        }
    };

    Ok(Json(RefreshResponse {
        source_id: source_id.clone(),
        status: "queued".to_string(),
        message: message.to_string(),
        started_at: Utc::now(),
    }))
}

fn check_env_var_configured(var_name: &str) -> bool {
    std::env::var(var_name)
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

fn get_config_status(definition: &DataSourceDefinition) -> ConfigStatus {
    let mut missing_fields = Vec::new();
    let config_fields: Vec<ConfigFieldStatus> = definition
        .config_fields
        .iter()
        .map(|field| {
            let is_set = check_env_var_configured(&field.env_var);
            if field.required && !is_set {
                missing_fields.push(field.name.clone());
            }
            ConfigFieldStatus {
                name: field.name.clone(),
                description: field.description.clone(),
                env_var: field.env_var.clone(),
                required: field.required,
                is_set,
            }
        })
        .collect();

    let is_configured = missing_fields.is_empty();

    ConfigStatus {
        is_configured,
        missing_fields,
        config_fields,
    }
}

fn determine_source_state(
    last_update: Option<DateTime<Utc>>,
    threshold_hours: i64,
    is_configured: bool,
) -> (DataSourceState, Option<i64>) {
    if !is_configured {
        return (DataSourceState::NotConfigured, None);
    }

    match last_update {
        Some(ts) => {
            let hours_ago = (Utc::now() - ts).num_hours();
            let state = if hours_ago <= threshold_hours {
                DataSourceState::Fresh
            } else if hours_ago <= threshold_hours * 3 {
                DataSourceState::Stale
            } else {
                DataSourceState::Outdated
            };
            (state, Some(hours_ago))
        }
        None => (DataSourceState::NoData, None),
    }
}

async fn get_table_stats(
    pool: &sqlx::PgPool,
    table_name: &str,
) -> Result<(Option<DateTime<Utc>>, i64), sqlx::Error> {
    let time_column = if table_name == "financials" {
        "created_at"
    } else {
        "time"
    };

    let latest_query = format!("SELECT MAX({}) FROM {}", time_column, table_name);
    let count_query = format!("SELECT COUNT(*) FROM {}", table_name);

    let latest: Option<(Option<DateTime<Utc>>,)> =
        sqlx::query_as(&latest_query).fetch_optional(pool).await?;

    let count: (i64,) = sqlx::query_as(&count_query).fetch_one(pool).await?;

    Ok((latest.and_then(|r| r.0), count.0))
}

async fn build_granular_source(
    pool: &sqlx::PgPool,
    definition: &DataSourceDefinition,
) -> Result<GranularDataSource, sqlx::Error> {
    let config_status = get_config_status(definition);

    let (last_update, record_count) = if let Some(table) = definition.db_table {
        get_table_stats(pool, table).await.unwrap_or((None, 0))
    } else {
        (None, 0)
    };

    let (state, freshness_hours) = determine_source_state(
        last_update,
        definition.freshness_threshold_hours,
        config_status.is_configured,
    );

    let can_trigger = config_status.is_configured
        && (definition.trigger_command.is_some()
            || matches!(
                definition.source_type,
                SourceType::Computed | SourceType::RustClient
            ));

    Ok(GranularDataSource {
        id: definition.id.to_string(),
        name: definition.name.to_string(),
        category: definition.category,
        category_name: definition.category.display_name().to_string(),
        source_type: definition.source_type,
        description: definition.description.to_string(),
        status: state,
        config_status,
        last_update,
        record_count,
        freshness_hours,
        can_trigger,
        trigger_command: definition.trigger_command.map(|s| s.to_string()),
    })
}

async fn list_data_sources(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<DataSourcesResponse>, (axum::http::StatusCode, String)> {
    let pool = &state.db;
    let registry = get_data_source_registry();

    let mut sources = Vec::new();
    for definition in &registry {
        let source = build_granular_source(pool, definition)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        sources.push(source);
    }

    let mut by_category: HashMap<String, Vec<GranularDataSource>> = HashMap::new();
    for source in &sources {
        by_category
            .entry(source.category.as_str().to_string())
            .or_default()
            .push(source.clone());
    }

    let configured_count = sources
        .iter()
        .filter(|s| s.config_status.is_configured)
        .count();
    let fresh_count = sources
        .iter()
        .filter(|s| matches!(s.status, DataSourceState::Fresh))
        .count();
    let stale_count = sources
        .iter()
        .filter(|s| {
            matches!(
                s.status,
                DataSourceState::Stale | DataSourceState::Outdated | DataSourceState::NoData
            )
        })
        .count();

    let categories = [
        DataSourceCategory::Broker,
        DataSourceCategory::Prices,
        DataSourceCategory::Fundamentals,
        DataSourceCategory::Scores,
    ]
    .iter()
    .map(|cat| {
        let cat_sources: Vec<_> = sources.iter().filter(|s| s.category == *cat).collect();
        CategorySummary {
            category: cat.as_str().to_string(),
            display_name: cat.display_name().to_string(),
            total: cat_sources.len(),
            fresh: cat_sources
                .iter()
                .filter(|s| matches!(s.status, DataSourceState::Fresh))
                .count(),
            stale: cat_sources
                .iter()
                .filter(|s| {
                    matches!(
                        s.status,
                        DataSourceState::Stale
                            | DataSourceState::Outdated
                            | DataSourceState::NoData
                    )
                })
                .count(),
            not_configured: cat_sources
                .iter()
                .filter(|s| !s.config_status.is_configured)
                .count(),
        }
    })
    .collect();

    let all_fresh = fresh_count == sources.len();
    let any_not_configured = sources.iter().any(|s| !s.config_status.is_configured);
    let any_outdated = sources.iter().any(|s| {
        matches!(
            s.status,
            DataSourceState::Outdated | DataSourceState::NoData
        )
    });

    let overall_status = if all_fresh {
        "healthy"
    } else if any_outdated || any_not_configured {
        "degraded"
    } else {
        "warning"
    }
    .to_string();

    Ok(Json(DataSourcesResponse {
        timestamp: Utc::now(),
        overall_status,
        sources,
        by_category,
        summary: DataSourcesSummary {
            total_sources: registry.len(),
            configured_sources: configured_count,
            fresh_sources: fresh_count,
            stale_sources: stale_count,
            categories,
        },
    }))
}

async fn get_data_source(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<String>,
) -> Result<Json<GranularDataSource>, (axum::http::StatusCode, String)> {
    let pool = &state.db;
    let registry = get_data_source_registry();

    let definition = registry.iter().find(|d| d.id == source_id).ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            format!("Unknown data source: {}", source_id),
        )
    })?;

    let source = build_granular_source(pool, definition)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(source))
}

async fn trigger_data_source(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<String>,
) -> Result<Json<TriggerResponse>, (axum::http::StatusCode, String)> {
    let registry = get_data_source_registry();

    let definition = registry.iter().find(|d| d.id == source_id).ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            format!("Unknown data source: {}", source_id),
        )
    })?;

    let config_status = get_config_status(definition);
    if !config_status.is_configured {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            format!(
                "Data source '{}' is not configured. Missing: {}",
                source_id,
                config_status.missing_fields.join(", ")
            ),
        ));
    }

    if let Some(running_job) = state.job_manager.is_source_running(&source_id).await {
        return Ok(Json(TriggerResponse {
            source_id: source_id.clone(),
            status: "already_running".to_string(),
            message: format!("Job {} is already running for this source", running_job.id),
            command: Some(running_job.command.clone()),
            started_at: running_job.started_at,
            job_id: Some(running_job.id.clone()),
            job: Some(running_job),
        }));
    }

    let (status, message, command, job_id, job) = match definition.source_type {
        SourceType::PythonScraper => {
            if let Some(cmd) = definition.trigger_command {
                let job = state
                    .job_manager
                    .spawn_job(
                        source_id.clone(),
                        definition.name.to_string(),
                        cmd.to_string(),
                    )
                    .await;

                (
                    "started",
                    format!("Background job started: {}", job.id),
                    Some(cmd.to_string()),
                    Some(job.id.clone()),
                    Some(job),
                )
            } else {
                ("error", "No trigger command configured".to_string(), None, None, None)
            }
        }
        SourceType::RustClient => (
            "available",
            format!(
                "Data source '{}' uses a Rust client. Trigger via the appropriate API endpoint.",
                definition.name
            ),
            None,
            None,
            None,
        ),
        SourceType::Computed => (
            "available",
            format!(
                "Score '{}' is computed from other data. Use POST /api/stocks/scores/recompute to refresh all scores.",
                definition.name
            ),
            None,
            None,
            None,
        ),
    };

    Ok(Json(TriggerResponse {
        source_id: source_id.clone(),
        status: status.to_string(),
        message,
        command,
        started_at: Utc::now(),
        job_id,
        job,
    }))
}

#[derive(Debug, Serialize)]
pub struct JobsListResponse {
    pub jobs: Vec<Job>,
    pub count: usize,
}

async fn list_jobs(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<JobsListResponse>, (axum::http::StatusCode, String)> {
    let jobs = state.job_manager.get_recent_jobs(50).await;
    let count = jobs.len();
    Ok(Json(JobsListResponse { jobs, count }))
}

async fn get_job(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<Json<Job>, (axum::http::StatusCode, String)> {
    state
        .job_manager
        .get_job(&job_id)
        .await
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Job not found: {}", job_id),
            )
        })
        .map(Json)
}

async fn get_source_jobs(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<String>,
) -> Result<Json<JobsListResponse>, (axum::http::StatusCode, String)> {
    let jobs = state.job_manager.get_jobs_for_source(&source_id).await;
    let count = jobs.len();
    Ok(Json(JobsListResponse { jobs, count }))
}

async fn trigger_category(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(category_str): Path<String>,
) -> Result<Json<CategoryTriggerResponse>, (axum::http::StatusCode, String)> {
    let category: DataSourceCategory = category_str.parse().map_err(|e: String| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Invalid category: {}", e),
        )
    })?;

    let registry = get_data_source_registry();
    let category_sources: Vec<_> = registry.iter().filter(|d| d.category == category).collect();

    let mut triggered = Vec::new();
    let mut skipped = Vec::new();

    for definition in category_sources {
        let config_status = get_config_status(definition);

        if !config_status.is_configured {
            skipped.push(SkippedSource {
                source_id: definition.id.to_string(),
                reason: format!(
                    "Not configured. Missing: {}",
                    config_status.missing_fields.join(", ")
                ),
            });
            continue;
        }

        if let Some(running_job) = state.job_manager.is_source_running(definition.id).await {
            skipped.push(SkippedSource {
                source_id: definition.id.to_string(),
                reason: format!("Already running job: {}", running_job.id),
            });
            continue;
        }

        let (status, message, command, job_id, job) = match definition.source_type {
            SourceType::PythonScraper => {
                if let Some(cmd) = definition.trigger_command {
                    let job = state
                        .job_manager
                        .spawn_job(
                            definition.id.to_string(),
                            definition.name.to_string(),
                            cmd.to_string(),
                        )
                        .await;

                    (
                        "started",
                        format!("Background job started: {}", job.id),
                        Some(cmd.to_string()),
                        Some(job.id.clone()),
                        Some(job),
                    )
                } else {
                    skipped.push(SkippedSource {
                        source_id: definition.id.to_string(),
                        reason: "No trigger command".to_string(),
                    });
                    continue;
                }
            }
            SourceType::RustClient => (
                "available",
                "Use appropriate API endpoint".to_string(),
                None,
                None,
                None,
            ),
            SourceType::Computed => (
                "available",
                "Use POST /api/stocks/scores/recompute".to_string(),
                None,
                None,
                None,
            ),
        };

        triggered.push(TriggerResponse {
            source_id: definition.id.to_string(),
            status: status.to_string(),
            message,
            command,
            started_at: Utc::now(),
            job_id,
            job,
        });
    }

    Ok(Json(CategoryTriggerResponse {
        category: category_str,
        triggered,
        skipped,
    }))
}

async fn get_source_config(
    _user: AuthUser,
    Path(source_id): Path<String>,
) -> Result<Json<ConfigResponse>, (axum::http::StatusCode, String)> {
    let registry = get_data_source_registry();

    let definition = registry.iter().find(|d| d.id == source_id).ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            format!("Unknown data source: {}", source_id),
        )
    })?;

    let config_status = get_config_status(definition);

    Ok(Json(ConfigResponse {
        source_id: definition.id.to_string(),
        source_name: definition.name.to_string(),
        fields: config_status.config_fields,
        is_configured: config_status.is_configured,
    }))
}
