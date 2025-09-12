//! # Background Processes Usage Examples
//!
//! This file contains practical examples demonstrating how to use the ProcessManager
//! for managing background processes, monitoring, and system coordination.

use crate::core::hive::background_processes::{ProcessManager, ProcessConfig};
use crate::core::hive::agent_management::AgentManager;
use crate::core::hive::metrics_collection::MetricsCollector;
use crate::core::hive::task_management::TaskDistributor;
use crate::infrastructure::resource_manager::ResourceManager;
use crate::utils::error::HiveResult;
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Example: Basic Background Process Setup
/// Description: Initialize and start all background processes
/// Use case: Setting up a complete multiagent system with background coordination
async fn basic_background_process_setup() -> HiveResult<()> {
    // Create coordination channel
    let (tx, _rx) = mpsc::unbounded_channel();

    // Initialize process manager
    let process_manager = ProcessManager::new(tx.clone()).await?;

    // Initialize core components
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let agent_manager = Arc::new(AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let task_distributor = Arc::new(TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let metrics_collector = Arc::new(MetricsCollector::new(tx).await?);

    // Start all background processes
    process_manager.start_all_processes(
        &agent_manager,
        &task_distributor,
        &metrics_collector,
        &resource_manager,
    ).await?;

    println!("All background processes started successfully");

    // Get initial process status
    let status = process_manager.get_process_status().await;
    println!("Process Status: {}", serde_json::to_string_pretty(&status)?);

    // Let processes run for a short time
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Stop all processes
    process_manager.stop_all_processes().await?;
    println!("All background processes stopped");

    Ok(())
}

/// Example: Custom Process Configuration
/// Description: Configure background processes with custom intervals and settings
/// Use case: Tuning process behavior for specific workloads
async fn custom_process_configuration() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let mut process_manager = ProcessManager::new(tx.clone()).await?;

    // Create custom configuration
    let custom_config = ProcessConfig {
        work_stealing_interval: Duration::from_millis(200),     // Faster work stealing
        learning_interval: Duration::from_secs(60),             // Less frequent learning
        swarm_coordination_interval: Duration::from_millis(500), // Moderate coordination
        metrics_collection_interval: Duration::from_secs(5),     // Frequent metrics
        resource_monitoring_interval: Duration::from_millis(2000), // Resource monitoring
    };

    // Update configuration
    process_manager.update_config(custom_config).await?;

    // Verify configuration
    let status = process_manager.get_process_status().await;
    println!("Custom Process Configuration: {}", serde_json::to_string_pretty(&status)?);

    // Initialize components and start processes
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let agent_manager = Arc::new(AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let task_distributor = Arc::new(TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let metrics_collector = Arc::new(MetricsCollector::new(tx).await?);

    process_manager.start_all_processes(
        &agent_manager,
        &task_distributor,
        &metrics_collector,
        &resource_manager,
    ).await?;

    // Let processes run with custom configuration
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Get updated status
    let updated_status = process_manager.get_process_status().await;
    println!("Status with Custom Configuration: {}", serde_json::to_string_pretty(&updated_status)?);

    process_manager.stop_all_processes().await?;

    Ok(())
}

/// Example: Process Monitoring and Health Checks
/// Description: Monitor background process health and performance
/// Use case: System monitoring and troubleshooting
async fn process_monitoring_and_health_checks() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let process_manager = ProcessManager::new(tx.clone()).await?;

    // Initialize components
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let agent_manager = Arc::new(AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let task_distributor = Arc::new(TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let metrics_collector = Arc::new(MetricsCollector::new(tx).await?);

    // Start processes
    process_manager.start_all_processes(
        &agent_manager,
        &task_distributor,
        &metrics_collector,
        &resource_manager,
    ).await?;

    // Monitor process status over time
    for i in 0..5 {
        let status = process_manager.get_process_status().await;
        let active_processes = status.get("active_processes").unwrap().as_u64().unwrap_or(0);
        let total_processes = status.get("total_processes").unwrap().as_u64().unwrap_or(0);

        println!("Monitoring Check {}: {} / {} processes active",
                 i + 1, active_processes, total_processes);

        // Check individual process health
        if active_processes < total_processes {
            println!("Warning: Some processes may have stopped");
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // Get detailed configuration info
    let config_info = process_manager.get_process_status().await;
    let config = config_info.get("configuration").unwrap();
    println!("Process Configuration Details: {}", serde_json::to_string_pretty(config)?);

    process_manager.stop_all_processes().await?;

    Ok(())
}

/// Example: Resource-Aware Process Management
/// Description: Manage processes based on system resource availability
/// Use case: Resource optimization and system stability
async fn resource_aware_process_management() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let mut process_manager = ProcessManager::new(tx.clone()).await?;

    // Get initial resource status
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let initial_resources = resource_manager.get_system_info().await;

    println!("Initial System Resources:");
    println!("  CPU Usage: {:.1}%", initial_resources.0.cpu_usage * 100.0);
    println!("  Memory Usage: {:.1}%", initial_resources.0.memory_usage * 100.0);
    println!("  Available CPU Cores: {}", initial_resources.0.cpu_cores);

    // Configure processes based on resource availability
    let config = if initial_resources.0.cpu_usage > 0.8 {
        // High CPU usage - reduce process frequency
        ProcessConfig {
            work_stealing_interval: Duration::from_millis(500),
            learning_interval: Duration::from_secs(120),
            swarm_coordination_interval: Duration::from_secs(10),
            metrics_collection_interval: Duration::from_secs(15),
            resource_monitoring_interval: Duration::from_secs(10),
        }
    } else {
        // Normal CPU usage - standard configuration
        ProcessConfig::default()
    };

    process_manager.update_config(config).await?;

    // Initialize components and start processes
    let agent_manager = Arc::new(AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let task_distributor = Arc::new(TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let metrics_collector = Arc::new(MetricsCollector::new(tx).await?);

    process_manager.start_all_processes(
        &agent_manager,
        &task_distributor,
        &metrics_collector,
        &resource_manager,
    ).await?;

    // Monitor resource usage during operation
    for i in 0..10 {
        let resources = resource_manager.get_system_info().await;
        let status = process_manager.get_process_status().await;

        println!("Resource Check {}: CPU {:.1}%, Memory {:.1}%, Active Processes: {}",
                 i + 1,
                 resources.0.cpu_usage * 100.0,
                 resources.0.memory_usage * 100.0,
                 status.get("active_processes").unwrap());

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    process_manager.stop_all_processes().await?;

    Ok(())
}

/// Example: Dynamic Process Scaling
/// Description: Scale background processes based on system load
/// Use case: Auto-scaling and performance optimization
async fn dynamic_process_scaling() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let mut process_manager = ProcessManager::new(tx.clone()).await?;

    // Initialize components
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let agent_manager = Arc::new(AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let task_distributor = Arc::new(TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let metrics_collector = Arc::new(MetricsCollector::new(tx).await?);

    // Start with conservative configuration
    let conservative_config = ProcessConfig {
        work_stealing_interval: Duration::from_millis(1000),
        learning_interval: Duration::from_secs(300),
        swarm_coordination_interval: Duration::from_secs(30),
        metrics_collection_interval: Duration::from_secs(30),
        resource_monitoring_interval: Duration::from_secs(30),
    };

    process_manager.update_config(conservative_config).await?;
    process_manager.start_all_processes(
        &agent_manager,
        &task_distributor,
        &metrics_collector,
        &resource_manager,
    ).await?;

    println!("Started with conservative configuration");

    // Simulate increasing load
    for load_level in 0..5 {
        // Adjust configuration based on simulated load
        let scaling_factor = 1.0 - (load_level as f64 * 0.15); // Decrease intervals as load increases

        let scaled_config = ProcessConfig {
            work_stealing_interval: Duration::from_millis((1000.0 * scaling_factor) as u64),
            learning_interval: Duration::from_secs((300.0 * scaling_factor) as u64),
            swarm_coordination_interval: Duration::from_secs((30.0 * scaling_factor) as u64),
            metrics_collection_interval: Duration::from_secs((30.0 * scaling_factor) as u64),
            resource_monitoring_interval: Duration::from_secs((30.0 * scaling_factor) as u64),
        };

        process_manager.update_config(scaled_config).await?;

        println!("Load Level {}: Scaled process intervals by factor {:.2}", load_level + 1, scaling_factor);

        // Check status after scaling
        let status = process_manager.get_process_status().await;
        println!("  Active Processes: {}", status.get("active_processes").unwrap());

        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    process_manager.stop_all_processes().await?;

    Ok(())
}

/// Example: Process Failure Recovery
/// Description: Handle process failures and implement recovery mechanisms
/// Use case: Building resilient background process systems
async fn process_failure_recovery() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let process_manager = ProcessManager::new(tx.clone()).await?;

    // Initialize components
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let agent_manager = Arc::new(AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let task_distributor = Arc::new(TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let metrics_collector = Arc::new(MetricsCollector::new(tx).await?);

    // Start processes
    process_manager.start_all_processes(
        &agent_manager,
        &task_distributor,
        &metrics_collector,
        &resource_manager,
    ).await?;

    println!("Processes started successfully");

    // Monitor processes and simulate recovery scenarios
    for check in 0..10 {
        let status = process_manager.get_process_status().await;
        let active_processes = status.get("active_processes").unwrap().as_u64().unwrap_or(0);
        let total_processes = status.get("total_processes").unwrap().as_u64().unwrap_or(0);

        println!("Health Check {}: {} / {} processes active", check + 1, active_processes, total_processes);

        // Simulate process failure detection
        if active_processes < total_processes {
            println!("  Detected process failure - attempting recovery...");

            // In a real scenario, you might restart failed processes
            // For this example, we'll just log the issue
            println!("  Recovery: Some processes have stopped. In production, implement restart logic.");
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // Graceful shutdown
    process_manager.stop_all_processes().await?;
    println!("Processes shut down gracefully");

    Ok(())
}

/// Example: Process Performance Benchmarking
/// Description: Benchmark background process performance under different configurations
/// Use case: Performance testing and optimization
async fn process_performance_benchmarking() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let mut process_manager = ProcessManager::new(tx.clone()).await?;

    // Initialize components
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let agent_manager = Arc::new(AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let task_distributor = Arc::new(TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let metrics_collector = Arc::new(MetricsCollector::new(tx).await?);

    // Define benchmark configurations
    let benchmark_configs = vec![
        ("high_frequency", ProcessConfig {
            work_stealing_interval: Duration::from_millis(50),
            learning_interval: Duration::from_secs(10),
            swarm_coordination_interval: Duration::from_millis(100),
            metrics_collection_interval: Duration::from_secs(1),
            resource_monitoring_interval: Duration::from_millis(500),
        }),
        ("balanced", ProcessConfig::default()),
        ("low_frequency", ProcessConfig {
            work_stealing_interval: Duration::from_millis(1000),
            learning_interval: Duration::from_secs(600),
            swarm_coordination_interval: Duration::from_secs(60),
            metrics_collection_interval: Duration::from_secs(60),
            resource_monitoring_interval: Duration::from_secs(60),
        }),
    ];

    for (config_name, config) in benchmark_configs {
        println!("Benchmarking configuration: {}", config_name);

        process_manager.update_config(config).await?;
        process_manager.start_all_processes(
            &agent_manager,
            &task_distributor,
            &metrics_collector,
            &resource_manager,
        ).await?;

        // Run benchmark for this configuration
        let benchmark_start = std::time::Instant::now();

        // Let processes run and collect some metrics
        tokio::time::sleep(Duration::from_secs(5)).await;

        let benchmark_duration = benchmark_start.elapsed();
        let status = process_manager.get_process_status().await;

        println!("  Duration: {:.2}s", benchmark_duration.as_secs_f64());
        println!("  Active Processes: {}", status.get("active_processes").unwrap());
        println!("  Configuration: {}", serde_json::to_string_pretty(status.get("configuration").unwrap())?);

        process_manager.stop_all_processes().await?;
        tokio::time::sleep(Duration::from_secs(1)).await; // Brief pause between benchmarks
    }

    Ok(())
}

/// Example: Process Coordination and Synchronization
/// Description: Coordinate multiple background processes for complex workflows
/// Use case: Complex multi-step processing pipelines
async fn process_coordination_and_synchronization() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let process_manager = ProcessManager::new(tx.clone()).await?;

    // Initialize components
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let agent_manager = Arc::new(AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let task_distributor = Arc::new(TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let metrics_collector = Arc::new(MetricsCollector::new(tx).await?);

    // Configure processes for coordinated operation
    let coordination_config = ProcessConfig {
        work_stealing_interval: Duration::from_millis(200),
        learning_interval: Duration::from_secs(30),
        swarm_coordination_interval: Duration::from_millis(500),
        metrics_collection_interval: Duration::from_secs(5),
        resource_monitoring_interval: Duration::from_millis(1000),
    };

    // Start processes with coordination configuration
    process_manager.start_all_processes(
        &agent_manager,
        &task_distributor,
        &metrics_collector,
        &resource_manager,
    ).await?;

    println!("Coordinated processes started");

    // Simulate coordinated workflow
    for phase in 0..3 {
        println!("Coordination Phase {}", phase + 1);

        // Adjust process timing for different phases
        let phase_config = ProcessConfig {
            work_stealing_interval: Duration::from_millis(200 + phase * 100),
            learning_interval: Duration::from_secs(30 - phase * 5),
            swarm_coordination_interval: Duration::from_millis(500 - phase * 100),
            metrics_collection_interval: Duration::from_secs(5),
            resource_monitoring_interval: Duration::from_millis(1000),
        };

        // Monitor coordination effectiveness
        let status_before = process_manager.get_process_status().await;
        println!("  Status before phase: {} active processes",
                 status_before.get("active_processes").unwrap());

        tokio::time::sleep(Duration::from_secs(3)).await;

        let status_after = process_manager.get_process_status().await;
        println!("  Status after phase: {} active processes",
                 status_after.get("active_processes").unwrap());
    }

    process_manager.stop_all_processes().await?;
    println!("Coordinated processes completed");

    Ok(())
}

/// Example: Process Lifecycle Management
/// Description: Manage the complete lifecycle of background processes
/// Use case: Application startup, runtime management, and shutdown
async fn process_lifecycle_management() -> HiveResult<()> {
    println!("=== Process Lifecycle Management Example ===");

    // Phase 1: Initialization
    println!("Phase 1: Initializing process manager...");
    let (tx, _rx) = mpsc::unbounded_channel();
    let process_manager = ProcessManager::new(tx.clone()).await?;
    println!("✓ Process manager initialized");

    // Phase 2: Component Setup
    println!("Phase 2: Setting up system components...");
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let agent_manager = Arc::new(AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let task_distributor = Arc::new(TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?);
    let metrics_collector = Arc::new(MetricsCollector::new(tx).await?);
    println!("✓ All components initialized");

    // Phase 3: Configuration
    println!("Phase 3: Configuring processes...");
    let config = ProcessConfig {
        work_stealing_interval: Duration::from_millis(300),
        learning_interval: Duration::from_secs(45),
        swarm_coordination_interval: Duration::from_millis(800),
        metrics_collection_interval: Duration::from_secs(8),
        resource_monitoring_interval: Duration::from_millis(1500),
    };
    process_manager.update_config(config).await?;
    println!("✓ Processes configured");

    // Phase 4: Startup
    println!("Phase 4: Starting background processes...");
    process_manager.start_all_processes(
        &agent_manager,
        &task_distributor,
        &metrics_collector,
        &resource_manager,
    ).await?;
    println!("✓ All processes started");

    // Phase 5: Runtime Monitoring
    println!("Phase 5: Monitoring runtime operation...");
    for i in 0..5 {
        let status = process_manager.get_process_status().await;
        println!("  Runtime check {}: {} processes active",
                 i + 1, status.get("active_processes").unwrap());
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // Phase 6: Graceful Shutdown
    println!("Phase 6: Initiating graceful shutdown...");
    process_manager.stop_all_processes().await?;
    println!("✓ All processes stopped gracefully");

    println!("=== Process Lifecycle Management Complete ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_background_process_setup() {
        basic_background_process_setup().await.unwrap();
    }

    #[tokio::test]
    async fn test_custom_process_configuration() {
        custom_process_configuration().await.unwrap();
    }

    #[tokio::test]
    async fn test_process_monitoring_and_health_checks() {
        process_monitoring_and_health_checks().await.unwrap();
    }

    #[tokio::test]
    async fn test_resource_aware_process_management() {
        resource_aware_process_management().await.unwrap();
    }

    #[tokio::test]
    async fn test_dynamic_process_scaling() {
        dynamic_process_scaling().await.unwrap();
    }

    #[tokio::test]
    async fn test_process_failure_recovery() {
        process_failure_recovery().await.unwrap();
    }

    #[tokio::test]
    async fn test_process_performance_benchmarking() {
        process_performance_benchmarking().await.unwrap();
    }

    #[tokio::test]
    async fn test_process_coordination_and_synchronization() {
        process_coordination_and_synchronization().await.unwrap();
    }

    #[tokio::test]
    async fn test_process_lifecycle_management() {
        process_lifecycle_management().await.unwrap();
    }
}
