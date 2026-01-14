//! Audit logger implementation

use crate::{AuditEvent, EventCategory, Severity};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Audit logger configuration
#[derive(Debug, Clone)]
pub struct AuditLoggerConfig {
    /// Database connection pool
    pub buffer_size: usize,
    /// Whether to log to console
    pub console_logging: bool,
    /// Minimum severity to log
    pub min_severity: Severity,
}

impl Default for AuditLoggerConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            console_logging: true,
            min_severity: Severity::Info,
        }
    }
}

/// Audit logger service
pub struct AuditLogger {
    config: AuditLoggerConfig,
    tx: mpsc::Sender<AuditEvent>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(config: AuditLoggerConfig, pool: PgPool) -> Self {
        let (tx, rx) = mpsc::channel(config.buffer_size);
        let console_logging = config.console_logging;

        // Spawn background task to process audit events
        tokio::spawn(async move {
            Self::process_events(rx, pool, console_logging).await;
        });

        Self { config, tx }
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) {
        // Apply severity filter
        if !self.should_log(&event) {
            return;
        }

        if let Err(e) = self.tx.send(event).await {
            error!("Failed to queue audit event: {}", e);
        }
    }

    /// Log an audit event (blocking version for sync contexts)
    pub fn log_sync(&self, event: AuditEvent) {
        if !self.should_log(&event) {
            return;
        }

        if let Err(e) = self.tx.try_send(event) {
            error!("Failed to queue audit event: {}", e);
        }
    }

    fn should_log(&self, event: &AuditEvent) -> bool {
        let event_level = match event.severity {
            Severity::Info => 0,
            Severity::Warning => 1,
            Severity::Error => 2,
            Severity::Critical => 3,
        };

        let min_level = match self.config.min_severity {
            Severity::Info => 0,
            Severity::Warning => 1,
            Severity::Error => 2,
            Severity::Critical => 3,
        };

        event_level >= min_level
    }

    async fn process_events(
        mut rx: mpsc::Receiver<AuditEvent>,
        pool: PgPool,
        console_logging: bool,
    ) {
        while let Some(event) = rx.recv().await {
            // Log to console if enabled
            if console_logging {
                Self::log_to_console(&event);
            }

            // Store in database
            if let Err(e) = Self::store_event(&pool, &event).await {
                error!("Failed to store audit event: {}", e);
            }
        }
    }

    fn log_to_console(event: &AuditEvent) {
        let msg = format!(
            "[AUDIT] {} | {:?} | {} | {} | {:?}",
            event.timestamp.format("%Y-%m-%d %H:%M:%S"),
            event.category,
            event.action,
            event.resource.resource_type,
            event.outcome
        );

        match event.severity {
            Severity::Info => info!("{}", msg),
            Severity::Warning => warn!("{}", msg),
            Severity::Error => error!("{}", msg),
            Severity::Critical => error!("[CRITICAL] {}", msg),
        }
    }

    async fn store_event(pool: &PgPool, event: &AuditEvent) -> Result<(), sqlx::Error> {
        let category = serde_json::to_string(&event.category).unwrap_or_default();
        let severity = serde_json::to_string(&event.severity).unwrap_or_default();
        let outcome = serde_json::to_string(&event.outcome).unwrap_or_default();
        let actor = serde_json::to_value(&event.actor).unwrap_or_default();
        let resource = serde_json::to_value(&event.resource).unwrap_or_default();
        let client = serde_json::to_value(&event.client).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                id, timestamp, category, severity, outcome,
                actor, action, resource, details, client
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(event.id)
        .bind(event.timestamp)
        .bind(&category)
        .bind(&severity)
        .bind(&outcome)
        .bind(&actor)
        .bind(&event.action)
        .bind(&resource)
        .bind(&event.details)
        .bind(&client)
        .execute(pool)
        .await?;

        Ok(())
    }
}

/// Audit middleware trait for framework integration
#[async_trait::async_trait]
pub trait AuditMiddleware: Send + Sync {
    async fn before_request(&self, event: &mut AuditEvent);
    async fn after_request(&self, event: &mut AuditEvent);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::events as audit_events;

    #[test]
    fn test_config_default() {
        let config = AuditLoggerConfig::default();
        assert!(config.console_logging);
        assert_eq!(config.buffer_size, 1000);
    }

    #[test]
    fn test_severity_filter() {
        let event = audit_events::api_access("/test", "GET");
        let level = match event.severity {
            Severity::Info => 0,
            Severity::Warning => 1,
            Severity::Error => 2,
            Severity::Critical => 3,
        };
        assert_eq!(level, 0);
    }
}
