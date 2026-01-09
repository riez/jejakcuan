//! API routes

pub mod auth;
pub mod stocks;
pub mod watchlist;

pub use auth::auth_routes;
pub use stocks::stock_routes;
pub use watchlist::watchlist_routes;
