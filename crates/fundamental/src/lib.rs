//! Fundamental analysis for JejakCuan
//!
//! This crate provides fundamental valuation metrics:
//! - P/E Ratio analysis
//! - EV/EBITDA valuation
//! - Price-to-Book ratio
//! - ROE/ROA metrics
//! - Sector peer comparison
//! - DCF (Discounted Cash Flow) valuation

pub mod dcf;
pub mod error;
pub mod metrics;
pub mod peers;

pub use dcf::*;
pub use error::*;
pub use metrics::*;
pub use peers::*;
