//! Test utilities and common fixtures for the multiagent hive system tests

use crate::agents::{Agent, AgentCapability, AgentType};
use crate::tasks::{Task, TaskPriority, TaskRequiredCapability};

/// Creates a test agent with default configuration
#[must_use]
pub fn create_test_agent(name: &str, agent_type: AgentType) -> Agent {
    let mut agent = Agent::new(name.to_string(), agent_type);

    // Add some basic capabilities
    agent.add_capability(AgentCapability {
        name: "general".to_string(),
        proficiency: 0.7,
        learning_rate: 0.1,
    });

    agent.add_capability(AgentCapability {
        name: "communication".to_string(),
        proficiency: 0.6,
        learning_rate: 0.15,
    });

    agent
}

/// Creates a test agent with specific capabilities
#[must_use]
pub fn create_test_agent_with_capabilities(
    name: &str,
    agent_type: AgentType,
    capabilities: Vec<AgentCapability>,
) -> Agent {
    let mut agent = Agent::new(name.to_string(), agent_type);

    for capability in capabilities {
        agent.add_capability(capability);
    }

    agent
}

/// Creates a test task with default configuration
#[must_use]
pub fn create_test_task(description: &str, task_type: &str, priority: TaskPriority) -> Task {
    Task::new(
        description.to_string(),
        description.to_string(),
        task_type.to_string(),
        priority,
        vec![],
    )
}

/// Creates a test task with required capabilities
#[must_use]
pub fn create_test_task_with_requirements(
    description: &str,
    task_type: &str,
    priority: TaskPriority,
    required_capabilities: Vec<TaskRequiredCapability>,
) -> Task {
    Task::new(
        description.to_string(),
        description.to_string(),
        task_type.to_string(),
        priority,
        required_capabilities,
    )
}

/// Creates a test capability
#[must_use]
pub fn create_test_capability(name: &str, proficiency: f64, learning_rate: f64) -> AgentCapability {
    AgentCapability {
        name: name.to_string(),
        proficiency: proficiency.clamp(0.0, 1.0),
        learning_rate: learning_rate.clamp(0.0, 1.0),
    }
}

/// Creates a test required capability
#[must_use]
pub fn create_test_required_capability(name: &str, min_proficiency: f64) -> TaskRequiredCapability {
    TaskRequiredCapability {
        name: name.to_string(),
        minimum_proficiency: min_proficiency.clamp(0.0, 1.0),
    }
}

/// Asserts that two floating point numbers are approximately equal
pub fn assert_approx_eq(a: f64, b: f64, tolerance: f64) {
    assert!(
        (a - b).abs() < tolerance,
        "Values not approximately equal: {a} vs {b} (tolerance: {tolerance})"
    );
}

/// Creates a mock JSON configuration for agent creation
#[must_use]
pub fn create_agent_config(
    name: &str,
    agent_type: &str,
    capabilities: Option<Vec<(&str, f64, f64)>>,
) -> serde_json::Value {
    let mut config = serde_json::json!({
        "name": name,
        "type": agent_type
    });

    if let Some(caps) = capabilities {
        let capabilities_json: Vec<serde_json::Value> = caps
            .into_iter()
            .map(|(name, proficiency, learning_rate)| {
                serde_json::json!({
                    "name": name,
                    "proficiency": proficiency,
                    "learning_rate": learning_rate
                })
            })
            .collect();

        config["capabilities"] = serde_json::Value::Array(capabilities_json);
    }

    config
}

/// Creates a mock JSON configuration for task creation
#[must_use]
pub fn create_task_config(
    description: &str,
    task_type: &str,
    priority: u64,
    required_capabilities: Option<Vec<(&str, f64)>>,
) -> serde_json::Value {
    let mut config = serde_json::json!({
        "description": description,
        "type": task_type,
        "priority": priority
    });

    if let Some(req_caps) = required_capabilities {
        let capabilities_json: Vec<serde_json::Value> = req_caps
            .into_iter()
            .map(|(name, min_proficiency)| {
                serde_json::json!({
                    "name": name,
                    "min_proficiency": min_proficiency
                })
            })
            .collect();

        config["required_capabilities"] = serde_json::Value::Array(capabilities_json);
    }

    config
}

/// Create a test task result
#[must_use]
pub fn create_test_task_result(success: bool) -> crate::tasks::TaskResult {
    crate::tasks::TaskResult {
        task_id: uuid::Uuid::new_v4(),
        agent_id: uuid::Uuid::new_v4(),
        success,
        output: if success {
            "Task completed successfully".to_string()
        } else {
            "Task failed".to_string()
        },
        execution_time: 1000,
        error_message: if success {
            None
        } else {
            Some("Execution error".to_string())
        },
        completed_at: chrono::Utc::now(),
        quality_score: Some(if success { 0.9 } else { 0.3 }),
        learned_insights: vec![],
    }
}

/// Assert that a result is an error with specific message pattern
pub fn assert_error_contains<T>(result: Result<T, impl std::fmt::Display>, pattern: &str) {
    match result {
        Ok(_) => panic!("Expected error but got success"),
        Err(e) => {
            let error_str = format!("{e}");
            assert!(
                error_str.contains(pattern),
                "Error '{error_str}' does not contain pattern '{pattern}'"
            );
        }
    }
}

/// Assert that a result is a success
pub fn assert_success<T>(result: Result<T, impl std::fmt::Display>) -> T {
    match result {
        Ok(value) => value,
        Err(e) => panic!("Expected success but got error: {e}"),
    }
}

/// Create a mock environment variable for testing
pub struct MockEnvVar {
    key: String,
    original_value: Option<String>,
}

impl MockEnvVar {
    /// Create a new mock environment variable
    #[must_use] 
    pub fn new(key: &str, value: &str) -> Self {
        let original_value = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self {
            key: key.to_string(),
            original_value,
        }
    }
}

impl Drop for MockEnvVar {
    /// Restore the original environment variable value
    fn drop(&mut self) {
        if let Some(original) = &self.original_value {
            std::env::set_var(&self.key, original);
        } else {
            std::env::remove_var(&self.key);
        }
    }
}

/// Test fixture for setting up and tearing down test state
#[derive(Default)]
pub struct TestFixture {
    setup_actions: Vec<Box<dyn FnOnce()>>,
    teardown_actions: Vec<Box<dyn FnOnce()>>,
}

impl TestFixture {
    /// Create a new test fixture
    #[must_use] 
    pub fn new() -> Self {
        Self {
            setup_actions: Vec::new(),
            teardown_actions: Vec::new(),
        }
    }

    /// Add a setup action
    pub fn add_setup<F>(mut self, action: F) -> Self
    where
        F: FnOnce() + 'static,
    {
        self.setup_actions.push(Box::new(action));
        self
    }

    /// Add a teardown action
    pub fn add_teardown<F>(mut self, action: F) -> Self
    where
        F: FnOnce() + 'static,
    {
        self.teardown_actions.push(Box::new(action));
        self
    }

    /// Execute setup actions
    pub fn setup(&mut self) {
        for action in self.setup_actions.drain(..) {
            action();
        }
    }

    /// Execute teardown actions
    pub fn teardown(&mut self) {
        for action in self.teardown_actions.drain(..) {
            action();
        }
    }
}

/// Helper for timing test execution
pub struct TestTimer {
    start_time: std::time::Instant,
}

impl TestTimer {
    /// Start timing
    #[must_use] 
    pub fn start() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds
    #[must_use] 
    pub fn elapsed_ms(&self) -> u128 {
        self.start_time.elapsed().as_millis()
    }

    /// Assert that elapsed time is within bounds
    pub fn assert_within_bounds(&self, min_ms: u128, max_ms: u128) {
        let elapsed = self.elapsed_ms();
        assert!(
            elapsed >= min_ms && elapsed <= max_ms,
            "Elapsed time {elapsed}ms not within bounds [{min_ms}, {max_ms}]ms"
        );
    }
}

/// Helper for generating test data
pub struct TestDataGenerator {
    counter: u64,
}

impl TestDataGenerator {
    /// Create a new test data generator
    #[must_use] 
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    /// Generate a unique string
    pub fn unique_string(&mut self, prefix: &str) -> String {
        self.counter += 1;
        format!("{}_{}", prefix, self.counter)
    }

    /// Generate a unique UUID
    pub fn unique_uuid(&mut self) -> uuid::Uuid {
        uuid::Uuid::new_v4()
    }

    /// Generate a sequence of numbers
    pub fn sequence(&mut self, count: usize) -> Vec<u64> {
        let start = self.counter;
        self.counter += count as u64;
        (start..self.counter).collect()
    }
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for concurrent test execution
pub struct ConcurrentTestRunner {
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl ConcurrentTestRunner {
    /// Create a new concurrent test runner
    #[must_use] 
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
        }
    }

    /// Spawn a concurrent task
    pub fn spawn<F>(&mut self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        self.handles.push(handle);
    }

    /// Wait for all tasks to complete
    pub async fn wait_all(&mut self) {
        for handle in self.handles.drain(..) {
            let _ = handle.await;
        }
    }
}

impl Default for ConcurrentTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Assert that a collection contains an item
pub fn assert_contains<T: std::fmt::Debug + PartialEq>(collection: &[T], item: &T) {
    assert!(
        collection.contains(item),
        "Collection {collection:?} does not contain item {item:?}"
    );
}

/// Assert that a collection does not contain an item
pub fn assert_not_contains<T: std::fmt::Debug + PartialEq>(collection: &[T], item: &T) {
    assert!(
        !collection.contains(item),
        "Collection {collection:?} unexpectedly contains item {item:?}"
    );
}

/// Safe unwrap with detailed error message
pub fn safe_unwrap<T>(option: Option<T>, context: &str) -> T {
    option.unwrap_or_else(|| panic!("Expected Some value in {context} but got None"))
}

/// Safe expect with context
pub fn safe_expect<T>(result: Result<T, impl std::fmt::Display>, context: &str) -> T {
    result.unwrap_or_else(|e| panic!("Expected Ok value in {context} but got error: {e}"))
}
