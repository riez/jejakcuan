//! API routes

pub mod analysis;
pub mod auth;
pub mod stocks;
pub mod streaming;
pub mod watchlist;

pub use analysis::analysis_routes;
pub use auth::auth_routes;
pub use stocks::stock_routes;
pub use streaming::streaming_routes;
pub use watchlist::watchlist_routes;
