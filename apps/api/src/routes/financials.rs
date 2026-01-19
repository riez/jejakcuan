//! Financial statements routes

use crate::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;

pub fn financials_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:symbol/income-statement", get(get_income_statements))
        .route("/:symbol/balance-sheet", get(get_balance_sheets))
        .route("/:symbol/cash-flow", get(get_cash_flows))
        .route("/:symbol/ratios", get(get_financial_ratios))
        .route("/:symbol/summary", get(get_financial_summary))
}

#[derive(Debug, Deserialize)]
pub struct FinancialsQuery {
    years: Option<i32>,
    quarterly: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct IncomeStatementResponse {
    pub symbol: String,
    pub fiscal_year: i32,
    pub fiscal_quarter: Option<i32>,
    pub period_end: NaiveDate,
    pub revenue: Option<i64>,
    pub gross_profit: Option<i64>,
    pub operating_income: Option<i64>,
    pub earnings_before_tax: Option<i64>,
    pub tax_expense: Option<i64>,
    pub net_income: Option<i64>,
    pub eps: Option<Decimal>,
    pub gross_margin: Option<Decimal>,
    pub operating_margin: Option<Decimal>,
    pub net_margin: Option<Decimal>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BalanceSheetResponse {
    pub symbol: String,
    pub fiscal_year: i32,
    pub fiscal_quarter: Option<i32>,
    pub period_end: NaiveDate,
    pub total_assets: Option<i64>,
    pub total_liabilities: Option<i64>,
    pub total_equity: Option<i64>,
    pub total_debt: Option<i64>,
    pub current_ratio: Option<Decimal>,
    pub debt_to_equity: Option<Decimal>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CashFlowResponse {
    pub symbol: String,
    pub fiscal_year: i32,
    pub fiscal_quarter: Option<i32>,
    pub period_end: NaiveDate,
    pub operating_cash_flow: Option<i64>,
    pub capital_expenditure: Option<i64>,
    pub free_cash_flow: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FinancialRatiosResponse {
    pub symbol: String,
    pub fiscal_year: i32,
    pub fiscal_quarter: Option<i32>,
    pub period_end: NaiveDate,
    pub roe: Option<Decimal>,
    pub roa: Option<Decimal>,
    pub gross_margin: Option<Decimal>,
    pub operating_margin: Option<Decimal>,
    pub net_margin: Option<Decimal>,
    pub current_ratio: Option<Decimal>,
    pub debt_to_equity: Option<Decimal>,
    pub eps: Option<Decimal>,
    pub revenue_growth: Option<Decimal>,
    pub earnings_growth: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct FinancialSummaryResponse {
    pub symbol: String,
    pub latest_year: Option<i32>,
    pub income_statements: Vec<IncomeStatementResponse>,
    pub balance_sheets: Vec<BalanceSheetResponse>,
    pub cash_flows: Vec<CashFlowResponse>,
    pub ratios: Vec<FinancialRatiosResponse>,
}

async fn get_income_statements(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(query): Query<FinancialsQuery>,
) -> Result<Json<Vec<IncomeStatementResponse>>, (axum::http::StatusCode, String)> {
    let upper_symbol = symbol.to_uppercase();
    let years = query.years.unwrap_or(5) as i64;
    let quarterly = query.quarterly.unwrap_or(false);

    let rows: Vec<IncomeStatementResponse> = sqlx::query_as(
        r#"
        SELECT symbol, fiscal_year, fiscal_quarter, period_end, revenue, gross_profit,
               operating_income, earnings_before_tax, tax_expense, net_income, eps,
               gross_margin, operating_margin, net_margin
        FROM income_statements
        WHERE symbol = $1 
          AND ($2 = true OR fiscal_quarter IS NULL)
        ORDER BY fiscal_year DESC, fiscal_quarter DESC NULLS FIRST
        LIMIT $3
        "#,
    )
    .bind(&upper_symbol)
    .bind(quarterly)
    .bind(years)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}

async fn get_balance_sheets(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(query): Query<FinancialsQuery>,
) -> Result<Json<Vec<BalanceSheetResponse>>, (axum::http::StatusCode, String)> {
    let upper_symbol = symbol.to_uppercase();
    let years = query.years.unwrap_or(5) as i64;
    let quarterly = query.quarterly.unwrap_or(false);

    let rows: Vec<BalanceSheetResponse> = sqlx::query_as(
        r#"
        SELECT symbol, fiscal_year, fiscal_quarter, period_end, total_assets,
               total_liabilities, total_equity, total_debt, current_ratio, debt_to_equity
        FROM balance_sheets
        WHERE symbol = $1
          AND ($2 = true OR fiscal_quarter IS NULL)
        ORDER BY fiscal_year DESC, fiscal_quarter DESC NULLS FIRST
        LIMIT $3
        "#,
    )
    .bind(&upper_symbol)
    .bind(quarterly)
    .bind(years)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}

async fn get_cash_flows(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(query): Query<FinancialsQuery>,
) -> Result<Json<Vec<CashFlowResponse>>, (axum::http::StatusCode, String)> {
    let upper_symbol = symbol.to_uppercase();
    let years = query.years.unwrap_or(5) as i64;
    let quarterly = query.quarterly.unwrap_or(false);

    let rows: Vec<CashFlowResponse> = sqlx::query_as(
        r#"
        SELECT symbol, fiscal_year, fiscal_quarter, period_end, operating_cash_flow,
               capital_expenditure, free_cash_flow
        FROM cash_flow_statements
        WHERE symbol = $1
          AND ($2 = true OR fiscal_quarter IS NULL)
        ORDER BY fiscal_year DESC, fiscal_quarter DESC NULLS FIRST
        LIMIT $3
        "#,
    )
    .bind(&upper_symbol)
    .bind(quarterly)
    .bind(years)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}

async fn get_financial_ratios(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(query): Query<FinancialsQuery>,
) -> Result<Json<Vec<FinancialRatiosResponse>>, (axum::http::StatusCode, String)> {
    let upper_symbol = symbol.to_uppercase();
    let years = query.years.unwrap_or(5) as i64;
    let quarterly = query.quarterly.unwrap_or(false);

    let rows: Vec<FinancialRatiosResponse> = sqlx::query_as(
        r#"
        SELECT symbol, fiscal_year, fiscal_quarter, period_end, roe, roa,
               gross_margin, operating_margin, net_margin, current_ratio,
               debt_to_equity, eps, revenue_growth, earnings_growth
        FROM financial_ratios
        WHERE symbol = $1
          AND ($2 = true OR fiscal_quarter IS NULL)
        ORDER BY fiscal_year DESC, fiscal_quarter DESC NULLS FIRST
        LIMIT $3
        "#,
    )
    .bind(&upper_symbol)
    .bind(quarterly)
    .bind(years)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}

async fn get_financial_summary(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(query): Query<FinancialsQuery>,
) -> Result<Json<FinancialSummaryResponse>, (axum::http::StatusCode, String)> {
    let upper_symbol = symbol.to_uppercase();
    let years = query.years.unwrap_or(5) as i64;

    let income_statements: Vec<IncomeStatementResponse> = sqlx::query_as(
        r#"
        SELECT symbol, fiscal_year, fiscal_quarter, period_end, revenue, gross_profit,
               operating_income, earnings_before_tax, tax_expense, net_income, eps,
               gross_margin, operating_margin, net_margin
        FROM income_statements
        WHERE symbol = $1 AND fiscal_quarter IS NULL
        ORDER BY fiscal_year DESC
        LIMIT $2
        "#,
    )
    .bind(&upper_symbol)
    .bind(years)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let balance_sheets: Vec<BalanceSheetResponse> = sqlx::query_as(
        r#"
        SELECT symbol, fiscal_year, fiscal_quarter, period_end, total_assets,
               total_liabilities, total_equity, total_debt, current_ratio, debt_to_equity
        FROM balance_sheets
        WHERE symbol = $1 AND fiscal_quarter IS NULL
        ORDER BY fiscal_year DESC
        LIMIT $2
        "#,
    )
    .bind(&upper_symbol)
    .bind(years)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let cash_flows: Vec<CashFlowResponse> = sqlx::query_as(
        r#"
        SELECT symbol, fiscal_year, fiscal_quarter, period_end, operating_cash_flow,
               capital_expenditure, free_cash_flow
        FROM cash_flow_statements
        WHERE symbol = $1 AND fiscal_quarter IS NULL
        ORDER BY fiscal_year DESC
        LIMIT $2
        "#,
    )
    .bind(&upper_symbol)
    .bind(years)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ratios: Vec<FinancialRatiosResponse> = sqlx::query_as(
        r#"
        SELECT symbol, fiscal_year, fiscal_quarter, period_end, roe, roa,
               gross_margin, operating_margin, net_margin, current_ratio,
               debt_to_equity, eps, revenue_growth, earnings_growth
        FROM financial_ratios
        WHERE symbol = $1 AND fiscal_quarter IS NULL
        ORDER BY fiscal_year DESC
        LIMIT $2
        "#,
    )
    .bind(&upper_symbol)
    .bind(years)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let latest_year = income_statements.first().map(|s| s.fiscal_year);

    Ok(Json(FinancialSummaryResponse {
        symbol: upper_symbol,
        latest_year,
        income_statements,
        balance_sheets,
        cash_flows,
        ratios,
    }))
}
