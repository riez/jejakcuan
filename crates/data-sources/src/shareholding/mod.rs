//! Shareholding data from KSEI/OJK
//!
//! This module handles fetching and analyzing shareholding data:
//! - KSEI (Kustodian Sentral Efek Indonesia) for custody data
//! - OJK (Otoritas Jasa Keuangan) for regulatory filings
//!
//! Key features:
//! - Track insider ownership changes
//! - Monitor institutional accumulation
//! - Analyze ownership concentration

mod analysis;
mod models;
mod scraper;

pub use analysis::*;
pub use models::*;
pub use scraper::*;
