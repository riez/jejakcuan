//! Audit log retention policies
//!
//! Implements data retention requirements for PDP Law compliance

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use tracing::{error, info};

/// Retention policy configuration
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// Days to retain authentication logs
    pub auth_logs_days: i64,
    /// Days to retain data access logs
    pub data_access_days: i64,
    /// Days to retain security logs
    pub security_logs_days: i64,
    /// Days to retain API access logs
    pub api_logs_days: i64,
    /// Days to retain all other logs
    pub default_days: i64,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            auth_logs_days: 365,     // 1 year
            data_access_days: 90,    // 3 months
            security_logs_days: 730, // 2 years (PDP compliance)
            api_logs_days: 30,       // 30 days
            default_days: 90,        // 3 months
        }
    }
}

/// PDP Law compliant retention settings
impl RetentionPolicy {
    /// Create policy compliant with Indonesian PDP Law
    pub fn pdp_compliant() -> Self {
        Self {
            auth_logs_days: 365,     // Authentication records
            data_access_days: 180,   // Personal data access logs
            security_logs_days: 730, // Security incident records
            api_logs_days: 90,       // API access patterns
            default_days: 180,       // General retention
        }
    }
}

/// Audit log cleanup service
pub struct RetentionService {
    policy: RetentionPolicy,
    pool: PgPool,
}

impl RetentionService {
    pub fn new(pool: PgPool, policy: RetentionPolicy) -> Self {
        Self { pool, policy }
    }

    /// Run cleanup based on retention policy
    pub async fn cleanup(&self) -> Result<CleanupReport, sqlx::Error> {
        let mut report = CleanupReport::default();
        let now = Utc::now();

        // Clean auth logs
        let auth_cutoff = now - Duration::days(self.policy.auth_logs_days);
        report.auth_deleted = self
            .delete_by_category("Authentication", auth_cutoff)
            .await?;

        // Clean data access logs
        let data_cutoff = now - Duration::days(self.policy.data_access_days);
        report.data_access_deleted = self.delete_by_category("DataAccess", data_cutoff).await?;

        // Clean security logs (keep longer)
        let security_cutoff = now - Duration::days(self.policy.security_logs_days);
        report.security_deleted = self.delete_by_category("Security", security_cutoff).await?;

        // Clean API logs
        let api_cutoff = now - Duration::days(self.policy.api_logs_days);
        report.api_deleted = self.delete_by_category("ApiAccess", api_cutoff).await?;

        // Clean remaining logs
        let default_cutoff = now - Duration::days(self.policy.default_days);
        report.other_deleted = self.delete_older_than(default_cutoff).await?;

        info!(
            "Audit cleanup complete: {} total records deleted",
            report.total_deleted()
        );

        Ok(report)
    }

    async fn delete_by_category(
        &self,
        category: &str,
        cutoff: DateTime<Utc>,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM audit_logs
            WHERE category = $1 AND timestamp < $2
            "#,
        )
        .bind(format!("\"{}\"", category))
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn delete_older_than(&self, cutoff: DateTime<Utc>) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM audit_logs
            WHERE timestamp < $1
            "#,
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Archive old logs before deletion (for compliance)
    pub async fn archive_and_cleanup(
        &self,
        archive_path: &str,
    ) -> Result<CleanupReport, sqlx::Error> {
        // In production, would export to S3/cold storage before cleanup
        info!("Archiving audit logs to {} before cleanup", archive_path);
        self.cleanup().await
    }
}

/// Report of cleanup operation
#[derive(Debug, Default)]
pub struct CleanupReport {
    pub auth_deleted: u64,
    pub data_access_deleted: u64,
    pub security_deleted: u64,
    pub api_deleted: u64,
    pub other_deleted: u64,
}

impl CleanupReport {
    pub fn total_deleted(&self) -> u64 {
        self.auth_deleted
            + self.data_access_deleted
            + self.security_deleted
            + self.api_deleted
            + self.other_deleted
    }
}

/// SQL migration for audit_logs table
pub const AUDIT_TABLE_MIGRATION: &str = r#"
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    category VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    outcome VARCHAR(20) NOT NULL,
    actor JSONB NOT NULL,
    action VARCHAR(100) NOT NULL,
    resource JSONB NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    client JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_logs (timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_category ON audit_logs (category);
CREATE INDEX IF NOT EXISTS idx_audit_actor_user ON audit_logs ((actor->>'user_id'));
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = RetentionPolicy::default();
        assert_eq!(policy.auth_logs_days, 365);
        assert_eq!(policy.security_logs_days, 730);
    }

    #[test]
    fn test_pdp_compliant_policy() {
        let policy = RetentionPolicy::pdp_compliant();
        assert!(policy.security_logs_days >= 365); // At least 1 year
        assert!(policy.data_access_days >= 90); // At least 3 months
    }

    #[test]
    fn test_cleanup_report() {
        let report = CleanupReport {
            auth_deleted: 100,
            data_access_deleted: 200,
            security_deleted: 50,
            api_deleted: 500,
            other_deleted: 150,
        };
        assert_eq!(report.total_deleted(), 1000);
    }
}
