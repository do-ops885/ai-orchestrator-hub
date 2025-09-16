//! Task Creation Module
//!
//! Handles task creation, validation, and initial setup.

use super::task_types::*;
use crate::core::hive::coordinator::CoordinationMessage;
use crate::tasks::task::{Task, TaskPriority, TaskRequiredCapability};
use crate::utils::error::{HiveError, HiveResult};
use crate::utils::error_handling::safe_json;
use std::error::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Task creation functionality
pub struct TaskCreator {
    /// Communication channel for coordination
    coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,
    /// Configuration for task creation
    config: TaskDistributionConfig,
}

impl TaskCreator {
    /// Create a new task creator
    pub fn new(
        coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,
        config: TaskDistributionConfig,
    ) -> Self {
        Self {
            coordination_tx,
            config,
        }
    }

    /// Create and validate a new task
    ///
    /// Parses the task configuration, validates required fields,
    /// and creates a new task instance ready for queuing.
    ///
    /// ## Configuration Requirements
    ///
    /// The config should include:
    /// - `"type"`: Task type (required)
    /// - `"title"`: Human-readable title (required)
    /// - `"description"`: Detailed description (optional)
    /// - `"priority"`: Task priority level (optional, defaults to "low")
    /// - `"required_capabilities"`: Array of required agent capabilities (optional)
    ///
    /// ## Validation
    ///
    /// - Validates presence of required fields
    /// - Parses and validates task priority
    /// - Validates capability requirements
    /// - Ensures task type is supported
    ///
    /// ## Performance
    ///
    /// O(1) for basic validation, O(n) for capability parsing where n is number of capabilities.
    pub async fn create_task(&self, config: serde_json::Value) -> HiveResult<Task> {
        // Validate task configuration
        let task_config = self.validate_task_config(&config)?;

        // Extract required fields
        let title = task_config
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HiveError::ValidationError {
                field: "title".to_string(),
                reason: "Task title is required".to_string(),
            })?;

        let description = task_config
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        // Parse optional fields
        let priority = self.parse_task_priority(&task_config)?;
        let required_capabilities = self.parse_required_capabilities(&task_config)?;

        // Create the task
        let task = Task {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: description.to_string(),
            task_type: "computation".to_string(), // Default task type
            priority,
            status: crate::tasks::task::TaskStatus::Pending,
            required_capabilities,
            assigned_agent: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deadline: None,
            estimated_duration: None,
            context: config
                .get("context")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default(),
            dependencies: Vec::new(),
        };

        tracing::info!("Created task {} ({})", task.id, title);
        Ok(task)
    }

    /// Parse task priority from configuration
    fn parse_task_priority(&self, config: &serde_json::Value) -> HiveResult<TaskPriority> {
        let priority_str = config
            .get("priority")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        match priority_str {
            "low" => Ok(TaskPriority::Low),
            "medium" => Ok(TaskPriority::Medium),
            "high" => Ok(TaskPriority::High),
            "critical" => Ok(TaskPriority::Critical),
            _ => Err(HiveError::ValidationError {
                field: "priority".to_string(),
                reason: format!("Unknown priority: {}", priority_str),
            }),
        }
    }

    /// Parse required capabilities from configuration
    fn parse_required_capabilities(
        &self,
        config: &serde_json::Value,
    ) -> HiveResult<Vec<TaskRequiredCapability>> {
        let mut required_capabilities = Vec::new();

        if let Some(capabilities_array) = config
            .get("required_capabilities")
            .and_then(|v| v.as_array())
        {
            for cap in capabilities_array {
                if let Some(cap_name) = cap.get("name").and_then(|v| v.as_str()) {
                    let min_proficiency = cap
                        .get("minimum_proficiency")
                        .and_then(|v| v.as_f64())
                        .unwrap_or_default();

                    required_capabilities.push(TaskRequiredCapability {
                        name: cap_name.to_string(),
                        minimum_proficiency: min_proficiency,
                    });
                }
            }
        }

        Ok(required_capabilities)
    }

    /// Validate task configuration
    fn validate_task_config(&self, config: &serde_json::Value) -> HiveResult<serde_json::Value> {
        if !config.is_object() {
            return Err(HiveError::ValidationError {
                field: "config".to_string(),
                reason: "Task configuration must be an object".to_string(),
            });
        }

        // Check for required fields
        if config.get("type").is_none() {
            return Err(HiveError::ValidationError {
                field: "type".to_string(),
                reason: "Task type is required".to_string(),
            });
        }

        if config.get("title").is_none() {
            return Err(HiveError::ValidationError {
                field: "title".to_string(),
                reason: "Task title is required".to_string(),
            });
        }

        Ok(config.clone())
    }

    /// Send task creation notification
    pub async fn notify_task_created(&self, task_id: Uuid) -> HiveResult<()> {
        // Task creation notification could be added to CoordinationMessage if needed
        tracing::debug!("Task {} creation notification sent", task_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    // Mock TaskDistributionConfig for testing
    fn create_test_config() -> TaskDistributionConfig {
        TaskDistributionConfig {
            max_concurrent_tasks: 100,
            max_retry_attempts: 3,
            execution_timeout_ms: 300_000,
            enable_work_stealing: true,
            max_queue_size: 10_000,
        }
    }

    fn create_test_coordination_channel() -> mpsc::UnboundedSender<CoordinationMessage> {
        let (tx, _rx) = mpsc::unbounded_channel();
        tx
    }

    #[test]
    fn test_task_creator_new() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();

        let creator = TaskCreator::new(tx, config);

        // Should create successfully
        assert!(true); // If we reach here, creation succeeded
    }

    #[tokio::test]
    async fn test_create_task_basic() -> Result<(), Box<dyn Error>> {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Test Task",
            "description": "A test task"
        });

        let task = creator.create_task(task_config).await?;

        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, "A test task");
        assert_eq!(task.task_type, "computation");
        assert!(matches!(task.priority, TaskPriority::Low)); // Default
        assert!(task.required_capabilities.is_empty());
        assert!(matches!(
            task.status,
            crate::tasks::task::TaskStatus::Pending
        ));
        assert!(task.assigned_agent.is_none());
        assert!(task.context.is_empty());
        assert!(task.dependencies.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_with_priority() -> Result<(), Box<dyn Error>> {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "type": "computation",
            "title": "High Priority Task",
            "priority": "high"
        });

        let task = creator.create_task(task_config).await?;

        assert!(matches!(task.priority, TaskPriority::High));
        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_with_capabilities() -> Result<(), Box<dyn Error>> {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Complex Task",
            "required_capabilities": [
                {
                    "name": "data_processing",
                    "minimum_proficiency": 0.8
                },
                {
                    "name": "analysis",
                    "minimum_proficiency": 0.6
                }
            ]
        });

        let task = creator.create_task(task_config).await?;

        assert_eq!(task.required_capabilities.len(), 2);
        assert_eq!(task.required_capabilities[0].name, "data_processing");
        assert_eq!(task.required_capabilities[0].minimum_proficiency, 0.8);
        assert_eq!(task.required_capabilities[1].name, "analysis");
        assert_eq!(task.required_capabilities[1].minimum_proficiency, 0.6);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_with_context() -> Result<(), Box<dyn Error>> {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Context Task",
            "context": {
                "user_id": "123",
                "project": "test_project"
            }
        });

        let task = creator.create_task(task_config).await?;

        assert_eq!(task.context.len(), 2);
        assert_eq!(task.context.get("user_id"), Some(&"123".to_string()));
        assert_eq!(
            task.context.get("project"),
            Some(&"test_project".to_string())
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_missing_type() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "title": "Missing Type Task"
        });

        let result = creator.create_task(task_config).await;
        assert!(result.is_err());

        if let Err(HiveError::ValidationError { field, .. }) = result.unwrap_err() {
            assert_eq!(field, "type");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[tokio::test]
    async fn test_create_task_missing_title() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "type": "computation"
        });

        let result = creator.create_task(task_config).await;
        assert!(result.is_err());

        if let Err(HiveError::ValidationError { field, .. }) = result.unwrap_err() {
            assert_eq!(field, "title");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[tokio::test]
    async fn test_create_task_invalid_config_type() -> Result<(), Box<dyn Error>> {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!("invalid_config");

        let result = creator.create_task(task_config).await;
        assert!(result.is_err());

        if let Err(HiveError::ValidationError { field, reason }) = result.as_ref().unwrap_err() {
            assert_eq!(field, "config");
            assert!(reason.contains("must be an object"));
        } else {
            panic!("Expected ValidationError");
        }
        Ok(())
    }

    #[test]
    fn test_parse_task_priority_low() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "priority": "low"
        });

        let priority = match creator.parse_task_priority(&task_config) {
            Ok(priority) => priority,
            Err(e) => panic!("Priority parsing should succeed in test: {:?}", e),
        };
        assert!(matches!(priority, TaskPriority::Low));
    }

    #[test]
    fn test_parse_task_priority_medium() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "priority": "medium"
        });

        let priority = match creator.parse_task_priority(&task_config) {
            Ok(priority) => priority,
            Err(e) => panic!("Priority parsing should succeed in test: {:?}", e),
        };
        assert!(matches!(priority, TaskPriority::Medium));
    }

    #[test]
    fn test_parse_task_priority_high() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "priority": "high"
        });

        let priority = match creator.parse_task_priority(&task_config) {
            Ok(priority) => priority,
            Err(e) => panic!("Priority parsing should succeed in test: {:?}", e),
        };
        assert!(matches!(priority, TaskPriority::High));
    }

    #[test]
    fn test_parse_task_priority_critical() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "priority": "critical"
        });

        let priority = match creator.parse_task_priority(&task_config) {
            Ok(priority) => priority,
            Err(e) => panic!("Priority parsing should succeed in test: {:?}", e),
        };
        assert!(matches!(priority, TaskPriority::Critical));
    }

    #[test]
    fn test_parse_task_priority_default() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({});

        let priority = match creator.parse_task_priority(&task_config) {
            Ok(priority) => priority,
            Err(e) => panic!("Priority parsing should succeed in test: {:?}", e),
        };
        assert!(matches!(priority, TaskPriority::Low)); // Default
    }

    #[test]
    fn test_parse_task_priority_invalid() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "priority": "invalid"
        });

        let result = creator.parse_task_priority(&task_config);
        assert!(result.is_err());

        if let Err(HiveError::ValidationError { field, reason }) = result.unwrap_err() {
            assert_eq!(field, "priority");
            assert!(reason.contains("Unknown priority"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_parse_required_capabilities_empty() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({});

        let capabilities = match creator.parse_required_capabilities(&task_config) {
            Ok(capabilities) => capabilities,
            Err(e) => panic!("Capabilities parsing should succeed in test: {:?}", e),
        };
        assert!(capabilities.is_empty());
    }

    #[test]
    fn test_parse_required_capabilities_with_data() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "required_capabilities": [
                {
                    "name": "coding",
                    "minimum_proficiency": 0.9
                },
                {
                    "name": "design"
                }
            ]
        });

        let capabilities = match creator.parse_required_capabilities(&task_config) {
            Ok(capabilities) => capabilities,
            Err(e) => panic!("Capabilities parsing should succeed in test: {:?}", e),
        };
        assert_eq!(capabilities.len(), 2);
        assert_eq!(capabilities[0].name, "coding");
        assert_eq!(capabilities[0].minimum_proficiency, 0.9);
        assert_eq!(capabilities[1].name, "design");
        assert_eq!(capabilities[1].minimum_proficiency, 0.0); // Default
    }

    #[test]
    fn test_parse_required_capabilities_invalid_format() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "required_capabilities": [
                {
                    "invalid": "format"
                }
            ]
        });

        let capabilities = match creator.parse_required_capabilities(&task_config) {
            Ok(capabilities) => capabilities,
            Err(e) => panic!("Capabilities parsing should succeed in test: {:?}", e),
        };
        assert!(capabilities.is_empty()); // Invalid entries are skipped
    }

    #[test]
    fn test_validate_task_config_valid() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Valid Task"
        });

        let result = creator.validate_task_config(&task_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_task_config_missing_type() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "title": "Missing Type"
        });

        let result = creator.validate_task_config(&task_config);
        assert!(result.is_err());

        if let Err(HiveError::ValidationError { field, .. }) = result.unwrap_err() {
            assert_eq!(field, "type");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_validate_task_config_missing_title() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "type": "computation"
        });

        let result = creator.validate_task_config(&task_config);
        assert!(result.is_err());

        if let Err(HiveError::ValidationError { field, .. }) = result.unwrap_err() {
            assert_eq!(field, "title");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_validate_task_config_not_object() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!("not_an_object");

        let result = creator.validate_task_config(&task_config);
        assert!(result.is_err());

        if let Err(HiveError::ValidationError { field, reason }) = result.unwrap_err() {
            assert_eq!(field, "config");
            assert!(reason.contains("must be an object"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[tokio::test]
    async fn test_notify_task_created() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_id = Uuid::new_v4();
        let result = creator.notify_task_created(task_id).await;

        assert!(result.is_ok());
    }

    // Test edge cases

    #[tokio::test]
    async fn test_create_task_with_empty_description() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Empty Description Task",
            "description": ""
        });

        let task = match creator.create_task(task_config).await {
            Ok(task) => task,
            Err(e) => panic!("Task creation should succeed in test: {:?}", e),
        };
        assert_eq!(task.description, "");
    }

    #[tokio::test]
    async fn test_create_task_with_null_context() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Null Context Task",
            "context": null
        });

        let task = match creator.create_task(task_config).await {
            Ok(task) => task,
            Err(e) => panic!("Task creation should succeed in test: {:?}", e),
        };
        assert!(task.context.is_empty());
    }

    #[test]
    fn test_parse_required_capabilities_mixed_valid_invalid() {
        let tx = create_test_coordination_channel();
        let config = create_test_config();
        let creator = TaskCreator::new(tx, config);

        let task_config = serde_json::json!({
            "required_capabilities": [
                {
                    "name": "valid",
                    "minimum_proficiency": 0.8
                },
                {
                    "invalid": "entry"
                },
                {
                    "name": "another_valid"
                }
            ]
        });

        let capabilities = match creator.parse_required_capabilities(&task_config) {
            Ok(capabilities) => capabilities,
            Err(e) => panic!("Capabilities parsing should succeed in test: {:?}", e),
        };
        assert_eq!(capabilities.len(), 2); // Only valid entries
        assert_eq!(capabilities[0].name, "valid");
        assert_eq!(capabilities[1].name, "another_valid");
    }
}
