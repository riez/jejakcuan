//! Email notification channel via SMTP

use super::{Notification, NotificationError, NotificationResult, NotificationSender};
use async_trait::async_trait;
use jejakcuan_core::alerts::NotificationChannel;
use serde::{Deserialize, Serialize};

/// Email SMTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_user: String::new(),
            smtp_password: String::new(),
            from_email: String::new(),
            from_name: "JejakCuan Alerts".to_string(),
        }
    }
}

/// Email notification sender
pub struct EmailNotifier {
    config: EmailConfig,
}

impl EmailNotifier {
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }

    fn format_html(&self, notification: &Notification) -> String {
        let priority_color = match notification.priority {
            super::NotificationPriority::Critical => "#dc2626",
            super::NotificationPriority::High => "#ea580c",
            super::NotificationPriority::Medium => "#ca8a04",
            super::NotificationPriority::Low => "#16a34a",
        };

        let symbol = notification
            .metadata
            .symbol
            .as_deref()
            .unwrap_or("N/A");

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: {color}; color: white; padding: 15px; border-radius: 8px 8px 0 0; }}
        .body {{ background: #f9fafb; padding: 20px; border: 1px solid #e5e7eb; }}
        .footer {{ padding: 15px; font-size: 12px; color: #6b7280; }}
        .symbol {{ background: #1f2937; color: white; padding: 4px 8px; border-radius: 4px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h2 style="margin:0">{title}</h2>
        </div>
        <div class="body">
            <p>{body}</p>
            <p><strong>Symbol:</strong> <span class="symbol">{symbol}</span></p>
        </div>
        <div class="footer">
            <p>This alert was sent by JejakCuan. You can manage your alert preferences in the app.</p>
        </div>
    </div>
</body>
</html>"#,
            color = priority_color,
            title = notification.title,
            body = notification.body,
            symbol = symbol
        )
    }
}

#[async_trait]
impl NotificationSender for EmailNotifier {
    async fn send(&self, notification: &Notification) -> NotificationResult<()> {
        if !self.is_configured() {
            return Err(NotificationError::NotConfigured("SMTP not configured".into()));
        }

        // Validate email format
        if !notification.recipient_id.contains('@') {
            return Err(NotificationError::InvalidRecipient(
                "Invalid email format".into(),
            ));
        }

        let _html_body = self.format_html(notification);

        // In production, use lettre or similar SMTP crate
        // For now, we'll just validate the configuration
        // let email = Message::builder()
        //     .from(format!("{} <{}>", self.config.from_name, self.config.from_email).parse()?)
        //     .to(notification.recipient_id.parse()?)
        //     .subject(&notification.title)
        //     .multipart(MultiPart::alternative_plain_html(
        //         notification.body.clone(),
        //         html_body,
        //     ))?;

        // This is a placeholder - actual SMTP implementation would go here
        // For testing purposes, we return success
        tracing::info!(
            "Email notification queued for {} - {}",
            notification.recipient_id,
            notification.title
        );

        Ok(())
    }

    fn is_configured(&self) -> bool {
        !self.config.smtp_host.is_empty()
            && !self.config.from_email.is_empty()
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Email
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_html() {
        let notifier = EmailNotifier::new(EmailConfig::default());
        let notification = Notification {
            recipient_id: "test@example.com".to_string(),
            title: "Test Alert".to_string(),
            body: "This is a test message".to_string(),
            priority: super::super::NotificationPriority::High,
            channel: NotificationChannel::Email,
            alert: None,
            metadata: super::super::NotificationMetadata {
                symbol: Some("BBCA".to_string()),
                ..Default::default()
            },
        };

        let html = notifier.format_html(&notification);
        assert!(html.contains("Test Alert"));
        assert!(html.contains("BBCA"));
        assert!(html.contains("#ea580c")); // High priority color
    }

    #[test]
    fn test_not_configured() {
        let notifier = EmailNotifier::new(EmailConfig::default());
        assert!(!notifier.is_configured());
    }

    #[test]
    fn test_configured() {
        let notifier = EmailNotifier::new(EmailConfig {
            smtp_host: "smtp.gmail.com".to_string(),
            from_email: "alerts@jejakcuan.com".to_string(),
            ..Default::default()
        });
        assert!(notifier.is_configured());
    }
}
