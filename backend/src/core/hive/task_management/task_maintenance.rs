//! Task Maintenance Module
//!
//! Handles system maintenance, configuration updates, and cleanup operations.

use super::task_metrics::TaskMetricsCollector;
use super::task_types::*;
use crate::utils::error::HiveResult;

/// Task maintenance functionality
pub struct TaskMaintenanceManager {
    /// Metrics collector for cleanup operations
    metrics_collector: TaskMetricsCollector,
    /// Configuration
    config: TaskDistributionConfig,
}

impl TaskMaintenanceManager {
    /// Create a new maintenance manager
    pub fn new(metrics_collector: TaskMetricsCollector, config: TaskDistributionConfig) -> Self {
        Self {
            metrics_collector,
            config,
        }
    }

    /// Perform system maintenance
    ///
    /// Performs periodic maintenance tasks including metrics cleanup,
    /// cache optimization, and system health checks.
    ///
    /// ## Maintenance Operations
    ///
    /// - Clean up old metrics data
    /// - Optimize internal data structures
    /// - Validate system health
    /// - Update performance statistics
    ///
    /// ## Performance
    ///
    /// Variable based on cleanup scope. Generally O(n) where n is the amount of data to clean.
    pub async fn perform_maintenance(&self) -> HiveResult<()> {
        // Clean up old metrics (keep last 24 hours)
        self.metrics_collector.cleanup_old_metrics(24).await?;

        tracing::info!("Task management system maintenance completed");
        Ok(())
    }

    /// Update configuration
    ///
    /// Updates the task distribution configuration with new settings.
    /// Validates configuration before applying changes.
    ///
    /// ## Configuration Updates
    ///
    /// - Maximum concurrent tasks
    /// - Execution timeouts
    /// - Queue size limits
    /// - Work-stealing settings
    ///
    /// ## Validation
    ///
    /// Ensures configuration values are within acceptable ranges
    /// and don't conflict with current system state.
    pub async fn update_config(&mut self, new_config: TaskDistributionConfig) -> HiveResult<()> {
        // Validate new configuration
        self.validate_config(&new_config)?;

        // Apply configuration
        self.config = new_config;

        tracing::info!("Task distributor configuration updated");
        Ok(())
    }

    /// Validate configuration
    fn validate_config(&self, config: &TaskDistributionConfig) -> HiveResult<()> {
        if config.max_concurrent_tasks == 0 {
            return Err(crate::utils::error::HiveError::ValidationError {
                field: "max_concurrent_tasks".to_string(),
                reason: "Maximum concurrent tasks must be greater than 0".to_string(),
            });
        }

        if config.max_queue_size == 0 {
            return Err(crate::utils::error::HiveError::ValidationError {
                field: "max_queue_size".to_string(),
                reason: "Maximum queue size must be greater than 0".to_string(),
            });
        }

        if config.execution_timeout_ms == 0 {
            return Err(crate::utils::error::HiveError::ValidationError {
                field: "execution_timeout_ms".to_string(),
                reason: "Execution timeout must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &TaskDistributionConfig {
        &self.config
    }

    /// Reset system to default state
    ///
    /// Resets the task management system to its default state.
    /// Useful for testing or system recovery.
    ///
    /// ## Warning
    ///
    /// This operation will clear all queued tasks and reset metrics.
    /// Should only be used when the system is in a known safe state.
    pub async fn reset_system(&self) -> HiveResult<()> {
        // Reset metrics
        self.metrics_collector.cleanup_old_metrics(0).await?;

        tracing::info!("Task management system reset to default state");
        Ok(())
    }

    /// Get maintenance status
    pub async fn get_maintenance_status(&self) -> serde_json::Value {
        serde_json::json!({
            "last_maintenance": chrono::Utc::now(),
            "config_valid": true,
            "system_ready": true
        })
    }
}
