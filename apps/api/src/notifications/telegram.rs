//! Telegram notification channel

use super::{Notification, NotificationError, NotificationResult, NotificationSender};
use async_trait::async_trait;
use jejakcuan_core::alerts::NotificationChannel;
use serde::{Deserialize, Serialize};

/// Telegram bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub api_url: String,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            api_url: "https://api.telegram.org".to_string(),
        }
    }
}

/// Telegram notification sender
pub struct TelegramNotifier {
    config: TelegramConfig,
    client: reqwest::Client,
}

impl TelegramNotifier {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn format_message(&self, notification: &Notification) -> String {
        let priority_emoji = match notification.priority {
            super::NotificationPriority::Critical => "🚨",
            super::NotificationPriority::High => "🔴",
            super::NotificationPriority::Medium => "🟡",
            super::NotificationPriority::Low => "🟢",
        };

        let symbol = notification.metadata.symbol.as_deref().unwrap_or("Unknown");

        format!(
            "{} *{}*\n\n{}\n\n📊 Symbol: `{}`",
            priority_emoji, notification.title, notification.body, symbol
        )
    }
}

#[async_trait]
impl NotificationSender for TelegramNotifier {
    async fn send(&self, notification: &Notification) -> NotificationResult<()> {
        if !self.is_configured() {
            return Err(NotificationError::NotConfigured(
                "Telegram bot token missing".into(),
            ));
        }

        let message = self.format_message(notification);
        let url = format!(
            "{}/bot{}/sendMessage",
            self.config.api_url, self.config.bot_token
        );

        let payload = serde_json::json!({
            "chat_id": notification.recipient_id,
            "text": message,
            "parse_mode": "Markdown"
        });

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else if response.status().as_u16() == 429 {
            // Rate limited
            Err(NotificationError::RateLimited(30))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(NotificationError::SendFailed(error_text))
        }
    }

    fn is_configured(&self) -> bool {
        !self.config.bot_token.is_empty()
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Telegram
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_message() {
        let notifier = TelegramNotifier::new(TelegramConfig::default());
        let notification = Notification {
            recipient_id: "123".to_string(),
            title: "Test Alert".to_string(),
            body: "This is a test".to_string(),
            priority: super::super::NotificationPriority::High,
            channel: NotificationChannel::Telegram,
            alert: None,
            metadata: super::super::NotificationMetadata {
                symbol: Some("BBCA".to_string()),
                ..Default::default()
            },
        };

        let message = notifier.format_message(&notification);
        assert!(message.contains("🔴"));
        assert!(message.contains("Test Alert"));
        assert!(message.contains("BBCA"));
    }

    #[test]
    fn test_not_configured() {
        let notifier = TelegramNotifier::new(TelegramConfig::default());
        assert!(!notifier.is_configured());
    }

    #[test]
    fn test_configured() {
        let notifier = TelegramNotifier::new(TelegramConfig {
            bot_token: "test_token".to_string(),
            ..Default::default()
        });
        assert!(notifier.is_configured());
    }
}
