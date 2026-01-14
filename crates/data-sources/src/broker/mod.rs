//! Broker summary data and analysis for bandarmology
//!
//! Provides:
//! - Broker classification (foreign/local institutional, retail)
//! - Data scraping from IDX
//! - Rolling accumulation detection (5-day, 20-day)
//! - Coordinated buying analysis

mod analysis;
mod classification;
mod models;
mod scraper;

pub use analysis::*;
pub use classification::*;
pub use models::*;
pub use scraper::*;
