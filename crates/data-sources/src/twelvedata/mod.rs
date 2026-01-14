//! TwelveData API client for real-time market data streaming
//!
//! Provides both REST and WebSocket access to TwelveData financial data:
//! - Real-time price streaming via WebSocket (170ms latency target)
//! - Historical time series data
//! - Market quotes and movers
//!
//! # WebSocket Features
//! - Auto-reconnection with exponential backoff
//! - Subscription management for multiple symbols
//! - Backpressure handling

mod client;
mod models;
mod websocket;

pub use client::TwelveDataClient;
pub use models::*;
pub use websocket::{TwelveDataWebSocket, WebSocketEvent};
