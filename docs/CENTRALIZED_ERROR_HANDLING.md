# Centralized Error Handling System

## Overview

The AI Orchestrator Hub implements a comprehensive centralized error handling system designed to prevent production panics, ensure system reliability, and provide consistent error patterns across all modules. This system integrates circuit breakers, retry mechanisms, agent-specific recovery strategies, and health monitoring to create a resilient multi-agent system.

## Core Components

### 1. Enhanced Error Types (`error.rs`)

The system provides a comprehensive set of error types covering all aspects of the multi-agent system:

#### Agent-Specific Errors
- `AgentLearningFailed`: When agent learning processes fail
- `AgentMemoryCorruption`: When agent memory becomes corrupted
- `AgentDeadlockDetected`: When agent deadlocks occur
- `AgentResourceStarvation`: When agents lack necessary resources
- `AgentAdaptationFailed`: When agent adaptation processes fail
- `AgentCommunicationProtocolError`: When agent communication fails
- `AgentSkillEvolutionFailed`: When skill evolution processes fail
- `AgentVerificationFailed`: When agent verification fails
- `AgentCollaborativeLearningFailed`: When collaborative learning fails
- `AgentEvolutionStalled`: When agent evolution stops progressing

#### System Resilience Errors
- `SystemResilienceCheckFailed`: When resilience checks fail
- `RecoveryCoordinatorUnavailable`: When recovery systems are unavailable
- `HealthCheckFailed`: When health checks fail
- `GracefulShutdownTimeout`: When graceful shutdown times out
- `ResourceCleanupFailed`: When resource cleanup fails
- `StateSynchronizationFailed`: When state synchronization fails

#### Task Orchestration Errors
- `TaskOrchestrationFailed`: When task orchestration fails
- `TaskDependencyCycleDetected`: When circular dependencies are found
- `TaskPriorityConflict`: When task priorities conflict
- `TaskResourceContention`: When tasks compete for resources
- `TaskDeadlineExceeded`: When tasks miss deadlines

#### External Service Errors
- `ExternalServiceUnavailable`: When external services are down
- `ExternalServiceRateLimited`: When rate limits are exceeded
- `ExternalServiceAuthenticationFailed`: When authentication fails
- `ExternalServiceResponseInvalid`: When responses are invalid
- `ExternalServiceTimeout`: When services time out

### 2. Error Recovery Mechanisms (`error_recovery.rs`)

The system provides multiple layers of error recovery:

#### Circuit Breakers
Prevent cascading failures by temporarily blocking operations to failing components:

```rust
let config = ErrorHandlerConfig {
    enable_circuit_breakers: true,
    circuit_breaker_threshold: 5,
    circuit_breaker_timeout: Duration::from_secs(60),
    ..Default::default()
};
let handler = CentralizedErrorHandler::new(config);
```

#### Retry Mechanisms
Configurable retry logic with exponential backoff:

```rust
let result = handler.execute_with_centralized_handling(
    || Box::pin(async { /* your operation */ }),
    "operation_name",
    "ComponentName",
    None,
).await?;
```

#### Agent-Specific Recovery Strategies
Specialized recovery strategies for different agent types:

- **RestartWithBackoff**: Restart agent with exponential backoff
- **FailoverToBackup**: Switch to backup agent instance
- **CapabilityReduction**: Temporarily reduce agent capabilities
- **MemoryReset**: Reset agent memory and learning state
- **SimplifiedMode**: Enter simplified operation mode
- **SwarmIsolation**: Isolate agent from swarm communication
- **LearningRollback**: Rollback to previous learning checkpoint
- **WorkloadRedistribution**: Redistribute workload to peer agents

#### Adaptive Error Recovery
Intelligent recovery based on error context and history:

```rust
let recovery = ContextAwareRecovery::new();
let mut context = RecoveryContext::new("operation", "component", 3);
let result = recovery.execute_with_adaptive_recovery(operation, &mut context).await?;
```

### 3. Centralized Error Handler

The main entry point for all error handling operations:

```rust
pub struct CentralizedErrorHandler {
    context_aware_recovery: Arc<ContextAwareRecovery>,
    health_monitor: Arc<RecoveryHealthMonitor>,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    error_classifier: ErrorClassifier,
    config: ErrorHandlerConfig,
}
```

## Usage Patterns

### 1. Basic Error Handling

```rust
use crate::utils::error_recovery::*;

async fn example_operation() -> HiveResult<String> {
    let handler = CentralizedErrorHandler::new(ErrorHandlerConfig::default());
    
    handler.execute_with_centralized_handling(
        || Box::pin(async {
            // Your operation logic here
            Ok("success".to_string())
        }),
        "example_operation",
        "ExampleComponent",
        None,
    ).await
}
```

### 2. Agent-Specific Error Handling

```rust
async fn agent_operation(agent_id: &str) -> HiveResult<String> {
    let handler = CentralizedErrorHandler::new(ErrorHandlerConfig::default());
    
    handler.execute_with_centralized_handling(
        || Box::pin(async {
            // Agent-specific logic here
            Ok("agent success".to_string())
        }),
        "agent_operation",
        "NeuralAgent",
        Some(agent_id),
    ).await
}
```

### 3. Using Convenience Macros

```rust
// General error handling
let result = handle_with_centralized_error_recovery!(
    Ok::<i32, &str>(42),
    "test_operation",
    "TestComponent"
);

// Agent-specific error handling
let result = handle_agent_with_centralized_error_recovery!(
    Ok::<String, &str>("success".to_string()),
    "agent_operation",
    "AgentComponent",
    "agent-123"
);
```

### 4. Safe Unwrap Alternatives

Replace `unwrap()` calls with safe alternatives:

```rust
// Instead of: option.unwrap()
let result = safe_unwrap!(option, "operation", "component")?;

// Instead of: result.unwrap()
let result = safe_unwrap!(result, "operation", "component")?;

// Instead of: option.unwrap_or_default()
let value = safe_unwrap_or_default!(option, "operation", "component");

// Instead of: option.unwrap_or(default)
let value = safe_unwrap_or!(option, default, "operation", "component");
```

## Configuration

### Error Handler Configuration

```rust
pub struct ErrorHandlerConfig {
    /// Enable/disable automatic recovery
    pub enable_automatic_recovery: bool,
    /// Maximum number of recovery attempts per error
    pub max_recovery_attempts: u32,
    /// Default timeout for recovery operations
    pub default_recovery_timeout: Duration,
    /// Enable/disable circuit breaker pattern
    pub enable_circuit_breakers: bool,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker recovery timeout
    pub circuit_breaker_timeout: Duration,
    /// Enable/disable health monitoring
    pub enable_health_monitoring: bool,
    /// Health check interval
    pub health_check_interval: Duration,
}
```

### Circuit Breaker Configuration

```rust
pub struct CircuitBreakerConfig {
    /// Failure threshold to open the circuit
    pub failure_threshold: u32,
    /// Success threshold to close the circuit from half-open
    pub success_threshold: u32,
    /// Timeout before transitioning from open to half-open
    pub timeout: Duration,
    /// Window size for tracking failures
    pub window_size: Duration,
}
```

## Health Monitoring

The system provides comprehensive health monitoring:

### Component Health Metrics

```rust
pub struct HealthMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub recovery_attempts: u64,
    pub successful_recoveries: u64,
    pub average_recovery_time: Duration,
    pub last_failure_time: Option<std::time::Instant>,
}
```

### System Health Score

Calculate overall system health as a score between 0.0 and 1.0:

```rust
let health_score = handler.get_system_health_score().await;
println!("System health: {:.2}", health_score);
```

## Error Classification

Errors are classified into categories for appropriate recovery strategies:

### Error Categories

- **Transient**: Temporary errors that should be retried (timeouts, network issues)
- **Permanent**: Permanent errors that should not be retried (validation errors, not found)
- **RateLimited**: Rate limiting errors that should be retried with backoff
- **Auth**: Authentication/authorization errors
- **ResourceExhausted**: Resource exhaustion errors
- **Configuration**: Configuration errors
- **Network**: Network connectivity errors
- **Unknown**: Unclassified errors

### Retry Configuration by Category

```rust
let retry_config = ErrorClassifier::get_retry_config(&category);
let should_retry = ErrorClassifier::should_retry(&category);
```

## Best Practices

### 1. Use Centralized Error Handling

Always use the centralized error handler instead of manual error handling:

```rust
// ✅ Good: Use centralized error handling
let result = handler.execute_with_centralized_handling(
    || Box::pin(async { operation() }),
    "operation_name",
    "ComponentName",
    None,
).await?;

// ❌ Bad: Manual error handling without recovery
match operation().await {
    Ok(result) => Ok(result),
    Err(e) => Err(e),
}
```

### 2. Provide Context

Always provide meaningful operation and component names:

```rust
// ✅ Good: Descriptive names
handler.execute_with_centralized_handling(
    || Box::pin(async { process_user_data(user_id) }),
    "process_user_data",
    "UserService",
    None,
).await?;

// ❌ Bad: Generic names
handler.execute_with_centralized_handling(
    || Box::pin(async { process_user_data(user_id) }),
    "op",
    "comp",
    None,
).await?;
```

### 3. Use Agent-Specific Handling for Agent Operations

For agent-related operations, always provide the agent ID:

```rust
// ✅ Good: Agent-specific handling
handler.execute_with_centralized_handling(
    || Box::pin(async { agent_learning_step(agent_id) }),
    "agent_learning",
    "NeuralAgent",
    Some(agent_id),
).await?;

// ❌ Bad: Generic handling for agent operation
handler.execute_with_centralized_handling(
    || Box::pin(async { agent_learning_step(agent_id) }),
    "learning",
    "Agent",
    None,
).await?;
```

### 4. Configure Appropriate Timeouts

Set appropriate timeouts for different operation types:

```rust
let config = ErrorHandlerConfig {
    default_recovery_timeout: Duration::from_secs(300), // 5 minutes for complex operations
    circuit_breaker_timeout: Duration::from_secs(60),   // 1 minute for circuit breaker
    ..Default::default()
};
```

### 5. Monitor Health Metrics

Regularly check health metrics to identify issues:

```rust
let health = handler.get_component_health("ComponentName").await;
if let Some(metrics) = health {
    let success_rate = metrics.successful_operations as f64 / metrics.total_operations as f64;
    if success_rate < 0.95 {
        warn!("Component success rate below 95%: {:.2}", success_rate);
    }
}
```

## Migration Guide

### Replacing `unwrap()` Calls

1. **Option unwrap()**:
   ```rust
   // Before
   let value = option.unwrap();
   
   // After
   let value = safe_unwrap!(option, "operation", "component")?;
   ```

2. **Result unwrap()**:
   ```rust
   // Before
   let value = result.unwrap();
   
   // After
   let value = safe_unwrap!(result, "operation", "component")?;
   ```

3. **expect() calls**:
   ```rust
   // Before
   let value = option.expect("This should not happen");
   
   // After
   let value = safe_unwrap!(option, "operation", "component")
       .map_err(|e| anyhow::anyhow!("Expected value: {}", e))?;
   ```

### Replacing Manual Error Handling

1. **Simple operations**:
   ```rust
   // Before
   match operation().await {
       Ok(result) => Ok(result),
       Err(e) => Err(HiveError::OperationFailed {
           reason: e.to_string(),
       }),
   }
   
   // After
   handle_with_centralized_error_recovery!(operation(), "operation", "Component")
   ```

2. **Agent operations**:
   ```rust
   // Before
   match agent_operation(agent_id).await {
       Ok(result) => Ok(result),
       Err(e) => Err(HiveError::AgentExecutionFailed {
           reason: e.to_string(),
       }),
   }
   
   // After
   handle_agent_with_centralized_error_recovery!(
       agent_operation(agent_id),
       "agent_operation",
       "AgentComponent",
       agent_id
   )
   ```

## Testing

The centralized error handling system includes comprehensive tests:

```rust
#[tokio::test]
async fn test_centralized_error_handler() {
    let handler = CentralizedErrorHandler::new(ErrorHandlerConfig::default());
    
    let result = handler.execute_with_centralized_handling(
        || Box::pin(async { Ok::<i32, &str>(42) }),
        "test",
        "TestComponent",
        None,
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}
```

Run tests with:
```bash
cargo test centralized_error_handling_tests
```

## Performance Considerations

### 1. Circuit Breaker Overhead
Circuit breakers add minimal overhead (typically < 1ms per operation).

### 2. Health Monitoring
Health monitoring is designed to be lightweight and asynchronous.

### 3. Memory Usage
The system uses bounded memory for storing health metrics and recovery history.

### 4. Concurrent Access
All components are designed for concurrent access using appropriate synchronization.

## Troubleshooting

### Common Issues

1. **Circuit Breaker Always Open**
   - Check if failure threshold is too low
   - Verify if timeout is appropriate for your use case
   - Monitor underlying service health

2. **High Recovery Failure Rate**
   - Review recovery strategy configuration
   - Check if error classification is appropriate
   - Monitor system resources

3. **Poor Health Scores**
   - Analyze failure patterns
   - Check if retry attempts are appropriate
   - Review timeout configurations

### Debug Logging

Enable debug logging for detailed error handling information:

```rust
tracing::info!("Error handler configuration: {:?}", handler.get_config());
tracing::debug!("Circuit breaker status: {:?}", handler.get_circuit_breaker_status("Component").await);
tracing::debug!("Component health: {:?}", handler.get_component_health("Component").await);
```

## Future Enhancements

### Planned Features

1. **Machine Learning-Based Recovery**
   - Predictive failure detection
   - Adaptive recovery strategy selection
   - Automated parameter tuning

2. **Distributed Error Handling**
   - Cross-node error coordination
   - Distributed circuit breakers
   - Global health monitoring

3. **Advanced Analytics**
   - Error pattern analysis
   - Performance impact assessment
   - Recovery optimization recommendations

### Integration Points

1. **Monitoring Systems**
   - Prometheus metrics export
   - Grafana dashboard integration
   - Alert system integration

2. **Logging Systems**
   - Structured logging integration
   - Error correlation IDs
   - Distributed tracing

3. **Configuration Management**
   - Dynamic configuration updates
   - Environment-specific settings
   - Feature flags for error handling

## Conclusion

The centralized error handling system provides a robust foundation for building reliable multi-agent applications. By following the patterns and best practices outlined in this document, you can ensure that your code handles errors gracefully, prevents production panics, and maintains system reliability even under failure conditions.

For questions or contributions, please refer to the project documentation or contact the development team.