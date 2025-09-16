//! # Centralized Error Handling Usage Examples
//!
//! This file demonstrates how to use the centralized error handling system
//! throughout the AI Orchestrator Hub codebase to ensure consistent error
//! patterns and prevent production panics.

use crate::utils::error::*;
use crate::utils::error_recovery::*;
use std::time::Duration;

/// Example agent implementation using centralized error handling
pub struct ExampleAgent {
    id: String,
    error_handler: Arc<CentralizedErrorHandler>,
}

impl ExampleAgent {
    /// Create a new agent with centralized error handling
    pub fn new(id: String) -> Self {
        let config = ErrorHandlerConfig {
            enable_automatic_recovery: true,
            max_recovery_attempts: 3,
            enable_circuit_breakers: true,
            circuit_breaker_threshold: 5,
            ..Default::default()
        };
        
        Self {
            id,
            error_handler: Arc::new(CentralizedErrorHandler::new(config)),
        }
    }

    /// Example operation using centralized error handling
    pub async fn perform_critical_task(&self, task_data: &str) -> HiveResult<String> {
        // Use the centralized error handler for this operation
        self.error_handler
            .execute_with_centralized_handling(
                || Box::pin(async move {
                    // Simulate task processing
                    if task_data.is_empty() {
                        return Err(HiveError::ValidationError {
                            field: "task_data".to_string(),
                            reason: "Task data cannot be empty".to_string(),
                        });
                    }

                    // Simulate processing that might fail
                    if task_data.contains("fail") {
                        return Err(HiveError::AgentExecutionFailed {
                            reason: "Simulated task failure".to_string(),
                        });
                    }

                    // Simulate successful processing
                    Ok(format!("Processed: {}", task_data))
                }),
                "perform_critical_task",
                "ExampleAgent",
                Some(&self.id),
            )
            .await
    }

    /// Example learning operation with agent-specific recovery
    pub async fn perform_learning(&self, training_data: &[f64]) -> HiveResult<f64> {
        self.error_handler
            .execute_with_centralized_handling(
                || Box::pin(async move {
                    if training_data.is_empty() {
                        return Err(HiveError::AdaptiveLearningDataInsufficient {
                            required_samples: 1,
                            available_samples: 0,
                        });
                    }

                    // Simulate learning process that might fail
                    if training_data.len() < 10 {
                        return Err(HiveError::AgentLearningFailed {
                            agent_id: self.id.clone(),
                            reason: "Insufficient training data".to_string(),
                        });
                    }

                    // Simulate successful learning
                    let accuracy = training_data.iter().sum::<f64>() / training_data.len() as f64;
                    Ok(accuracy)
                }),
                "perform_learning",
                "ExampleAgent",
                Some(&self.id),
            )
            .await
    }

    /// Example of handling external service calls
    pub async fn call_external_service(&self, service_url: &str) -> HiveResult<String> {
        self.error_handler
            .execute_with_centralized_handling(
                || Box::pin(async move {
                    // Simulate external service call
                    if service_url.contains("timeout") {
                        return Err(HiveError::ExternalServiceTimeout {
                            service_name: "ExternalAPI".to_string(),
                            timeout_ms: 5000,
                        });
                    }

                    if service_url.contains("unavailable") {
                        return Err(HiveError::ExternalServiceUnavailable {
                            service_name: "ExternalAPI".to_string(),
                            endpoint: service_url.to_string(),
                        });
                    }

                    // Simulate successful service call
                    Ok(format!("Response from {}", service_url))
                }),
                "call_external_service",
                "ExampleAgent",
                Some(&self.id),
            )
            .await
    }

    /// Get agent health status
    pub async fn get_health_status(&self) -> HealthMetrics {
        match self.error_handler
            .get_component_health("ExampleAgent")
            .await
        {
            Some(health) => health,
            None => HealthMetrics {
                total_operations: 0,
                successful_operations: 0,
                failed_operations: 0,
                recovery_attempts: 0,
                successful_recoveries: 0,
                average_recovery_time: Duration::from_millis(0),
                last_failure_time: None,
            }
        }
    }

    /// Get circuit breaker status
    pub async fn get_circuit_breaker_status(&self) -> Option<CircuitState> {
        self.error_handler
            .get_circuit_breaker_status("ExampleAgent")
            .await
    }
}

/// Example task manager using centralized error handling
pub struct ExampleTaskManager {
    error_handler: Arc<CentralizedErrorHandler>,
}

impl ExampleTaskManager {
    pub fn new() -> Self {
        let config = ErrorHandlerConfig {
            enable_automatic_recovery: true,
            max_recovery_attempts: 5,
            enable_circuit_breakers: true,
            circuit_breaker_threshold: 3,
            circuit_breaker_timeout: Duration::from_secs(30),
            enable_health_monitoring: true,
            health_check_interval: Duration::from_secs(10),
            ..Default::default()
        };

        Self {
            error_handler: Arc::new(CentralizedErrorHandler::new(config)),
        }
    }

    /// Example task orchestration with error handling
    pub async fn orchestrate_tasks(&self, tasks: Vec<String>) -> HiveResult<Vec<String>> {
        self.error_handler
            .execute_with_centralized_handling(
                || Box::pin(async move {
                    if tasks.is_empty() {
                        return Err(HiveError::TaskCreationFailed {
                            reason: "No tasks provided".to_string(),
                        });
                    }

                    if tasks.len() > 100 {
                        return Err(HiveError::SystemOverloaded {
                            reason: "Too many tasks for single orchestration".to_string(),
                        });
                    }

                    // Simulate task processing
                    let results: Vec<String> = tasks
                        .into_iter()
                        .map(|task| format!("Completed: {}", task))
                        .collect();

                    Ok(results)
                }),
                "orchestrate_tasks",
                "TaskManager",
                None,
            )
            .await
    }

    /// Example of handling task dependencies
    pub async fn resolve_task_dependencies(&self, task_id: &str, dependencies: &[String]) -> HiveResult<()> {
        self.error_handler
            .execute_with_centralized_handling(
                || Box::pin(async move {
                    if dependencies.is_empty() {
                        return Ok(()); // No dependencies to resolve
                    }

                    // Check for circular dependencies (simplified)
                    if dependencies.contains(&task_id.to_string()) {
                        return Err(HiveError::TaskDependencyCycleDetected {
                            task_id: task_id.to_string(),
                            cycle_path: format!("{} -> {}", task_id, task_id),
                        });
                    }

                    // Simulate dependency resolution
                    for dep in dependencies {
                        if dep.starts_with("missing_") {
                            return Err(HiveError::TaskNotFound {
                                id: dep.clone(),
                            });
                        }
                    }

                    Ok(())
                }),
                "resolve_task_dependencies",
                "TaskManager",
                None,
            )
            .await
    }

    /// Get system health score
    pub async fn get_system_health(&self) -> f64 {
        self.error_handler.get_system_health_score().await
    }
}

/// Example usage of the centralized error handling system
pub async fn demonstrate_centralized_error_handling() {
    println!("=== Centralized Error Handling Demonstration ===");

    // Create an agent with centralized error handling
    let agent = ExampleAgent::new("demo-agent-001".to_string());

    // Test successful operation
    println!("1. Testing successful operation...");
    match agent.perform_critical_task("valid task data").await {
        Ok(result) => println!("✓ Success: {}", result),
        Err(e) => println!("✗ Error: {}", e),
    }

    // Test validation error
    println!("\n2. Testing validation error...");
    match agent.perform_critical_task("").await {
        Ok(result) => println!("✓ Success: {}", result),
        Err(e) => println!("✗ Expected validation error: {}", e),
    }

    // Test execution error
    println!("\n3. Testing execution error...");
    match agent.perform_critical_task("this should fail").await {
        Ok(result) => println!("✓ Success: {}", result),
        Err(e) => println!("✗ Expected execution error: {}", e),
    }

    // Test learning operation
    println!("\n4. Testing learning operation...");
    match agent.perform_learning(&[1.0, 2.0, 3.0, 4.0, 5.0]).await {
        Ok(accuracy) => println!("✓ Learning accuracy: {}", accuracy),
        Err(e) => println!("✗ Learning error: {}", e),
    }

    // Test insufficient data for learning
    println!("\n5. Testing insufficient learning data...");
    match agent.perform_learning(&[1.0]).await {
        Ok(accuracy) => println!("✓ Learning accuracy: {}", accuracy),
        Err(e) => println!("✗ Expected learning error: {}", e),
    }

    // Test external service call
    println!("\n6. Testing external service call...");
    match agent.call_external_service("https://api.example.com/data").await {
        Ok(response) => println!("✓ Service response: {}", response),
        Err(e) => println!("✗ Service error: {}", e),
    }

    // Test external service timeout
    println!("\n7. Testing external service timeout...");
    match agent.call_external_service("https://api.example.com/timeout").await {
        Ok(response) => println!("✓ Service response: {}", response),
        Err(e) => println!("✗ Expected timeout error: {}", e),
    }

    // Check agent health
    println!("\n8. Checking agent health...");
    let health = agent.get_health_status().await;
    println!("Total operations: {}", health.total_operations);
    println!("Successful operations: {}", health.successful_operations);
    println!("Failed operations: {}", health.failed_operations);

    // Check circuit breaker status
    println!("\n9. Checking circuit breaker status...");
    if let Some(status) = agent.get_circuit_breaker_status().await {
        println!("Circuit breaker status: {:?}", status);
    } else {
        println!("Circuit breaker not enabled");
    }

    // Test task manager
    println!("\n10. Testing task manager...");
    let task_manager = ExampleTaskManager::new();

    match task_manager.orchestrate_tasks(vec![
        "task1".to_string(),
        "task2".to_string(),
        "task3".to_string(),
    ]).await {
        Ok(results) => println!("✓ Orchestrated {} tasks", results.len()),
        Err(e) => println!("✗ Task orchestration error: {}", e),
    }

    // Test dependency resolution
    println!("\n11. Testing dependency resolution...");
    match task_manager
        .resolve_task_dependencies("task1", &["dep1".to_string(), "dep2".to_string()])
        .await
    {
        Ok(()) => println!("✓ Dependencies resolved"),
        Err(e) => println!("✗ Dependency resolution error: {}", e),
    }

    // Test circular dependency detection
    println!("\n12. Testing circular dependency detection...");
    match task_manager
        .resolve_task_dependencies("task1", &["task1".to_string()])
        .await
    {
        Ok(()) => println!("✓ Dependencies resolved"),
        Err(e) => println!("✗ Expected circular dependency error: {}", e),
    }

    // Get system health
    println!("\n13. Getting system health...");
    let system_health = task_manager.get_system_health().await;
    println!("System health score: {:.2}", system_health);

    println!("\n=== Demonstration Complete ===");
}

/// Example of using the convenience macros
pub async fn demonstrate_macro_usage() {
    println!("\n=== Macro Usage Demonstration ===");

    // Using the centralized error handling macro
    let result = handle_with_centralized_error_recovery!(
        Ok::<i32, &str>(42),
        "macro_test_operation",
        "MacroComponent"
    );

    match result {
        Ok(value) => println!("✓ Macro success: {}", value),
        Err(e) => println!("✗ Macro error: {}", e),
    }

    // Using the agent-specific macro
    let result = handle_agent_with_centralized_error_recovery!(
        Ok::<String, &str>("agent macro success".to_string()),
        "agent_macro_operation",
        "AgentMacroComponent",
        "demo-agent-002"
    );

    match result {
        Ok(value) => println!("✓ Agent macro success: {}", value),
        Err(e) => println!("✗ Agent macro error: {}", e),
    }

    println!("=== Macro Demonstration Complete ===");
}

/// Example of error prevention patterns
pub async fn demonstrate_error_prevention() {
    println!("\n=== Error Prevention Demonstration ===");

    // Example 1: Safe option handling
    let some_value: Option<i32> = Some(42);
    let none_value: Option<i32> = None;

    let safe_result = some_value.ok_or_else(|| {
        error!("Option handling failed in TestComponent during test_operation");
        HiveError::OperationFailed {
            reason: "Expected Some value in TestComponent during test_operation".to_string(),
        }
    });
    match safe_result {
        Ok(value) => println!("✓ Safe option success: {}", value),
        Err(e) => println!("✗ Safe option error: {}", e),
    }

    let safe_none_result = none_value.ok_or_else(|| {
        error!("Option handling failed in TestComponent during test_operation_none");
        HiveError::OperationFailed {
            reason: "Expected Some value in TestComponent during test_operation_none".to_string(),
        }
    });
    match safe_none_result {
        Ok(value) => println!("✓ Safe option success: {}", value),
        Err(e) => println!("✗ Expected safe option error: {}", e),
    }

    // Example 2: Safe result handling
    let ok_result: Result<i32, &str> = Ok(100);
    let err_result: Result<i32, &str> = Err("error message");

    let safe_ok_result = ok_result.map_err(|e| {
        error!("Result handling failed in TestComponent during test_result_ok: {}", e);
        HiveError::OperationFailed {
            reason: format!("Operation failed in TestComponent during test_result_ok: {}", e),
        }
    });
    match safe_ok_result {
        Ok(value) => println!("✓ Safe result success: {}", value),
        Err(e) => println!("✗ Safe result error: {}", e),
    }

    let safe_err_result = err_result.map_err(|e| {
        error!("Result handling failed in TestComponent during test_result_err: {}", e);
        HiveError::OperationFailed {
            reason: format!("Operation failed in TestComponent during test_result_err: {}", e),
        }
    });
    match safe_err_result {
        Ok(value) => println!("✓ Safe result success: {}", value),
        Err(e) => println!("✗ Expected safe result error: {}", e),
    }

    // Example 3: Using defaults safely
    let default_value = match none_value {
        Some(value) => value,
        None => {
            warn!("Using default value in TestComponent during default_test");
            i32::default()
        }
    };
    println!("✓ Safe default value: {}", default_value);

    let custom_default = match none_value {
        Some(value) => value,
        None => {
            warn!("Using fallback value in TestComponent during custom_default_test");
            999
        }
    };
    println!("✓ Safe custom default: {}", custom_default);

    println!("=== Error Prevention Demonstration Complete ===");
}

#[tokio::main]
async fn main() {
    demonstrate_centralized_error_handling().await;
    demonstrate_macro_usage().await;
    demonstrate_error_prevention().await;
}