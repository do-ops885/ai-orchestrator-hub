//! Comprehensive test utilities for the multiagent hive
//!
//! This module provides shared utilities, mocks, and helpers for testing
//! across the entire codebase.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use multiagent_hive::agents::{Agent, AgentConfig, AgentStatus};
use multiagent_hive::communication::{Message, MessageType};
use multiagent_hive::persistence::{PersistenceManager, StorageBackend};
use multiagent_hive::swarm::{SwarmConfig, SwarmCoordinator};

/// Test fixture for creating mock agents
pub struct MockAgent {
    pub id: Uuid,
    pub config: AgentConfig,
    pub status: AgentStatus,
}

impl MockAgent {
    pub fn new(agent_type: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            config: AgentConfig {
                name: format!("test-{}-agent", agent_type),
                agent_type: agent_type.to_string(),
                capabilities: vec!["test".to_string()],
                max_concurrent_tasks: 5,
                memory_limit_mb: 100,
                timeout_seconds: 30,
                retry_attempts: 3,
                specialization: Some(format!("test-{}", agent_type)),
                metadata: HashMap::new(),
            },
            status: AgentStatus::Idle,
        }
    }

    pub fn with_capabilities(mut self, capabilities: Vec<&str>) -> Self {
        self.config.capabilities = capabilities.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_memory_limit(mut self, limit: u32) -> Self {
        self.config.memory_limit_mb = limit;
        self
    }
}

/// Test fixture for creating mock messages
pub struct MockMessage {
    pub id: Uuid,
    pub message_type: MessageType,
    pub sender: Uuid,
    pub recipient: Option<Uuid>,
    pub payload: serde_json::Value,
}

impl MockMessage {
    pub fn new(sender: Uuid, message_type: MessageType) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type,
            sender,
            recipient: None,
            payload: serde_json::json!({"test": true}),
        }
    }

    pub fn to(recipient: Uuid) -> Self {
        let mut msg = Self::new(Uuid::new_v4(), MessageType::Task);
        msg.recipient = Some(recipient);
        msg
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }
}

/// Mock storage backend for testing
pub struct MockStorage {
    pub data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_data(&self, key: &str) -> Option<Vec<u8>> {
        self.data.read().await.get(key).cloned()
    }

    pub async fn set_data(&self, key: String, value: Vec<u8>) {
        self.data.write().await.insert(key, value);
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Test fixture for creating mock swarm configurations
pub struct MockSwarmConfig {
    pub config: SwarmConfig,
}

impl MockSwarmConfig {
    pub fn new() -> Self {
        Self {
            config: SwarmConfig {
                max_agents: 10,
                task_timeout_seconds: 300,
                coordination_strategy: "round_robin".to_string(),
                enable_load_balancing: true,
                health_check_interval_seconds: 30,
                auto_scaling_enabled: false,
                min_agents: 1,
                max_agents_per_task: 3,
                metadata: HashMap::new(),
            },
        }
    }

    pub fn with_max_agents(mut self, max: usize) -> Self {
        self.config.max_agents = max;
        self
    }

    pub fn with_auto_scaling(mut self, enabled: bool) -> Self {
        self.config.auto_scaling_enabled = enabled;
        self
    }
}

/// Test utilities for async operations
pub mod async_utils {
    use std::future::Future;
    use tokio::time::{timeout, Duration};

    /// Run a future with a timeout
    pub async fn with_timeout<T, F>(
        future: F,
        duration: Duration,
    ) -> Result<T, tokio::time::error::Elapsed>
    where
        F: Future<Output = T>,
    {
        timeout(duration, future).await
    }

    /// Wait for a condition to be true with timeout
    pub async fn wait_for_condition<F>(
        mut condition: F,
        timeout_duration: Duration,
        check_interval: Duration,
    ) -> Result<(), tokio::time::error::Elapsed>
    where
        F: FnMut() -> bool,
    {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout_duration {
            if condition() {
                return Ok(());
            }
            tokio::time::sleep(check_interval).await;
        }
        Err(tokio::time::error::Elapsed::new(timeout_duration))
    }
}

/// Test utilities for generating test data
pub mod test_data {
    use fake::{Fake, Faker};
    use uuid::Uuid;

    /// Generate a random agent name
    pub fn random_agent_name() -> String {
        format!("agent-{}", Faker.fake::<String>())
    }

    /// Generate a random task ID
    pub fn random_task_id() -> Uuid {
        Uuid::new_v4()
    }

    /// Generate random test data of specified size
    pub fn random_bytes(size: usize) -> Vec<u8> {
        (0..size).map(|_| rand::random::<u8>()).collect()
    }

    /// Generate a random JSON payload
    pub fn random_json() -> serde_json::Value {
        serde_json::json!({
            "id": random_task_id().to_string(),
            "data": Faker.fake::<String>(),
            "timestamp": chrono::Utc::now().timestamp(),
        })
    }
}

/// Test assertions and helpers
pub mod assertions {
    use std::fmt::Debug;

    /// Assert that two values are approximately equal within a tolerance
    pub fn assert_approx_eq<T>(left: T, right: T, tolerance: T, message: &str)
    where
        T: std::ops::Sub<Output = T> + std::cmp::PartialOrd + Debug + Copy,
    {
        let diff = if left > right {
            left - right
        } else {
            right - left
        };
        assert!(
            diff <= tolerance,
            "{}: {:?} vs {:?} (diff: {:?})",
            message,
            left,
            right,
            diff
        );
    }

    /// Assert that a future completes within a timeout
    pub async fn assert_completes_within<T, F>(future: F, timeout_ms: u64, message: &str) -> T
    where
        F: std::future::Future<Output = T>,
    {
        match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), future).await {
            Ok(result) => result,
            Err(_) => panic!(
                "{}: Future did not complete within {}ms",
                message, timeout_ms
            ),
        }
    }

    /// Assert that a collection contains at least one item matching a predicate
    pub fn assert_contains<T, F>(collection: &[T], predicate: F, message: &str)
    where
        F: Fn(&T) -> bool,
        T: Debug,
    {
        let found = collection.iter().any(predicate);
        assert!(
            found,
            "{}: No item in {:?} matches the predicate",
            message, collection
        );
    }
}

/// Test database utilities
pub mod db_utils {
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Create a temporary database file
    pub fn temp_db_path() -> (TempDir, PathBuf) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        (temp_dir, db_path)
    }

    /// Clean up test database
    pub async fn cleanup_db(path: &std::path::Path) {
        if path.exists() {
            tokio::fs::remove_file(path).await.ok();
        }
    }
}

/// Integration test helpers
pub mod integration {
    use std::process::Command;

    /// Start a test server process
    pub fn start_test_server(port: u16) -> std::process::Child {
        Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "mcp_server",
                "--",
                "--port",
                &port.to_string(),
            ])
            .spawn()
            .expect("Failed to start test server")
    }

    /// Wait for server to be ready
    pub async fn wait_for_server(port: u16, timeout_ms: u64) {
        let start = std::time::Instant::now();
        while start.elapsed().as_millis() < timeout_ms as u128 {
            if tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
                .await
                .is_err()
            {
                // Port is in use, server might be ready
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        panic!("Server did not start within {}ms", timeout_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_storage() {
        let storage = MockStorage::new();
        let key = "test_key".to_string();
        let data = b"test_data".to_vec();

        storage.set_data(key.clone(), data.clone()).await;
        let retrieved = storage.get_data(&key).await;

        assert_eq!(retrieved, Some(data));
    }

    #[test]
    fn test_mock_agent_creation() {
        let agent = MockAgent::new("worker");
        assert_eq!(agent.config.agent_type, "worker");
        assert!(agent.config.capabilities.contains(&"test".to_string()));
    }

    #[test]
    fn test_mock_message_creation() {
        let sender = Uuid::new_v4();
        let message = MockMessage::new(sender, MessageType::Task);
        assert_eq!(message.sender, sender);
        assert_eq!(message.message_type, MessageType::Task);
    }

    #[test]
    fn test_test_data_generation() {
        let name = test_data::random_agent_name();
        assert!(!name.is_empty());

        let id = test_data::random_task_id();
        assert_ne!(id, Uuid::nil());

        let bytes = test_data::random_bytes(10);
        assert_eq!(bytes.len(), 10);
    }
}
