# Testing Guide for Multiagent Hive System

This document provides comprehensive information about the test suite for the multiagent hive system.

## Test Structure

The test suite is organized into several modules:

### Unit Tests (`src/tests/`)

- **`agent_tests.rs`** - Tests for agent creation, capabilities, learning, and behavior
- **`task_tests.rs`** - Tests for task management, queuing, and execution
- **`hive_tests.rs`** - Tests for hive coordination, metrics, and system management
- **`neural_tests.rs`** - Tests for NLP processing and neural network integration
- **`integration_tests.rs`** - End-to-end integration tests
- **`test_utils.rs`** - Common test utilities and helper functions

### Integration Tests (`tests/`)

- **`comprehensive_test_suite.rs`** - Complete system test suite with performance benchmarks

## Running Tests

### Run All Tests
```bash
cd backend
cargo test
```

### Run Specific Test Modules
```bash
# Run only agent tests
cargo test agent_tests

# Run only hive coordination tests
cargo test hive_tests

# Run only neural processing tests
cargo test neural_tests

# Run integration tests
cargo test integration_tests

# Run comprehensive test suite
cargo test comprehensive_test_suite
```

### Run Tests with Features
```bash
# Run tests with advanced neural features
cargo test --features advanced-neural

# Run tests with all features
cargo test --all-features
```

### Run Tests with Output
```bash
# Show test output
cargo test -- --nocapture

# Show test output for specific test
cargo test test_hive_coordinator_creation -- --nocapture
```

## Test Categories

### 1. Agent System Tests

**Coverage:**
- Agent creation and initialization
- Agent types (Worker, Coordinator, Specialist, Learner)
- Capability management and scoring
- Learning from experiences
- Social connections and trust
- Task execution and fitness calculation
- Communication between agents
- Position updates and swarm behavior

**Key Test Cases:**
- `test_agent_creation()` - Basic agent initialization
- `test_agent_capability_management()` - Capability addition and scoring
- `test_agent_learning_from_experience()` - Learning mechanism
- `test_agent_can_perform_task()` - Task capability matching
- `test_agent_execute_task()` - Task execution workflow

### 2. Task Management Tests

**Coverage:**
- Task creation and configuration
- Priority-based queuing
- Task dependencies and execution readiness
- Task assignment and completion
- Task result handling
- Queue operations and metrics

**Key Test Cases:**
- `test_task_creation()` - Basic task initialization
- `test_task_queue_priority_ordering()` - Priority queue behavior
- `test_task_dependency_checking()` - Dependency resolution
- `test_task_queue_assignment()` - Task assignment workflow
- `test_task_result_creation()` - Result handling

### 3. Hive Coordination Tests

**Coverage:**
- Hive coordinator initialization
- Agent and task creation via API
- Metrics collection and reporting
- Resource management
- Simple verification system
- Auto-scaling and skill evolution
- WebSocket communication

**Key Test Cases:**
- `test_hive_coordinator_creation()` - System initialization
- `test_hive_agent_creation()` - Agent creation via coordinator
- `test_hive_task_creation()` - Task creation via coordinator
- `test_hive_metrics_and_monitoring()` - Metrics system
- `test_concurrent_operations()` - Concurrency handling

### 4. Neural Processing Tests

**Coverage:**
- NLP processor functionality
- Sentiment analysis
- Keyword extraction
- Semantic similarity calculation
- Pattern learning
- Hybrid neural processor
- Neural agent creation and coordination

**Key Test Cases:**
- `test_nlp_sentiment_analysis()` - Sentiment detection
- `test_nlp_keyword_extraction()` - Keyword identification
- `test_hybrid_neural_processor_creation()` - Neural system init
- `test_hybrid_neural_prediction()` - Performance prediction
- `test_hybrid_neural_learning()` - Neural learning

### 5. Integration Tests

**Coverage:**
- End-to-end task execution workflows
- Multi-agent coordination
- Swarm behavior and positioning
- Neural integration with agents
- Work-stealing queue integration
- Error recovery and resilience
- System scalability
- Concurrent operations

**Key Test Cases:**
- `test_end_to_end_task_execution()` - Complete workflow
- `test_multiple_agents_task_distribution()` - Multi-agent coordination
- `test_swarm_coordination_and_positioning()` - Swarm behavior
- `test_system_scalability()` - Performance under load
- `test_error_recovery_and_resilience()` - Error handling

## Test Utilities

### Helper Functions (`test_utils.rs`)

- `create_test_agent()` - Creates agents with default configuration
- `create_test_task()` - Creates tasks with specified parameters
- `create_agent_config()` - Generates JSON configuration for agents
- `create_task_config()` - Generates JSON configuration for tasks
- `assert_approx_eq()` - Floating-point comparison with tolerance

### Mock Data Generation

The test utilities provide functions to create realistic test data:
- Agents with various types and capabilities
- Tasks with different priorities and requirements
- Configuration objects for API testing

## Performance Benchmarks

### Benchmark Tests

- **Agent Creation Performance** - Measures time to create multiple agents
- **Task Creation Performance** - Measures time to create multiple tasks
- **Memory Usage** - Monitors system memory consumption
- **Concurrent Operations** - Tests system under concurrent load

### Performance Targets

- Agent creation: < 5 seconds for 10 agents
- Task creation: < 5 seconds for 20 tasks
- Memory usage: < 90% system memory
- Concurrent operations: No deadlocks or race conditions

## Test Configuration

### Environment Variables

```bash
# Set log level for tests
export RUST_LOG=debug

# Set test timeout
export CARGO_TEST_TIMEOUT=300

# Enable backtrace on panic
export RUST_BACKTRACE=1
```

### Feature Flags

Tests automatically adapt to available features:
- `basic-nlp` (default) - Basic NLP processing tests
- `advanced-neural` - Advanced neural network tests
- `gpu-acceleration` - GPU-accelerated processing tests

## Continuous Integration

### GitHub Actions

The test suite runs automatically on:
- Pull requests
- Pushes to main branch
- Scheduled daily runs

### Test Matrix

Tests run across multiple configurations:
- Rust stable and nightly
- Different feature combinations
- Various operating systems (Linux, macOS, Windows)

## Debugging Tests

### Common Issues

1. **Async Test Timeouts**
   ```bash
   # Increase timeout for slow tests
   cargo test -- --test-threads=1
   ```

2. **Resource Cleanup**
   ```bash
   # Run tests sequentially to avoid resource conflicts
   cargo test -- --test-threads=1
   ```

3. **Feature Dependencies**
   ```bash
   # Ensure correct features are enabled
   cargo test --features advanced-neural
   ```

### Debug Output

```bash
# Enable debug logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Show test execution details
cargo test -- --nocapture --test-threads=1
```

## Adding New Tests

### Test Naming Convention

- Unit tests: `test_<component>_<functionality>()`
- Integration tests: `test_<workflow>_<scenario>()`
- Benchmark tests: `benchmark_<operation>()`

### Test Structure Template

```rust
#[tokio::test]
async fn test_new_functionality() {
    // Arrange
    let hive = HiveCoordinator::new().await.unwrap();

    // Act
    let result = hive.some_operation().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(expected_value, actual_value);
}
```

### Best Practices

1. **Use descriptive test names** that explain what is being tested
2. **Follow AAA pattern** (Arrange, Act, Assert)
3. **Test both success and failure cases**
4. **Use test utilities** for common setup
5. **Clean up resources** in async tests
6. **Add documentation** for complex test scenarios

## Test Coverage

### Coverage Report

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# View coverage report
open tarpaulin-report.html
```

### Coverage Targets

- **Unit Tests**: > 80% line coverage
- **Integration Tests**: > 70% feature coverage
- **Critical Paths**: 100% coverage for core functionality

## Troubleshooting

### Common Test Failures

1. **Timing Issues**: Increase wait times in integration tests
2. **Resource Conflicts**: Run tests sequentially
3. **Feature Mismatches**: Verify feature flags are correct
4. **Memory Issues**: Check for memory leaks in long-running tests

### Getting Help

- Check test output for specific error messages
- Review the test documentation for expected behavior
- Run tests with debug output for more information
- Consult the main README for system requirements
