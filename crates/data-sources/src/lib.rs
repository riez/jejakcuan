//! Data source adapters for JejakCuan
//!
//! This crate handles fetching data from external APIs:
//! - IDX (Indonesia Stock Exchange) data via Yahoo Finance
//! - Yahoo Finance for stock quotes and historical data
//! - News sources for sentiment analysis

pub mod error;
pub mod yahoo;

pub use error::DataSourceError;
pub use yahoo::YahooFinanceClient;
