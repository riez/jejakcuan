//! Audit event definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Audit event categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventCategory {
    /// User authentication events
    Authentication,
    /// User authorization/access events  
    Authorization,
    /// Data access events
    DataAccess,
    /// Data modification events
    DataModification,
    /// System configuration changes
    SystemConfig,
    /// Security-related events
    Security,
    /// API access events
    ApiAccess,
    /// Data export events (PDP compliance)
    DataExport,
    /// User consent events (PDP compliance)
    Consent,
}

/// Audit event severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Severity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error or failure
    Error,
    /// Security critical
    Critical,
}

/// Audit event outcome
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Outcome {
    Success,
    Failure,
    Partial,
}

/// Main audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: Uuid,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event category
    pub category: EventCategory,
    /// Severity level
    pub severity: Severity,
    /// Event outcome
    pub outcome: Outcome,
    /// Actor (user) information
    pub actor: ActorInfo,
    /// Action performed
    pub action: String,
    /// Resource affected
    pub resource: ResourceInfo,
    /// Event details/context
    pub details: serde_json::Value,
    /// Client information
    pub client: ClientInfo,
}

/// Information about the actor (user) who performed the action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorInfo {
    /// User ID (if authenticated)
    pub user_id: Option<String>,
    /// Username
    pub username: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Actor type
    pub actor_type: ActorType,
}

/// Type of actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorType {
    User,
    System,
    Api,
    Scheduler,
    Anonymous,
}

/// Information about the resource being accessed/modified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// Resource type (e.g., "stock", "user", "alert")
    pub resource_type: String,
    /// Resource ID
    pub resource_id: Option<String>,
    /// Resource path/location
    pub path: Option<String>,
}

/// Client information for request tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        category: EventCategory,
        severity: Severity,
        action: &str,
        resource_type: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category,
            severity,
            outcome: Outcome::Success,
            actor: ActorInfo {
                user_id: None,
                username: None,
                session_id: None,
                actor_type: ActorType::Anonymous,
            },
            action: action.to_string(),
            resource: ResourceInfo {
                resource_type: resource_type.to_string(),
                resource_id: None,
                path: None,
            },
            details: serde_json::json!({}),
            client: ClientInfo {
                ip_address: None,
                user_agent: None,
                request_id: None,
            },
        }
    }

    pub fn with_user(mut self, user_id: &str, username: &str) -> Self {
        self.actor.user_id = Some(user_id.to_string());
        self.actor.username = Some(username.to_string());
        self.actor.actor_type = ActorType::User;
        self
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.actor.session_id = Some(session_id.to_string());
        self
    }

    pub fn with_resource_id(mut self, id: &str) -> Self {
        self.resource.resource_id = Some(id.to_string());
        self
    }

    pub fn with_path(mut self, path: &str) -> Self {
        self.resource.path = Some(path.to_string());
        self
    }

    pub fn with_outcome(mut self, outcome: Outcome) -> Self {
        self.outcome = outcome;
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }

    pub fn with_client(mut self, ip: Option<&str>, user_agent: Option<&str>) -> Self {
        self.client.ip_address = ip.map(String::from);
        self.client.user_agent = user_agent.map(String::from);
        self
    }

    pub fn with_request_id(mut self, request_id: &str) -> Self {
        self.client.request_id = Some(request_id.to_string());
        self
    }
}

/// Common audit event builders
pub mod events {
    use super::*;

    /// User login event
    pub fn login(username: &str, success: bool) -> AuditEvent {
        AuditEvent::new(
            EventCategory::Authentication,
            if success {
                Severity::Info
            } else {
                Severity::Warning
            },
            "user_login",
            "session",
        )
        .with_outcome(if success {
            Outcome::Success
        } else {
            Outcome::Failure
        })
        .with_details(serde_json::json!({ "username": username }))
    }

    /// User logout event
    pub fn logout(user_id: &str, username: &str) -> AuditEvent {
        AuditEvent::new(
            EventCategory::Authentication,
            Severity::Info,
            "user_logout",
            "session",
        )
        .with_user(user_id, username)
    }

    /// Data access event
    pub fn data_access(resource_type: &str, resource_id: &str) -> AuditEvent {
        AuditEvent::new(
            EventCategory::DataAccess,
            Severity::Info,
            "data_read",
            resource_type,
        )
        .with_resource_id(resource_id)
    }

    /// Data export event (PDP compliance)
    pub fn data_export(user_id: &str, export_type: &str) -> AuditEvent {
        AuditEvent::new(
            EventCategory::DataExport,
            Severity::Info,
            "data_export",
            "export",
        )
        .with_details(serde_json::json!({ "export_type": export_type, "user_id": user_id }))
    }

    /// API access event
    pub fn api_access(endpoint: &str, method: &str) -> AuditEvent {
        AuditEvent::new(
            EventCategory::ApiAccess,
            Severity::Info,
            "api_request",
            "endpoint",
        )
        .with_path(endpoint)
        .with_details(serde_json::json!({ "method": method }))
    }

    /// Security alert event
    pub fn security_alert(alert_type: &str, details: &str) -> AuditEvent {
        AuditEvent::new(
            EventCategory::Security,
            Severity::Critical,
            alert_type,
            "security",
        )
        .with_details(serde_json::json!({ "details": details }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            EventCategory::Authentication,
            Severity::Info,
            "test_action",
            "test_resource",
        );

        assert_eq!(event.action, "test_action");
        assert_eq!(event.resource.resource_type, "test_resource");
    }

    #[test]
    fn test_event_builder() {
        let event =
            events::login("testuser", true).with_client(Some("192.168.1.1"), Some("TestAgent"));

        assert_eq!(event.client.ip_address, Some("192.168.1.1".to_string()));
        assert!(matches!(event.outcome, Outcome::Success));
    }

    #[test]
    fn test_serialization() {
        let event = events::api_access("/api/stocks", "GET");
        let json = serde_json::to_string(&event).unwrap();

        assert!(json.contains("ApiAccess"));
        assert!(json.contains("/api/stocks"));
    }
}
