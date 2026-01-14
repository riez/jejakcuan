//! Sectors.app API client for Indonesian stock market data
//!
//! Sectors.app provides comprehensive IDX financial data including:
//! - Company financials and quarterly reports
//! - Sector and industry analysis
//! - Market movers and top performers
//! - Shareholding and executive data

mod client;
mod models;

pub use client::SectorsClient;
pub use models::*;
