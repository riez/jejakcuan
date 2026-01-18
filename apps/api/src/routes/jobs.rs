//! Background job management for data source triggers
//!
//! Provides async execution of Python scrapers with status tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Status of a background job
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// A background job record
#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: String,
    pub source_id: String,
    pub source_name: String,
    pub command: String,
    pub status: JobStatus,
    pub message: Option<String>,
    pub output: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_secs: Option<f64>,
}

/// Job manager for tracking background jobs
#[derive(Debug, Default)]
pub struct JobManager {
    jobs: RwLock<HashMap<String, Job>>,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new job and start it in the background
    pub async fn spawn_job(
        self: &Arc<Self>,
        source_id: String,
        source_name: String,
        command: String,
    ) -> Job {
        let job_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let job = Job {
            id: job_id.clone(),
            source_id: source_id.clone(),
            source_name,
            command: command.clone(),
            status: JobStatus::Running,
            message: Some("Job started".to_string()),
            output: None,
            started_at: now,
            completed_at: None,
            duration_secs: None,
        };

        // Store job
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id.clone(), job.clone());
        }

        // Spawn background task
        let manager = Arc::clone(self);
        let job_id_clone = job_id.clone();
        let command_clone = command.clone();

        tokio::spawn(async move {
            let result = execute_command(&command_clone).await;
            let completed_at = Utc::now();

            let mut jobs = manager.jobs.write().await;
            if let Some(job) = jobs.get_mut(&job_id_clone) {
                job.completed_at = Some(completed_at);
                job.duration_secs =
                    Some((completed_at - job.started_at).num_milliseconds() as f64 / 1000.0);

                match result {
                    Ok(output) => {
                        job.status = JobStatus::Completed;
                        job.message = Some("Completed successfully".to_string());
                        job.output = Some(output);
                    }
                    Err(error) => {
                        job.status = JobStatus::Failed;
                        job.message = Some(format!("Failed: {}", error));
                        job.output = Some(error);
                    }
                }
            }

            // Cleanup old jobs (keep last 50)
            if jobs.len() > 50 {
                let mut job_list: Vec<_> = jobs.values().cloned().collect();
                job_list.sort_by(|a, b| b.started_at.cmp(&a.started_at));

                let to_remove: Vec<String> =
                    job_list.iter().skip(50).map(|j| j.id.clone()).collect();

                for id in to_remove {
                    jobs.remove(&id);
                }
            }
        });

        job
    }

    /// Get a job by ID
    pub async fn get_job(&self, job_id: &str) -> Option<Job> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).cloned()
    }

    /// Get all jobs for a source
    pub async fn get_jobs_for_source(&self, source_id: &str) -> Vec<Job> {
        let jobs = self.jobs.read().await;
        let mut source_jobs: Vec<_> = jobs
            .values()
            .filter(|j| j.source_id == source_id)
            .cloned()
            .collect();
        source_jobs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        source_jobs
    }

    /// Get recent jobs (last N)
    pub async fn get_recent_jobs(&self, limit: usize) -> Vec<Job> {
        let jobs = self.jobs.read().await;
        let mut all_jobs: Vec<_> = jobs.values().cloned().collect();
        all_jobs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        all_jobs.truncate(limit);
        all_jobs
    }

    /// Check if a source has a running job
    pub async fn is_source_running(&self, source_id: &str) -> Option<Job> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .find(|j| j.source_id == source_id && matches!(j.status, JobStatus::Running))
            .cloned()
    }
}

/// Execute a shell command and return output
async fn execute_command(command: &str) -> Result<String, String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    let ml_dir = std::env::current_dir()
        .map(|p| p.join("apps/ml"))
        .unwrap_or_else(|_| std::path::PathBuf::from("apps/ml"));

    tracing::info!("Executing job: {} in {:?}", command, ml_dir);

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .current_dir(&ml_dir)
        .env(
            "PYTHONPATH",
            ml_dir.join("src").to_string_lossy().to_string(),
        )
        .output()
        .await
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        let msg = if stdout.is_empty() {
            "Completed successfully (no output)".to_string()
        } else {
            // Return last 20 lines
            let lines: Vec<&str> = stdout.lines().collect();
            let last_lines: Vec<&str> = lines.iter().rev().take(20).rev().cloned().collect();
            last_lines.join("\n")
        };
        Ok(msg)
    } else {
        let error_msg = if stderr.is_empty() {
            format!("Exit code: {:?}\n{}", output.status.code(), stdout)
        } else {
            // Return last 20 lines of stderr
            let lines: Vec<&str> = stderr.lines().collect();
            let last_lines: Vec<&str> = lines.iter().rev().take(20).rev().cloned().collect();
            last_lines.join("\n")
        };
        Err(error_msg)
    }
}
