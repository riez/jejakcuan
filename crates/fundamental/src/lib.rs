//! Fundamental analysis for JejakCuan
//!
//! This crate implements fundamental analysis metrics:
//! - Price-to-Earnings (P/E) ratio
//! - Price-to-Book (P/B) ratio
//! - Debt-to-Equity ratio
//! - Return on Equity (ROE)
//! - Dividend yield analysis
//! - DCF (Discounted Cash Flow) valuation

pub mod dcf;
pub mod error;

pub use dcf::*;
pub use error::*;
