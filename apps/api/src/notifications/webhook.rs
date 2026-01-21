//! Webhook notification channel

use super::{Notification, NotificationError, NotificationResult, NotificationSender};
use async_trait::async_trait;
use chrono::Utc;
use jejakcuan_core::alerts::NotificationChannel;
use serde::{Deserialize, Serialize};

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub secret_header: Option<String>,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_retries: 3,
            secret_header: None,
        }
    }
}

/// Webhook payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub timestamp: String,
    pub data: WebhookData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookData {
    pub alert_id: Option<String>,
    pub symbol: Option<String>,
    pub title: String,
    pub body: String,
    pub priority: String,
    pub action_url: Option<String>,
}

/// Webhook notification sender
pub struct WebhookNotifier {
    config: WebhookConfig,
    client: reqwest::Client,
}

impl WebhookNotifier {
    pub fn new(config: WebhookConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    fn create_payload(&self, notification: &Notification) -> WebhookPayload {
        WebhookPayload {
            event_type: "alert.triggered".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: WebhookData {
                alert_id: notification.metadata.alert_id.clone(),
                symbol: notification.metadata.symbol.clone(),
                title: notification.title.clone(),
                body: notification.body.clone(),
                priority: format!("{:?}", notification.priority).to_lowercase(),
                action_url: notification.metadata.action_url.clone(),
            },
        }
    }

    fn compute_signature(&self, payload: &str) -> Option<String> {
        self.config.secret_header.as_ref().map(|secret| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            payload.hash(&mut hasher);
            secret.hash(&mut hasher);
            format!("sha256={:x}", hasher.finish())
        })
    }
}

#[async_trait]
impl NotificationSender for WebhookNotifier {
    async fn send(&self, notification: &Notification) -> NotificationResult<()> {
        // recipient_id is the webhook URL for this channel
        let webhook_url = &notification.recipient_id;

        if !webhook_url.starts_with("http://") && !webhook_url.starts_with("https://") {
            return Err(NotificationError::InvalidRecipient(
                "Webhook URL must start with http:// or https://".into(),
            ));
        }

        let payload = self.create_payload(notification);
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| NotificationError::SendFailed(e.to_string()))?;

        let mut request = self
            .client
            .post(webhook_url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "JejakCuan-Webhook/1.0");

        // Add signature header if secret is configured
        if let Some(signature) = self.compute_signature(&payload_json) {
            request = request.header("X-JejakCuan-Signature", signature);
        }

        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            }

            match request
                .try_clone()
                .ok_or_else(|| NotificationError::SendFailed("Failed to clone request".into()))?
                .body(payload_json.clone())
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(());
                    } else if response.status().as_u16() == 429 {
                        return Err(NotificationError::RateLimited(60));
                    } else {
                        last_error = Some(NotificationError::SendFailed(format!(
                            "HTTP {}",
                            response.status()
                        )));
                    }
                }
                Err(e) => {
                    last_error = Some(NotificationError::NetworkError(e.to_string()));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| NotificationError::SendFailed("Unknown error".into())))
    }

    fn is_configured(&self) -> bool {
        // Webhook doesn't require static configuration - URLs are per-notification
        true
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Webhook
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_payload() {
        let notifier = WebhookNotifier::new(WebhookConfig::default());
        let notification = Notification {
            recipient_id: "https://example.com/webhook".to_string(),
            title: "Test Alert".to_string(),
            body: "Test body".to_string(),
            priority: super::super::NotificationPriority::High,
            channel: NotificationChannel::Webhook,
            alert: None,
            metadata: super::super::NotificationMetadata {
                symbol: Some("BBCA".to_string()),
                alert_id: Some("alert_123".to_string()),
                ..Default::default()
            },
        };

        let payload = notifier.create_payload(&notification);
        assert_eq!(payload.event_type, "alert.triggered");
        assert_eq!(payload.data.symbol, Some("BBCA".to_string()));
        assert_eq!(payload.data.title, "Test Alert");
    }

    #[test]
    fn test_signature_computation() {
        let notifier = WebhookNotifier::new(WebhookConfig {
            secret_header: Some("my_secret".to_string()),
            ..Default::default()
        });

        let signature = notifier.compute_signature("test payload");
        assert!(signature.is_some());
        assert!(signature.unwrap().starts_with("sha256="));
    }

    #[test]
    fn test_no_signature_without_secret() {
        let notifier = WebhookNotifier::new(WebhookConfig::default());
        let signature = notifier.compute_signature("test payload");
        assert!(signature.is_none());
    }
}
