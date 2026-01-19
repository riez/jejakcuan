//! Data models for Sectors.app API responses

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Company basic information from Sectors.app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorsCompany {
    pub symbol: String,
    pub company_name: String,
    #[serde(default)]
    pub listing_board: Option<String>,
    #[serde(default)]
    pub industry: Option<String>,
    #[serde(default)]
    pub sub_industry: Option<String>,
    #[serde(default)]
    pub sector: Option<String>,
    #[serde(default)]
    pub sub_sector: Option<String>,
    #[serde(default)]
    pub market_cap: Option<i64>,
    #[serde(default)]
    pub market_cap_rank: Option<i32>,
    #[serde(default)]
    pub employee_num: Option<i32>,
    #[serde(default)]
    pub listing_date: Option<NaiveDate>,
    #[serde(default)]
    pub last_close_price: Option<Decimal>,
    #[serde(default)]
    pub daily_close_change: Option<Decimal>,
    #[serde(default)]
    pub forward_pe: Option<Decimal>,
    #[serde(default)]
    pub yield_ttm: Option<Decimal>,
    #[serde(default)]
    pub pe_ttm: Option<Decimal>,
    #[serde(default)]
    pub pb_mrq: Option<Decimal>,
    #[serde(default)]
    pub roe_ttm: Option<Decimal>,
    #[serde(default)]
    pub roa_ttm: Option<Decimal>,
}

/// Pagination info from Sectors.app responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorsPagination {
    pub total_count: i32,
    pub showing: i32,
    pub limit: i32,
    pub offset: i32,
    pub has_next: bool,
    pub has_previous: bool,
    pub next_offset: Option<i32>,
    pub previous_offset: Option<i32>,
}

/// Companies screener response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompaniesResponse {
    pub results: Vec<SectorsCompany>,
    pub pagination: SectorsPagination,
}

/// Subsector information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subsector {
    pub sector: String,
    pub subsector: String,
}

/// Industry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Industry {
    pub subsector: String,
    pub industry: String,
}

/// Company financial data for a specific year
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyFinancials {
    pub symbol: String,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub revenue: Option<i64>,
    #[serde(default)]
    pub earnings: Option<i64>,
    #[serde(default)]
    pub total_assets: Option<i64>,
    #[serde(default)]
    pub total_equity: Option<i64>,
    #[serde(default)]
    pub total_liabilities: Option<i64>,
    #[serde(default)]
    pub total_debt: Option<i64>,
    #[serde(default)]
    pub operating_cash_flow: Option<i64>,
    #[serde(default)]
    pub free_cash_flow: Option<i64>,
    #[serde(default)]
    pub eps: Option<Decimal>,
    #[serde(default)]
    pub roe: Option<Decimal>,
    #[serde(default)]
    pub roa: Option<Decimal>,
    #[serde(default)]
    pub net_profit_margin: Option<Decimal>,
    #[serde(default)]
    pub gross_profit_margin: Option<Decimal>,
    #[serde(default)]
    pub debt_to_equity_ratio: Option<Decimal>,
    #[serde(default)]
    pub current_ratio: Option<Decimal>,
}

/// Major shareholder information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MajorShareholder {
    pub name: String,
    #[serde(default)]
    pub share_amount: Option<i64>,
    #[serde(default)]
    pub share_percentage: Option<Decimal>,
    #[serde(default)]
    pub share_value: Option<i64>,
}

/// Key executive information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExecutive {
    pub name: String,
    #[serde(default)]
    pub position: Option<String>,
}

/// Historical financial data from Sectors.app company report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalFinancial {
    pub year: i32,
    #[serde(default)]
    pub revenue: Option<i64>,
    #[serde(default)]
    pub earnings: Option<i64>,
    #[serde(default)]
    pub tax: Option<i64>,
    #[serde(default)]
    pub gross_profit: Option<i64>,
    #[serde(default)]
    pub operating_pnl: Option<i64>,
    #[serde(default)]
    pub earnings_before_tax: Option<i64>,
    #[serde(default)]
    pub total_assets: Option<i64>,
    #[serde(default)]
    pub total_equity: Option<i64>,
    #[serde(default)]
    pub total_liabilities: Option<i64>,
    #[serde(default)]
    pub total_debt: Option<i64>,
    #[serde(default)]
    pub operating_cash_flow: Option<i64>,
    #[serde(default)]
    pub free_cash_flow: Option<i64>,
    #[serde(default)]
    pub capex: Option<i64>,
    #[serde(default)]
    pub eps: Option<Decimal>,
    #[serde(default)]
    pub roe: Option<Decimal>,
    #[serde(default)]
    pub roa: Option<Decimal>,
    #[serde(default)]
    pub net_profit_margin: Option<Decimal>,
    #[serde(default)]
    pub gross_profit_margin: Option<Decimal>,
    #[serde(default)]
    pub operating_margin: Option<Decimal>,
    #[serde(default)]
    pub debt_to_equity: Option<Decimal>,
    #[serde(default)]
    pub current_ratio: Option<Decimal>,
}

/// Company overview data from Sectors.app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyOverview {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub headquarters: Option<String>,
    #[serde(default)]
    pub employee_num: Option<i32>,
    #[serde(default)]
    pub listing_date: Option<NaiveDate>,
    #[serde(default)]
    pub listing_board: Option<String>,
}

/// Financials section from company report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyFinancialsSection {
    #[serde(default)]
    pub historical_financials: Vec<HistoricalFinancial>,
    #[serde(default)]
    pub latest: Option<CompanyFinancials>,
}

/// Full company report from Sectors.app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyReport {
    pub symbol: String,
    #[serde(default)]
    pub company_name: Option<String>,
    #[serde(default)]
    pub sector: Option<String>,
    #[serde(default)]
    pub sub_sector: Option<String>,
    #[serde(default)]
    pub industry: Option<String>,
    #[serde(default)]
    pub sub_industry: Option<String>,
    #[serde(default)]
    pub market_cap: Option<i64>,
    #[serde(default)]
    pub last_close_price: Option<Decimal>,
    #[serde(default)]
    pub overview: Option<CompanyOverview>,
    #[serde(default)]
    pub financials: Option<CompanyFinancialsSection>,
    #[serde(default)]
    pub peers: Option<Vec<String>>,
    #[serde(default)]
    pub major_shareholders: Option<Vec<MajorShareholder>>,
    #[serde(default)]
    pub key_executives: Option<Vec<KeyExecutive>>,
}

/// Daily transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTransaction {
    pub date: NaiveDate,
    pub symbol: String,
    #[serde(default)]
    pub open: Option<Decimal>,
    #[serde(default)]
    pub high: Option<Decimal>,
    #[serde(default)]
    pub low: Option<Decimal>,
    #[serde(default)]
    pub close: Option<Decimal>,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub value: Option<i64>,
    #[serde(default)]
    pub frequency: Option<i64>,
}

/// Top movers response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopMovers {
    #[serde(default)]
    pub top_gainers: Vec<StockMover>,
    #[serde(default)]
    pub top_losers: Vec<StockMover>,
    #[serde(default)]
    pub most_active: Vec<StockMover>,
}

/// Stock mover entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMover {
    pub symbol: String,
    #[serde(default)]
    pub company_name: Option<String>,
    #[serde(default)]
    pub price: Option<Decimal>,
    #[serde(default)]
    pub change: Option<Decimal>,
    #[serde(default)]
    pub change_percent: Option<Decimal>,
    #[serde(default)]
    pub volume: Option<i64>,
}

/// Query parameters for company screener
#[derive(Debug, Clone, Default)]
pub struct CompanyQuery {
    /// Natural language query (if provided, other params are ignored)
    pub q: Option<String>,
    /// SQL-like where clause
    pub where_clause: Option<String>,
    /// Sort field (prefix with - for descending)
    pub order_by: Option<String>,
    /// Max results (max 200)
    pub limit: Option<i32>,
    /// Offset for pagination
    pub offset: Option<i32>,
}

impl CompanyQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn natural_query(mut self, q: &str) -> Self {
        self.q = Some(q.to_string());
        self
    }

    pub fn where_clause(mut self, clause: &str) -> Self {
        self.where_clause = Some(clause.to_string());
        self
    }

    pub fn order_by(mut self, order: &str) -> Self {
        self.order_by = Some(order.to_string());
        self
    }

    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit.min(200));
        self
    }

    pub fn offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Build query params for request
    pub fn to_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();

        if let Some(ref q) = self.q {
            params.push(("q", q.clone()));
        } else {
            if let Some(ref w) = self.where_clause {
                params.push(("where", w.clone()));
            }
            if let Some(ref o) = self.order_by {
                params.push(("order_by", o.clone()));
            }
        }

        if let Some(l) = self.limit {
            params.push(("limit", l.to_string()));
        }

        if let Some(o) = self.offset {
            params.push(("offset", o.to_string()));
        }

        params
    }
}
