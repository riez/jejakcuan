//! Data source adapters for JejakCuan
//!
//! This crate handles fetching data from external APIs:
//! - IDX (Indonesia Stock Exchange) data via Yahoo Finance
//! - Yahoo Finance for stock quotes and historical data
//! - Broker summary data for bandarmology analysis
//! - News sources for sentiment analysis

pub mod broker;
pub mod error;
pub mod yahoo;

pub use broker::{
    get_broker_category, is_foreign_broker, is_institutional_broker, BrokerAccumulationScore,
    BrokerActivity, BrokerCategory, BrokerScraper, BrokerSummary,
};
pub use error::DataSourceError;
pub use yahoo::YahooFinanceClient;
