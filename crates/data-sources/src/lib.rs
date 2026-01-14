//! Data source adapters for JejakCuan
//!
//! This crate handles fetching data from external APIs:
//! - IDX (Indonesia Stock Exchange) data via Yahoo Finance
//! - Yahoo Finance for stock quotes and historical data
//! - Sectors.app for Indonesian market data and financials
//! - Broker summary data for bandarmology analysis
//! - News sources for sentiment analysis
//! - Shareholding data from KSEI/OJK for ownership tracking

pub mod broker;
pub mod error;
pub mod sectors;
pub mod shareholding;
pub mod twelvedata;
pub mod yahoo;

pub use broker::{
    get_broker_category, is_foreign_broker, is_institutional_broker, BrokerAccumulationScore,
    BrokerActivity, BrokerCategory, BrokerScraper, BrokerSummary,
};
pub use error::DataSourceError;
pub use sectors::{
    CompaniesResponse, CompanyFinancials, CompanyQuery, DailyTransaction, Industry,
    KeyExecutive, MajorShareholder, SectorsClient, SectorsCompany, SectorsPagination,
    StockMover, Subsector, TopMovers,
};
pub use twelvedata::{
    ExchangeInfo, Interval, LatestPrice, MarketMover, MarketMoversResponse, PriceUpdate,
    Quote, StockInfo, TimeSeriesMeta, TimeSeriesPoint, TimeSeriesResponse,
    TwelveDataClient, TwelveDataWebSocket, WebSocketEvent,
};
pub use shareholding::{
    ConcentrationMetrics, InsiderActivityScore, InstitutionalFlow, OwnershipChange, Shareholder,
    ShareholderType, ShareholdingScore, ShareholdingScraper, ShareholdingSnapshot,
    ShareholdingSource,
};
pub use yahoo::YahooFinanceClient;
