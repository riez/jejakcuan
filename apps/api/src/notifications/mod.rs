//! Notification channels for JejakCuan alerts
//!
//! Supports multiple notification channels:
//! - Telegram bot
//! - Email (SMTP)
//! - Webhook
//! - Web push notifications
//! - In-app notifications via WebSocket/SSE

mod email;
mod telegram;
mod webhook;

pub use email::*;
pub use telegram::*;
pub use webhook::*;

use async_trait::async_trait;
use jejakcuan_core::alerts::{Alert, NotificationChannel};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Result type for notification operations
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Notification errors
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Failed to send notification: {0}")]
    SendFailed(String),
    #[error("Channel not configured: {0}")]
    NotConfigured(String),
    #[error("Rate limited: retry after {0} seconds")]
    RateLimited(u64),
    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Trait for notification channel implementations
#[async_trait]
pub trait NotificationSender: Send + Sync {
    /// Send a notification
    async fn send(&self, notification: &Notification) -> NotificationResult<()>;

    /// Check if channel is configured and ready
    fn is_configured(&self) -> bool;

    /// Get channel type
    fn channel_type(&self) -> NotificationChannel;
}

/// Notification to send
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub recipient_id: String,
    pub title: String,
    pub body: String,
    pub priority: NotificationPriority,
    pub channel: NotificationChannel,
    pub alert: Option<Alert>,
    pub metadata: NotificationMetadata,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl From<jejakcuan_core::alerts::AlertPriority> for NotificationPriority {
    fn from(priority: jejakcuan_core::alerts::AlertPriority) -> Self {
        match priority {
            jejakcuan_core::alerts::AlertPriority::Critical => NotificationPriority::Critical,
            jejakcuan_core::alerts::AlertPriority::High => NotificationPriority::High,
            jejakcuan_core::alerts::AlertPriority::Medium => NotificationPriority::Medium,
            jejakcuan_core::alerts::AlertPriority::Low => NotificationPriority::Low,
        }
    }
}

/// Additional metadata for notifications
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationMetadata {
    pub symbol: Option<String>,
    pub alert_id: Option<String>,
    pub action_url: Option<String>,
    pub icon: Option<String>,
}

/// Notification service that routes to appropriate channels
pub struct NotificationService {
    telegram: Option<Arc<TelegramNotifier>>,
    email: Option<Arc<EmailNotifier>>,
    webhook: Option<Arc<WebhookNotifier>>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            telegram: None,
            email: None,
            webhook: None,
        }
    }

    pub fn with_telegram(mut self, notifier: TelegramNotifier) -> Self {
        self.telegram = Some(Arc::new(notifier));
        self
    }

    pub fn with_email(mut self, notifier: EmailNotifier) -> Self {
        self.email = Some(Arc::new(notifier));
        self
    }

    pub fn with_webhook(mut self, notifier: WebhookNotifier) -> Self {
        self.webhook = Some(Arc::new(notifier));
        self
    }

    /// Send notification via specified channel
    pub async fn send(&self, notification: &Notification) -> NotificationResult<()> {
        match notification.channel {
            NotificationChannel::Telegram => {
                if let Some(ref sender) = self.telegram {
                    sender.send(notification).await
                } else {
                    Err(NotificationError::NotConfigured("Telegram".into()))
                }
            }
            NotificationChannel::Email => {
                if let Some(ref sender) = self.email {
                    sender.send(notification).await
                } else {
                    Err(NotificationError::NotConfigured("Email".into()))
                }
            }
            NotificationChannel::Webhook => {
                if let Some(ref sender) = self.webhook {
                    sender.send(notification).await
                } else {
                    Err(NotificationError::NotConfigured("Webhook".into()))
                }
            }
            NotificationChannel::WebPush => {
                // WebPush would require additional setup
                Err(NotificationError::NotConfigured("WebPush".into()))
            }
            NotificationChannel::InApp => {
                // In-app handled separately via SSE/WebSocket
                Ok(())
            }
        }
    }

    /// Send notification to all configured channels for a user
    pub async fn broadcast(
        &self,
        notification: &Notification,
        channels: &[NotificationChannel],
    ) -> Vec<(NotificationChannel, NotificationResult<()>)> {
        let mut results = Vec::new();

        for channel in channels {
            let mut notif = notification.clone();
            notif.channel = channel.clone();
            let result = self.send(&notif).await;
            results.push((channel.clone(), result));
        }

        results
    }

    /// Create notification from alert
    pub fn notification_from_alert(
        alert: &Alert,
        recipient_id: String,
        channel: NotificationChannel,
    ) -> Notification {
        Notification {
            recipient_id,
            title: format!("{} Alert", alert.symbol()),
            body: alert.message().to_string(),
            priority: alert.priority().into(),
            channel,
            alert: Some(alert.clone()),
            metadata: NotificationMetadata {
                symbol: Some(alert.symbol().to_string()),
                alert_id: Some(alert.id().to_string()),
                action_url: Some(format!("/stocks/{}", alert.symbol())),
                icon: None,
            },
        }
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jejakcuan_core::alerts::{AlertPriority, BrokerAlert, BrokerAlertType};
    use rust_decimal_macros::dec;

    #[test]
    fn test_notification_from_alert() {
        let alert = Alert::Broker(BrokerAlert::new(
            "BBCA".to_string(),
            BrokerAlertType::CoordinatedBuying {
                broker_count: 3,
                broker_codes: vec!["BK".into(), "CC".into(), "KZ".into()],
            },
            AlertPriority::High,
            dec!(3),
            dec!(3),
        ));

        let notification = NotificationService::notification_from_alert(
            &alert,
            "user123".to_string(),
            NotificationChannel::Telegram,
        );

        assert_eq!(notification.recipient_id, "user123");
        assert!(notification.title.contains("BBCA"));
        assert_eq!(notification.priority, NotificationPriority::High);
    }

    #[test]
    fn test_priority_conversion() {
        assert_eq!(
            NotificationPriority::from(AlertPriority::Critical),
            NotificationPriority::Critical
        );
        assert_eq!(
            NotificationPriority::from(AlertPriority::High),
            NotificationPriority::High
        );
    }
}
