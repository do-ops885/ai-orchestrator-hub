# Testing Guide for AI Orchestrator Hub Backend

This document provides comprehensive information about the testing strategy and practices for the AI Orchestrator Hub Backend, a Rust-based multi-agent system.

## Overview

The AI Orchestrator Hub Backend implements a sophisticated testing strategy that covers:

- **Unit Testing**: Individual component testing with high coverage
- **Integration Testing**: End-to-end workflow validation
- **Performance Testing**: Benchmarking and load testing
- **Chaos Engineering**: Resilience and fault tolerance testing
- **API Testing**: REST and WebSocket endpoint validation

## Test Structure

### Source Code Organization

```
src/
├── agents/              # Agent system implementation
│   ├── mod.rs
│   └── [agent modules]
├── api/                 # REST API handlers
├── communication/       # WebSocket and MCP protocols
├── core/                # Core orchestration logic
├── infrastructure/      # System infrastructure
├── neural/              # Neural processing
├── tasks/               # Task management
└── utils/               # Shared utilities
```

### Test Organization

```
tests/
├── api_integration_tests.rs      # API endpoint testing
├── chaos_engineering_tests.rs    # Fault injection testing
├── comprehensive_test_suite.rs   # Full system integration
├── performance_regression_tests.rs # Performance validation
└── standalone_chaos_tests.rs     # Isolated chaos testing

benches/
├── agent_benchmarks.rs           # Agent performance benchmarks
├── neural_benchmarks.rs          # Neural processing benchmarks
├── swarm_benchmarks.rs           # Swarm coordination benchmarks
└── [other benchmark files]
```

## Running Tests

### Prerequisites

Ensure you have the required dependencies:

```bash
# Install cargo-nextest for faster test execution
cargo install cargo-nextest

# Install cargo-tarpaulin for coverage reports
cargo install cargo-tarpaulin

# Install cargo-criterion for benchmarking
cargo install cargo-criterion
```

### Basic Test Execution

```bash
# Run all tests
cargo test

# Run tests with nextest (faster, better output)
cargo nextest run

# Run specific test
cargo test test_agent_creation

# Run tests in a specific module
cargo test agents::

# Run integration tests only
cargo test --test api_integration_tests

# Run with verbose output
cargo test -- --nocapture
```

### Feature-Specific Testing

```bash
# Test basic functionality (default)
cargo test

# Test with advanced neural features
cargo test --features advanced-neural

# Test with GPU acceleration
cargo test --features advanced-neural,gpu-acceleration

# Test all features
cargo test --all-features
```

### Performance Testing

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench agent_benchmarks

# Generate flame graphs (requires cargo-flamegraph)
cargo flamegraph --bench agent_benchmarks
```

## Test Categories

### 1. Unit Tests

Unit tests focus on individual components and functions.

#### Agent System Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[tokio::test]
    async fn test_agent_creation() {
        let config = create_test_agent_config("worker");
        let agent = Agent::new(config).await.unwrap();

        assert_eq!(agent.agent_type, AgentType::Worker);
        assert!(!agent.capabilities.is_empty());
    }

    #[tokio::test]
    async fn test_agent_capability_matching() {
        let agent = create_test_agent(vec!["data_processing"]).await;
        let task = create_test_task("data_processing");

        assert!(agent.can_perform_task(&task).await);
    }

    #[tokio::test]
    async fn test_agent_learning() {
        let mut agent = create_test_agent(vec!["data_processing"]).await;
        let initial_proficiency = agent.get_proficiency("data_processing");

        // Simulate successful task execution
        agent.record_experience("data_processing", 0.9).await;

        let updated_proficiency = agent.get_proficiency("data_processing");
        assert!(updated_proficiency > initial_proficiency);
    }
}
```

#### Task Management Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_creation() {
        let config = TaskConfig {
            description: "Process data".to_string(),
            task_type: TaskType::DataProcessing,
            priority: Priority::Medium,
            requirements: vec!["data_processing".to_string()],
        };

        let task = Task::new(config).await.unwrap();
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, Priority::Medium);
    }

    #[tokio::test]
    async fn test_task_queue_priority() {
        let mut queue = TaskQueue::new();

        let high_priority = create_test_task_with_priority(Priority::High);
        let low_priority = create_test_task_with_priority(Priority::Low);

        queue.add_task(high_priority).await;
        queue.add_task(low_priority).await;

        let next_task = queue.get_next_task().await.unwrap();
        assert_eq!(next_task.priority, Priority::High);
    }
}
```

#### Neural Processing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nlp_sentiment_analysis() {
        let processor = HybridNeuralProcessor::new().await.unwrap();

        let text = "This product is excellent and works perfectly!";
        let sentiment = processor.analyze_sentiment(text).await.unwrap();

        assert!(sentiment.score > 0.5);
        assert_eq!(sentiment.label, Sentiment::Positive);
    }

    #[tokio::test]
    async fn test_pattern_recognition() {
        let processor = HybridNeuralProcessor::new().await.unwrap();

        let patterns = vec![
            "user logged in",
            "user viewed page",
            "user made purchase"
        ];

        let analysis = processor.find_patterns(patterns).await.unwrap();
        assert!(!analysis.patterns.is_empty());
    }
}
```

### 2. Integration Tests

Integration tests validate complete workflows and component interactions.

#### API Integration Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    #[tokio::test]
    async fn test_agent_creation_workflow() {
        let client = Client::new();
        let agent_config = serde_json::json!({
            "name": "TestAgent",
            "type": "worker",
            "capabilities": [
                {
                    "name": "data_processing",
                    "proficiency": 0.8
                }
            ]
        });

        // Create agent
        let response = client
            .post("http://localhost:3001/api/agents")
            .json(&agent_config)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let result: serde_json::Value = response.json().await.unwrap();
        assert!(result["success"].as_bool().unwrap());

        let agent_id = result["data"]["agent_id"].as_str().unwrap();

        // Verify agent was created
        let response = client
            .get("http://localhost:3001/api/agents")
            .send()
            .await
            .unwrap();

        let agents: serde_json::Value = response.json().await.unwrap();
        let agent_exists = agents["data"]["agents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["id"] == agent_id);

        assert!(agent_exists);
    }
}
```

#### End-to-End Workflow Tests

```rust
#[tokio::test]
async fn test_complete_task_workflow() {
    // Setup
    let hive = HiveCoordinator::new().await.unwrap();

    // Create agent
    let agent_config = create_agent_config("worker", vec!["data_processing"]);
    let agent_id = hive.create_agent(agent_config).await.unwrap();

    // Create task
    let task_config = create_task_config("Process data", "data_processing");
    let task_id = hive.create_task(task_config).await.unwrap();

    // Wait for task completion
    let mut attempts = 0;
    loop {
        let status = hive.get_task_status(&task_id).await.unwrap();
        if status == TaskStatus::Completed {
            break;
        }
        attempts += 1;
        if attempts > 10 {
            panic!("Task did not complete within timeout");
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Verify results
    let task = hive.get_task(&task_id).await.unwrap();
    assert!(task.result.is_some());
    assert!(task.execution_time_ms > 0);
}
```

### 3. Performance Tests

Performance tests ensure the system meets performance requirements.

#### Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_agent_creation(c: &mut Criterion) {
    c.bench_function("create_agent", |b| {
        b.iter(|| {
            let config = create_test_agent_config("worker");
            black_box(Agent::new(config))
        })
    });
}

fn benchmark_task_assignment(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_assignment");

    for num_agents in [10, 50, 100].iter() {
        group.bench_with_input(
            format!("{} agents", num_agents),
            num_agents,
            |b, &num_agents| {
                b.iter(|| {
                    let agents = create_multiple_agents(num_agents);
                    let task = create_test_task("data_processing");
                    black_box(assign_task_to_best_agent(&task, &agents))
                })
            }
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_agent_creation, benchmark_task_assignment);
criterion_main!(benches);
```

#### Load Testing

```rust
#[tokio::test]
async fn test_concurrent_agent_operations() {
    let num_operations = 100;
    let mut handles = vec![];

    for i in 0..num_operations {
        let handle = tokio::spawn(async move {
            let agent_config = create_agent_config(
                &format!("ConcurrentAgent-{}", i),
                vec!["data_processing"]
            );

            let hive = get_shared_hive_instance().await;
            hive.create_agent(agent_config).await
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;

    // Verify all operations succeeded
    let success_count = results.iter()
        .filter(|result| result.is_ok())
        .count();

    assert_eq!(success_count, num_operations);
}
```

### 4. Chaos Engineering Tests

Chaos engineering tests validate system resilience.

#### Fault Injection Tests

```rust
#[tokio::test]
async fn test_agent_failure_recovery() {
    let hive = HiveCoordinator::new().await.unwrap();

    // Create multiple agents
    let agent_ids = create_multiple_agents(&hive, 5).await;

    // Simulate agent failure
    hive.simulate_agent_failure(&agent_ids[0]).await;

    // Create task that requires failed agent's capabilities
    let task_config = create_task_config("data_processing", "data_processing");
    let task_id = hive.create_task(task_config).await.unwrap();

    // Verify task is reassigned to another agent
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let task = hive.get_task(&task_id).await.unwrap();
    assert_ne!(task.assigned_agent, Some(agent_ids[0]));
    assert!(task.assigned_agent.is_some());
}
```

#### Network Failure Tests

```rust
#[tokio::test]
async fn test_network_partition_recovery() {
    let hive = HiveCoordinator::new().await.unwrap();

    // Create distributed agents
    let agent_ids = create_distributed_agents(&hive, 10).await;

    // Simulate network partition
    hive.simulate_network_partition(vec![agent_ids[0], agent_ids[1]]).await;

    // Create tasks
    let task_ids = create_multiple_tasks(&hive, 5).await;

    // Wait for recovery
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Verify all tasks completed despite partition
    for task_id in task_ids {
        let task = hive.get_task(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
    }
}
```

## Test Utilities

### Helper Functions

```rust
// test_utils.rs
use crate::agents::{Agent, AgentConfig};
use crate::tasks::{Task, TaskConfig};

pub fn create_test_agent_config(agent_type: &str) -> AgentConfig {
    AgentConfig {
        name: format!("Test{}", agent_type),
        agent_type: agent_type.to_string(),
        capabilities: vec!["general".to_string()],
        ..Default::default()
    }
}

pub async fn create_test_agent(capabilities: Vec<&str>) -> Agent {
    let config = AgentConfig {
        name: "TestAgent".to_string(),
        agent_type: "worker".to_string(),
        capabilities: capabilities.into_iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    };

    Agent::new(config).await.unwrap()
}

pub fn create_test_task_config(description: &str, task_type: &str) -> TaskConfig {
    TaskConfig {
        description: description.to_string(),
        task_type: task_type.to_string(),
        priority: Priority::Medium,
        ..Default::default()
    }
}

pub async fn create_test_task(task_type: &str) -> Task {
    let config = create_test_task_config("Test task", task_type);
    Task::new(config).await.unwrap()
}
```

### Mock Data Generation

```rust
pub fn generate_realistic_agent_configs(count: usize) -> Vec<AgentConfig> {
    let agent_types = ["worker", "specialist", "coordinator"];
    let capabilities = [
        "data_processing", "analysis", "communication",
        "coordination", "learning", "optimization"
    ];

    (0..count).map(|i| {
        let agent_type = agent_types[i % agent_types.len()];
        let mut agent_capabilities = vec![];

        // Add 2-4 random capabilities
        for _ in 0..(2 + (i % 3)) {
            let capability = capabilities[i % capabilities.len()];
            agent_capabilities.push(capability.to_string());
        }

        AgentConfig {
            name: format!("Agent-{}", i),
            agent_type: agent_type.to_string(),
            capabilities: agent_capabilities,
            proficiency_levels: (0..agent_capabilities.len())
                .map(|_| 0.5 + (i as f64 * 0.1) % 0.4)
                .collect(),
            ..Default::default()
        }
    }).collect()
}
```

## Test Configuration

### Environment Variables

```bash
# Test settings
export RUST_TEST_THREADS=4
export CARGO_TEST_TIMEOUT=300
export RUST_BACKTRACE=1

# Logging
export RUST_LOG=debug,hive=trace

# Feature flags
export HIVE_TEST_MODE=true
export HIVE_DISABLE_METRICS=false
```

### Test-Specific Configuration

```toml
# tests/config/test_config.toml
[agents]
max_test_agents = 50
default_test_timeout = 30

[tasks]
max_concurrent_test_tasks = 20
test_task_timeout = 60

[neural]
test_model_path = "tests/models/"
enable_test_caching = true

[performance]
benchmark_iterations = 1000
warmup_iterations = 100
```

## Continuous Integration

### GitHub Actions Configuration

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta]

    steps:
    - uses: actions/checkout@v3

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}

    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Run tests
      run: cargo test --all-features

    - name: Run benchmarks
      run: cargo bench

    - name: Generate coverage
      run: cargo tarpaulin --out Xml

    - name: Upload coverage
      uses: codecov/codecov-action@v3
      with:
        file: cobertura.xml
```

### Test Matrix

The CI pipeline tests across multiple configurations:

- **Rust Versions**: stable, beta, nightly
- **Operating Systems**: Ubuntu, macOS, Windows
- **Feature Combinations**:
  - Default features
  - Advanced neural features
  - GPU acceleration
  - All features combined

## Code Coverage

### Coverage Targets

- **Unit Tests**: > 85% line coverage
- **Integration Tests**: > 75% feature coverage
- **Critical Paths**: 100% coverage for core functionality
- **API Endpoints**: 100% coverage for all endpoints

### Coverage Reporting

```bash
# Generate HTML coverage report
cargo tarpaulin --out Html

# Generate lcov format for CI
cargo tarpaulin --out Lcov

# Exclude specific files/patterns
cargo tarpaulin --exclude-files "src/bin/*" --out Html
```

### Coverage Analysis

```bash
# View coverage by crate
cargo tarpaulin --all-features --out Json | jq '.crates[] | {name: .name, coverage: .coverage}'

# Identify uncovered lines
cargo tarpaulin --out Json | jq '.files[] | select(.coverage < 80) | {file: .name, coverage: .coverage}'
```

## Debugging Tests

### Common Issues and Solutions

#### Async Test Timeouts

```rust
#[tokio::test]
async fn test_slow_operation() {
    // Increase timeout for slow operations
    tokio::time::timeout(
        std::time::Duration::from_secs(30),
        slow_operation()
    ).await.expect("Operation timed out");
}
```

#### Resource Conflicts

```bash
# Run tests sequentially
cargo test -- --test-threads=1

# Use unique resource names
let db_name = format!("test_db_{}", uuid::Uuid::new_v4());
```

#### Feature Dependencies

```rust
#[cfg(feature = "advanced-neural")]
#[tokio::test]
async fn test_neural_features() {
    // Test code that requires advanced-neural feature
}

#[cfg(not(feature = "advanced-neural"))]
#[tokio::test]
async fn test_basic_features() {
    // Test code for basic functionality
}
```

### Debug Output

```bash
# Enable detailed logging
RUST_LOG=trace cargo test test_name -- --nocapture

# Show backtraces
RUST_BACKTRACE=1 cargo test test_name

# Run single test with output
cargo test test_name -- --nocapture --test-threads=1
```

## Performance Testing Best Practices

### Benchmarking Guidelines

1. **Warm-up Phase**: Always include warm-up iterations
2. **Statistical Significance**: Run enough iterations for reliable results
3. **Baseline Comparison**: Compare against known good baselines
4. **Realistic Data**: Use production-like test data

### Performance Regression Detection

```rust
#[tokio::test]
async fn test_performance_regression() {
    let baseline = std::time::Duration::from_millis(100);
    let start = std::time::Instant::now();

    // Perform operation
    let result = perform_operation().await;

    let elapsed = start.elapsed();

    // Allow 10% variance from baseline
    let max_allowed = baseline.mul_f64(1.1);
    assert!(elapsed <= max_allowed,
        "Performance regression detected: {:?} > {:?}",
        elapsed, max_allowed);
}
```

## Test Organization Best Practices

### Test File Structure

```
tests/
├── unit/
│   ├── agents.rs
│   ├── tasks.rs
│   └── neural.rs
├── integration/
│   ├── api.rs
│   ├── workflows.rs
│   └── performance.rs
├── chaos/
│   ├── fault_injection.rs
│   ├── network_failures.rs
│   └── recovery.rs
└── utils/
    ├── test_helpers.rs
    └── mock_data.rs
```

### Test Naming Conventions

- `test_<component>_<action>_<condition>()`
- `test_<workflow>_<scenario>()`
- `benchmark_<operation>_<variant>()`

Examples:
- `test_agent_creation_success()`
- `test_task_assignment_with_insufficient_capabilities()`
- `test_end_to_end_workflow_complex()`
- `benchmark_agent_creation_100_agents()`

## Troubleshooting

### Test Failures

#### Timing Issues
- Increase timeouts for slow operations
- Use exponential backoff for retries
- Add delays between dependent operations

#### Resource Issues
- Ensure proper cleanup in test teardown
- Use unique identifiers for resources
- Limit concurrent test execution

#### Environment Differences
- Use environment-agnostic test data
- Mock external dependencies
- Test with multiple configurations

### Getting Help

1. **Check Test Output**: Look for specific error messages
2. **Review Logs**: Enable debug logging for more information
3. **Isolate Issues**: Run failing tests individually
4. **Check Dependencies**: Verify all required services are running
5. **Update Dependencies**: Ensure all crates are up to date

## Future Enhancements

### Planned Test Improvements

- **Property-Based Testing**: Use proptest for comprehensive input testing
- **Fuzz Testing**: Implement fuzzing for critical components
- **Load Testing**: Add distributed load testing capabilities
- **Performance Profiling**: Integrate continuous performance monitoring
- **Mutation Testing**: Validate test suite effectiveness

### Test Automation

- **Scheduled Test Runs**: Daily comprehensive test execution
- **Performance Baselines**: Automated baseline updates
- **Coverage Enforcement**: CI pipeline coverage requirements
- **Test Result Analytics**: Historical test performance tracking

This testing guide provides a comprehensive framework for maintaining high-quality, reliable code in the AI Orchestrator Hub Backend. Following these practices ensures robust, performant, and maintainable software.