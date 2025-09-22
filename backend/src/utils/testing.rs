use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    agents::AgentType,
    core::HiveCoordinator,
    infrastructure::{cache::CacheManager, MetricsCollector, TelemetryCollector},
    tasks::TaskPriority,
    utils::HiveConfig,
};

/// Test utilities and fixtures for the hive system
pub struct TestHarness {
    pub hive: Arc<RwLock<HiveCoordinator>>,
    pub config: Arc<HiveConfig>,
    pub metrics: Arc<MetricsCollector>,
    pub cache: Arc<CacheManager>,
    pub telemetry: Arc<TelemetryCollector>,
}

impl TestHarness {
    /// Create a new test harness with default configuration
    pub async fn new() -> anyhow::Result<Self> {
        let config = Arc::new(HiveConfig::default());
        let metrics = Arc::new(MetricsCollector::new(100));
        let cache = Arc::new(CacheManager::new());
        let telemetry = Arc::new(TelemetryCollector::new(1000));
        let hive = Arc::new(RwLock::new(HiveCoordinator::new().await?));

        Ok(Self {
            hive,
            config,
            metrics,
            cache,
            telemetry,
        })
    }

    /// Create test agents with various configurations
    pub async fn create_test_agents(&self, count: usize) -> anyhow::Result<Vec<Uuid>> {
        let mut agent_ids = Vec::new();
        let hive = self.hive.write().await;

        for i in 0..count {
            let agent_type = match i % 4 {
                0 => AgentType::Worker,
                1 => AgentType::Coordinator,
                2 => AgentType::Learner,
                _ => AgentType::Specialist(format!("test_specialist_{i}")),
            };

            let payload = json!({
                "name": format!("test_agent_{}", i),
                "agent_type": format!("{:?}", agent_type),
                "capabilities": [
                    {
                        "name": "test_capability",
                        "proficiency": 0.8,
                        "learning_rate": 0.1
                    }
                ]
            });

            let agent_id = hive.create_agent(payload).await?;
            agent_ids.push(agent_id);
        }

        Ok(agent_ids)
    }

    /// Create test tasks with various priorities
    pub async fn create_test_tasks(&self, count: usize) -> anyhow::Result<Vec<Uuid>> {
        let mut task_ids = Vec::new();
        let hive = self.hive.write().await;

        for i in 0..count {
            let priority = match i % 4 {
                0 => TaskPriority::Low,
                1 => TaskPriority::Medium,
                2 => TaskPriority::High,
                _ => TaskPriority::Critical,
            };

            let payload = json!({
                "description": format!("test_task_{}", i),
                "priority": format!("{:?}", priority),
                "required_capabilities": [
                    {
                        "name": "test_capability",
                        "minimum_proficiency": 0.5
                    }
                ]
            });

            let task_id = hive.create_task(payload).await?;
            task_ids.push(task_id);
        }

        Ok(task_ids)
    }

    /// Simulate system load for performance testing
    pub async fn simulate_load(
        &self,
        duration_secs: u64,
        agents_per_sec: u32,
        tasks_per_sec: u32,
    ) -> anyhow::Result<LoadTestResults> {
        let start_time = std::time::Instant::now();
        let mut results = LoadTestResults::default();

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        let mut elapsed = 0;

        while elapsed < duration_secs {
            interval.tick().await;
            elapsed += 1;

            // Create agents
            for _ in 0..agents_per_sec {
                match self.create_test_agents(1).await {
                    Ok(_) => results.agents_created += 1,
                    Err(_) => results.agent_creation_failures += 1,
                }
            }

            // Create tasks
            for _ in 0..tasks_per_sec {
                match self.create_test_tasks(1).await {
                    Ok(_) => results.tasks_created += 1,
                    Err(_) => results.task_creation_failures += 1,
                }
            }

            // Record metrics
            if elapsed % 10 == 0 {
                let metrics = self.metrics.get_current_metrics().await;
                results.performance_samples.push(PerformanceSample {
                    timestamp: elapsed,
                    cpu_usage: metrics.resource_usage.cpu_usage_percent,
                    memory_usage: metrics.resource_usage.memory_usage_percent,
                    active_agents: metrics.agent_metrics.active_agents,
                    tasks_in_queue: metrics.task_metrics.tasks_in_queue,
                });
            }
        }

        results.duration_secs = start_time.elapsed().as_secs();
        Ok(results)
    }

    /// Validate system consistency
    pub async fn validate_system_consistency(&self) -> anyhow::Result<ConsistencyReport> {
        let hive = self.hive.read().await;
        let status = hive.get_status().await;
        let agents_info = hive.get_agents_info().await;
        let tasks_info = hive.get_tasks_info().await.unwrap_or_else(|_| json!({}));

        let mut report = ConsistencyReport::default();

        // Validate agent consistency
        if let Some(agents_array) = agents_info.get("agents").and_then(|a| a.as_array()) {
            report.total_agents = agents_array.len();

            for agent in agents_array {
                if let Some(agent_id) = agent.get("id").and_then(|id| id.as_str()) {
                    if uuid::Uuid::parse_str(agent_id).is_ok() {
                        report.valid_agents += 1;
                    } else {
                        report.invalid_agent_ids += 1;
                    }
                }
            }
        }

        // Validate task consistency
        if let Some(tasks_array) = tasks_info.get("tasks").and_then(|t| t.as_array()) {
            report.total_tasks = tasks_array.len();

            for task in tasks_array {
                if let Some(task_id) = task.get("id").and_then(|id| id.as_str()) {
                    if uuid::Uuid::parse_str(task_id).is_ok() {
                        report.valid_tasks += 1;
                    } else {
                        report.invalid_task_ids += 1;
                    }
                }
            }
        }

        // Check metrics consistency
        if let Some(metrics) = status.get("metrics") {
            if let Some(total_agents) = metrics
                .get("total_agents")
                .and_then(serde_json::Value::as_u64)
            {
                if total_agents as usize == report.total_agents {
                    report.metrics_consistent = true;
                }
            }
        }

        Ok(report)
    }

    /// Cleanup test data
    pub async fn cleanup(&self) -> anyhow::Result<()> {
        // Clear caches
        self.cache.agents.clear().await;
        self.cache.tasks.clear().await;
        self.cache.status.clear().await;

        // Reset metrics (in a real implementation)
        // self.metrics.reset().await;

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct LoadTestResults {
    pub duration_secs: u64,
    pub agents_created: u32,
    pub tasks_created: u32,
    pub agent_creation_failures: u32,
    pub task_creation_failures: u32,
    pub performance_samples: Vec<PerformanceSample>,
}

#[derive(Debug)]
pub struct PerformanceSample {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_agents: usize,
    pub tasks_in_queue: usize,
}

#[derive(Debug, Default)]
pub struct ConsistencyReport {
    pub total_agents: usize,
    pub valid_agents: usize,
    pub invalid_agent_ids: usize,
    pub total_tasks: usize,
    pub valid_tasks: usize,
    pub invalid_task_ids: usize,
    pub metrics_consistent: bool,
}

impl LoadTestResults {
    /// Calculate performance statistics
    #[must_use]
    pub fn calculate_stats(&self) -> LoadTestStats {
        let total_operations = self.agents_created + self.tasks_created;
        let total_failures = self.agent_creation_failures + self.task_creation_failures;

        let success_rate = if total_operations + total_failures > 0 {
            f64::from(total_operations) / f64::from(total_operations + total_failures)
        } else {
            0.0
        };

        let ops_per_second = if self.duration_secs > 0 {
            f64::from(total_operations) / self.duration_secs as f64
        } else {
            0.0
        };

        let avg_cpu = if self.performance_samples.is_empty() {
            0.0
        } else {
            self.performance_samples
                .iter()
                .map(|s| s.cpu_usage)
                .sum::<f64>()
                / self.performance_samples.len() as f64
        };

        let avg_memory = if self.performance_samples.is_empty() {
            0.0
        } else {
            self.performance_samples
                .iter()
                .map(|s| s.memory_usage)
                .sum::<f64>()
                / self.performance_samples.len() as f64
        };

        LoadTestStats {
            success_rate,
            operations_per_second: ops_per_second,
            average_cpu_usage: avg_cpu,
            average_memory_usage: avg_memory,
            peak_agents: self
                .performance_samples
                .iter()
                .map(|s| s.active_agents)
                .max()
                .unwrap_or(0),
            peak_queue_size: self
                .performance_samples
                .iter()
                .map(|s| s.tasks_in_queue)
                .max()
                .unwrap_or(0),
        }
    }
}

#[derive(Debug)]
pub struct LoadTestStats {
    pub success_rate: f64,
    pub operations_per_second: f64,
    pub average_cpu_usage: f64,
    pub average_memory_usage: f64,
    pub peak_agents: usize,
    pub peak_queue_size: usize,
}

/// Integration test suite
pub struct IntegrationTests;

impl IntegrationTests {
    /// Test basic agent lifecycle
    pub async fn test_agent_lifecycle() -> anyhow::Result<()> {
        let harness = TestHarness::new().await?;

        // Create agents
        let agent_ids = harness.create_test_agents(5).await?;
        assert_eq!(agent_ids.len(), 5);

        // Validate agents exist
        let agents_info = harness.hive.read().await.get_agents_info().await;
        if let Some(agents) = agents_info.get("agents").and_then(|a| a.as_array()) {
            assert_eq!(agents.len(), 5);
        }

        // Cleanup
        harness.cleanup().await?;

        println!("✅ Agent lifecycle test passed");
        Ok(())
    }

    /// Test task processing
    pub async fn test_task_processing() -> anyhow::Result<()> {
        let harness = TestHarness::new().await?;

        // Create agents and tasks
        let _agent_ids = harness.create_test_agents(3).await?;
        let task_ids = harness.create_test_tasks(5).await?;

        assert_eq!(task_ids.len(), 5);

        // Wait for processing
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Validate tasks
        let tasks_info = harness.hive.read().await.get_tasks_info().await?;
        if let Some(tasks) = tasks_info.get("tasks").and_then(|t| t.as_array()) {
            assert_eq!(tasks.len(), 5);
        }

        harness.cleanup().await?;

        println!("✅ Task processing test passed");
        Ok(())
    }

    /// Test system under load
    pub async fn test_load_handling() -> anyhow::Result<()> {
        let harness = TestHarness::new().await?;

        // Run load test
        let results = harness.simulate_load(10, 2, 3).await?;
        let stats = results.calculate_stats();

        // Validate performance
        assert!(
            stats.success_rate > 0.9,
            "Success rate too low: {}",
            stats.success_rate
        );
        assert!(stats.operations_per_second > 0.0, "No operations processed");

        println!(
            "✅ Load handling test passed - Success rate: {:.2}%, Ops/sec: {:.2}",
            stats.success_rate * 100.0,
            stats.operations_per_second
        );

        harness.cleanup().await?;
        Ok(())
    }

    /// Test system consistency
    pub async fn test_system_consistency() -> anyhow::Result<()> {
        let harness = TestHarness::new().await?;

        // Create test data
        let _agent_ids = harness.create_test_agents(10).await?;
        let _task_ids = harness.create_test_tasks(15).await?;

        // Validate consistency
        let report = harness.validate_system_consistency().await?;

        assert_eq!(report.total_agents, 10);
        assert_eq!(report.valid_agents, 10);
        assert_eq!(report.invalid_agent_ids, 0);
        assert_eq!(report.total_tasks, 15);
        assert_eq!(report.valid_tasks, 15);
        assert_eq!(report.invalid_task_ids, 0);

        println!("✅ System consistency test passed");

        harness.cleanup().await?;
        Ok(())
    }

    /// Run all integration tests
    pub async fn run_all() -> anyhow::Result<()> {
        println!("🧪 Running integration tests...");

        Self::test_agent_lifecycle().await?;
        Self::test_task_processing().await?;
        Self::test_load_handling().await?;
        Self::test_system_consistency().await?;

        println!("🎉 All integration tests passed!");
        Ok(())
    }
}
