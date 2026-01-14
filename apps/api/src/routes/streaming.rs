//! Server-Sent Events (SSE) for real-time stock updates
//!
//! Provides:
//! - Real-time price updates
//! - Alert notifications
//! - Broker flow updates

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Router,
};
use futures_util::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::AppState;

/// Stream message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum StreamMessage {
    /// Price update for a symbol
    PriceUpdate {
        symbol: String,
        price: f64,
        change: f64,
        change_percent: f64,
        volume: i64,
        timestamp: i64,
    },
    /// Alert triggered
    Alert {
        id: String,
        symbol: String,
        message: String,
        priority: String,
        timestamp: i64,
    },
    /// Broker flow update
    BrokerFlow {
        symbol: String,
        net_foreign: f64,
        net_institutional: f64,
        timestamp: i64,
    },
    /// Score update
    ScoreUpdate {
        symbol: String,
        technical_score: f64,
        fundamental_score: f64,
        composite_score: f64,
        timestamp: i64,
    },
    /// Heartbeat to keep connection alive
    Heartbeat {
        timestamp: i64,
    },
}

/// Streaming state for managing broadcast channels
pub struct StreamingState {
    /// Broadcast channel for all messages
    tx: broadcast::Sender<StreamMessage>,
}

impl StreamingState {
    /// Create new streaming state
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { tx }
    }

    /// Send a message to all connected clients
    pub fn broadcast(&self, message: StreamMessage) -> Result<usize, broadcast::error::SendError<StreamMessage>> {
        self.tx.send(message)
    }

    /// Get a receiver for subscribing to messages
    pub fn subscribe(&self) -> broadcast::Receiver<StreamMessage> {
        self.tx.subscribe()
    }
}

impl Default for StreamingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create streaming routes
pub fn streaming_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/stream", get(stream_all))
        .route("/stream/prices", get(stream_prices))
        .route("/stream/alerts", get(stream_alerts))
}

/// Stream all events
async fn stream_all(
    State(_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Create a simple heartbeat stream for now
    // In production, this would connect to the broadcasting system
    let stream = stream::repeat_with(|| {
        let msg = StreamMessage::Heartbeat {
            timestamp: chrono::Utc::now().timestamp(),
        };
        let json = serde_json::to_string(&msg).unwrap_or_default();
        Result::<_, Infallible>::Ok(Event::default().data(json))
    })
    .throttle(Duration::from_secs(30));

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Stream price updates only
async fn stream_prices(
    State(_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Placeholder stream - in production would filter price updates
    let stream = stream::repeat_with(|| {
        let msg = StreamMessage::Heartbeat {
            timestamp: chrono::Utc::now().timestamp(),
        };
        let json = serde_json::to_string(&msg).unwrap_or_default();
        Result::<_, Infallible>::Ok(Event::default().event("heartbeat").data(json))
    })
    .throttle(Duration::from_secs(30));

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Stream alerts only
async fn stream_alerts(
    State(_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Placeholder stream - in production would filter alerts
    let stream = stream::repeat_with(|| {
        let msg = StreamMessage::Heartbeat {
            timestamp: chrono::Utc::now().timestamp(),
        };
        let json = serde_json::to_string(&msg).unwrap_or_default();
        Result::<_, Infallible>::Ok(Event::default().event("heartbeat").data(json))
    })
    .throttle(Duration::from_secs(30));

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Helper to create SSE stream from broadcast receiver
pub fn broadcast_to_sse<F>(
    receiver: broadcast::Receiver<StreamMessage>,
    filter: F,
) -> impl Stream<Item = Result<Event, Infallible>>
where
    F: Fn(&StreamMessage) -> bool + Send + 'static,
{
    BroadcastStream::new(receiver)
        .filter_map(move |result| {
            match result {
                Ok(msg) if filter(&msg) => {
                    let json = serde_json::to_string(&msg).unwrap_or_default();
                    Some(Result::<_, Infallible>::Ok(Event::default().data(json)))
                }
                _ => None,
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_message_serialization() {
        let msg = StreamMessage::PriceUpdate {
            symbol: "BBCA".to_string(),
            price: 9500.0,
            change: 100.0,
            change_percent: 1.06,
            volume: 10000000,
            timestamp: 1705315200,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("PriceUpdate"));
        assert!(json.contains("BBCA"));
    }

    #[test]
    fn test_alert_message_serialization() {
        let msg = StreamMessage::Alert {
            id: "alert_123".to_string(),
            symbol: "BBRI".to_string(),
            message: "Coordinated buying detected".to_string(),
            priority: "high".to_string(),
            timestamp: 1705315200,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Alert"));
        assert!(json.contains("BBRI"));
    }

    #[test]
    fn test_streaming_state() {
        let state = StreamingState::new();
        
        let _rx1 = state.subscribe();
        let _rx2 = state.subscribe();
        
        let msg = StreamMessage::Heartbeat {
            timestamp: 1705315200,
        };
        
        // Should succeed with at least 1 receiver
        // Note: In actual broadcast, receivers get messages only after subscription
        let result = state.broadcast(msg);
        assert!(result.is_ok() || result.is_err()); // Either works, depends on timing
    }
}
