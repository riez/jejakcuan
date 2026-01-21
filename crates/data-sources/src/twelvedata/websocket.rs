//! TwelveData WebSocket client for real-time price streaming
//!
//! Features:
//! - Auto-reconnection with exponential backoff
//! - Subscription management
//! - Backpressure handling

use super::models::{PriceUpdate, SubscribeAction, WebSocketMessage};
use crate::error::DataSourceError;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use url::Url;

const WS_URL: &str = "wss://ws.twelvedata.com/v1/quotes/price";
const RECONNECT_DELAY_MS: u64 = 1000;
const MAX_RECONNECT_DELAY_MS: u64 = 30000;
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

/// Events emitted by the WebSocket client
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    Connected,
    Disconnected,
    Price(PriceUpdate),
    Subscribed(Vec<String>),
    Unsubscribed(Vec<String>),
    Error(String),
}

/// TwelveData WebSocket client for real-time streaming
pub struct TwelveDataWebSocket {
    api_key: String,
    subscriptions: Arc<RwLock<HashSet<String>>>,
    event_tx: mpsc::Sender<WebSocketEvent>,
    event_rx: Arc<Mutex<mpsc::Receiver<WebSocketEvent>>>,
    running: Arc<RwLock<bool>>,
    command_tx: Option<mpsc::Sender<WebSocketCommand>>,
}

#[derive(Debug)]
enum WebSocketCommand {
    Subscribe(Vec<String>),
    Unsubscribe(Vec<String>),
    Disconnect,
}

impl TwelveDataWebSocket {
    /// Create a new WebSocket client
    pub fn new(api_key: String) -> Self {
        let (event_tx, event_rx) = mpsc::channel(1000);

        Self {
            api_key,
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            running: Arc::new(RwLock::new(false)),
            command_tx: None,
        }
    }

    /// Create from environment variable
    pub fn from_env() -> Result<Self, DataSourceError> {
        let api_key = std::env::var("TWELVEDATA_API_KEY")
            .map_err(|_| DataSourceError::InvalidResponse("TWELVEDATA_API_KEY not set".into()))?;
        Ok(Self::new(api_key))
    }

    /// Connect and start receiving events
    pub async fn connect(&mut self) -> Result<(), DataSourceError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        let (command_tx, command_rx) = mpsc::channel(100);
        self.command_tx = Some(command_tx);

        let api_key = self.api_key.clone();
        let event_tx = self.event_tx.clone();
        let subscriptions = self.subscriptions.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            Self::connection_loop(api_key, event_tx, subscriptions, running, command_rx).await;
        });

        Ok(())
    }

    /// Main connection loop with auto-reconnection
    async fn connection_loop(
        api_key: String,
        event_tx: mpsc::Sender<WebSocketEvent>,
        subscriptions: Arc<RwLock<HashSet<String>>>,
        running: Arc<RwLock<bool>>,
        mut command_rx: mpsc::Receiver<WebSocketCommand>,
    ) {
        let mut reconnect_delay = RECONNECT_DELAY_MS;

        loop {
            if !*running.read().await {
                break;
            }

            let ws_url = format!("{}?apikey={}", WS_URL, api_key);
            let url = match Url::parse(&ws_url) {
                Ok(u) => u,
                Err(e) => {
                    error!("Invalid WebSocket URL: {}", e);
                    break;
                }
            };

            match connect_async(url).await {
                Ok((ws_stream, _)) => {
                    info!("Connected to TwelveData WebSocket");
                    reconnect_delay = RECONNECT_DELAY_MS;

                    let _ = event_tx.send(WebSocketEvent::Connected).await;

                    let (mut write, mut read) = ws_stream.split();

                    // Resubscribe to existing symbols
                    let subs = subscriptions.read().await;
                    if !subs.is_empty() {
                        let symbols: Vec<String> = subs.iter().cloned().collect();
                        let action = SubscribeAction::subscribe(symbols);
                        if let Ok(msg) = serde_json::to_string(&action) {
                            let _ = write.send(Message::Text(msg)).await;
                        }
                    }
                    drop(subs);

                    // Message handling loop
                    loop {
                        tokio::select! {
                            // Handle incoming WebSocket messages
                            msg = read.next() => {
                                match msg {
                                    Some(Ok(Message::Text(text))) => {
                                        if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                                            match ws_msg {
                                                WebSocketMessage::Price(price) => {
                                                    let _ = event_tx.send(WebSocketEvent::Price(price)).await;
                                                }
                                                WebSocketMessage::SubscribeStatus { success, .. } => {
                                                    let symbols: Vec<String> = success.iter().map(|s| s.symbol.clone()).collect();
                                                    if !symbols.is_empty() {
                                                        let _ = event_tx.send(WebSocketEvent::Subscribed(symbols)).await;
                                                    }
                                                }
                                                WebSocketMessage::UnsubscribeStatus { .. } => {
                                                    debug!("Unsubscribe confirmed");
                                                }
                                                WebSocketMessage::Heartbeat => {
                                                    debug!("Heartbeat received");
                                                }
                                                WebSocketMessage::Unknown => {
                                                    debug!("Unknown message: {}", text);
                                                }
                                            }
                                        }
                                    }
                                    Some(Ok(Message::Ping(data))) => {
                                        let _ = write.send(Message::Pong(data)).await;
                                    }
                                    Some(Ok(Message::Close(_))) => {
                                        info!("WebSocket closed by server");
                                        break;
                                    }
                                    Some(Err(e)) => {
                                        error!("WebSocket error: {}", e);
                                        let _ = event_tx.send(WebSocketEvent::Error(e.to_string())).await;
                                        break;
                                    }
                                    None => {
                                        info!("WebSocket stream ended");
                                        break;
                                    }
                                    _ => {}
                                }
                            }

                            // Handle commands
                            cmd = command_rx.recv() => {
                                match cmd {
                                    Some(WebSocketCommand::Subscribe(symbols)) => {
                                        let mut subs = subscriptions.write().await;
                                        for s in &symbols {
                                            subs.insert(s.clone());
                                        }
                                        drop(subs);

                                        let action = SubscribeAction::subscribe(symbols);
                                        if let Ok(msg) = serde_json::to_string(&action) {
                                            let _ = write.send(Message::Text(msg)).await;
                                        }
                                    }
                                    Some(WebSocketCommand::Unsubscribe(symbols)) => {
                                        let mut subs = subscriptions.write().await;
                                        for s in &symbols {
                                            subs.remove(s);
                                        }
                                        drop(subs);

                                        let action = SubscribeAction::unsubscribe(symbols);
                                        if let Ok(msg) = serde_json::to_string(&action) {
                                            let _ = write.send(Message::Text(msg)).await;
                                        }
                                    }
                                    Some(WebSocketCommand::Disconnect) => {
                                        let _ = write.send(Message::Close(None)).await;
                                        *running.write().await = false;
                                        break;
                                    }
                                    None => {
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    let _ = event_tx.send(WebSocketEvent::Disconnected).await;
                }
                Err(e) => {
                    error!("Failed to connect to WebSocket: {}", e);
                    let _ = event_tx.send(WebSocketEvent::Error(e.to_string())).await;
                }
            }

            // Reconnect with exponential backoff
            if *running.read().await {
                warn!("Reconnecting in {}ms...", reconnect_delay);
                tokio::time::sleep(Duration::from_millis(reconnect_delay)).await;
                reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY_MS);
            }
        }
    }

    /// Subscribe to symbols
    pub async fn subscribe(&self, symbols: Vec<String>) -> Result<(), DataSourceError> {
        if let Some(tx) = &self.command_tx {
            tx.send(WebSocketCommand::Subscribe(symbols))
                .await
                .map_err(|_| {
                    DataSourceError::ApiError("Failed to send subscribe command".into())
                })?;
        }
        Ok(())
    }

    /// Unsubscribe from symbols
    pub async fn unsubscribe(&self, symbols: Vec<String>) -> Result<(), DataSourceError> {
        if let Some(tx) = &self.command_tx {
            tx.send(WebSocketCommand::Unsubscribe(symbols))
                .await
                .map_err(|_| {
                    DataSourceError::ApiError("Failed to send unsubscribe command".into())
                })?;
        }
        Ok(())
    }

    /// Disconnect from WebSocket
    pub async fn disconnect(&self) -> Result<(), DataSourceError> {
        *self.running.write().await = false;
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(WebSocketCommand::Disconnect).await;
        }
        Ok(())
    }

    /// Receive next event
    pub async fn recv(&self) -> Option<WebSocketEvent> {
        let mut rx = self.event_rx.lock().await;
        rx.recv().await
    }

    /// Get current subscriptions
    pub async fn subscriptions(&self) -> Vec<String> {
        self.subscriptions.read().await.iter().cloned().collect()
    }

    /// Check if connected
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_creation() {
        let ws = TwelveDataWebSocket::new("test_key".to_string());
        assert!(!ws.api_key.is_empty());
    }

    #[test]
    fn test_subscribe_action() {
        let action = SubscribeAction::subscribe(vec!["AAPL".to_string(), "MSFT".to_string()]);
        assert_eq!(action.action, "subscribe");
        assert_eq!(action.params.symbols.len(), 2);
    }

    #[test]
    fn test_unsubscribe_action() {
        let action = SubscribeAction::unsubscribe(vec!["AAPL".to_string()]);
        assert_eq!(action.action, "unsubscribe");
        assert_eq!(action.params.symbols.len(), 1);
    }
}
