//! # Metrics Collection Usage Examples
//!
//! This file contains practical examples demonstrating how to use the MetricsCollector
//! for comprehensive system monitoring, analytics, and performance tracking.

use crate::core::hive::metrics_collection::{MetricsCollector, HiveMetrics, AgentMetrics, TaskMetrics, SystemMetrics, ResourceMetrics};
use crate::utils::error::HiveResult;
use serde_json;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Example: Basic Metrics Collection Setup
/// Description: Initialize metrics collection and start basic monitoring
/// Use case: Setting up system monitoring for a multiagent system
async fn basic_metrics_collection_setup() -> HiveResult<()> {
    // Create coordination channel
    let (tx, _rx) = mpsc::unbounded_channel();

    // Initialize metrics collector
    let metrics_collector = MetricsCollector::new(tx).await?;

    // Get initial metrics
    let initial_metrics = metrics_collector.get_current_metrics().await;
    println!("Initial Metrics: {}", serde_json::to_string_pretty(&initial_metrics)?);

    // Get metrics summary
    let summary = metrics_collector.get_metrics_summary().await;
    println!("Metrics Summary: {}", serde_json::to_string_pretty(&summary)?);

    Ok(())
}

/// Example: Agent Metrics Tracking
/// Description: Track and analyze agent performance metrics
/// Use case: Monitoring agent health and performance
async fn agent_metrics_tracking() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let metrics_collector = MetricsCollector::new(tx).await?;

    // Simulate agent events
    let agent_ids = vec![
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    ];

    // Register agents
    for &agent_id in &agent_ids {
        metrics_collector.record_agent_event("registered", agent_id).await;
    }

    // Simulate task executions with different performance characteristics
    for (i, &agent_id) in agent_ids.iter().enumerate() {
        let tasks_completed = 10 + i * 5; // Different numbers of tasks
        let base_time = 100 + i * 50; // Different execution times

        for task_num in 0..tasks_completed {
            let execution_time = base_time + (rand::random::<u64>() % 100);
            let success = rand::random::<f64>() < 0.85; // 85% success rate
            let task_id = Uuid::new_v4();

            metrics_collector.record_task_completion(task_id, agent_id, success).await;
        }
    }

    // Remove one agent
    if let Some(&agent_to_remove) = agent_ids.first() {
        metrics_collector.record_agent_event("removed", agent_to_remove).await;
    }

    // Get current metrics
    let metrics = metrics_collector.get_current_metrics().await;
    println!("Agent Metrics: {}", serde_json::to_string_pretty(&metrics.agent_metrics)?);

    // Get enhanced analytics
    let analytics = metrics_collector.get_enhanced_metrics().await;
    println!("Enhanced Analytics: {}", serde_json::to_string_pretty(&analytics)?);

    Ok(())
}

/// Example: Task Performance Analytics
/// Description: Analyze task execution patterns and performance
/// Use case: Optimizing task distribution and execution
async fn task_performance_analytics() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let metrics_collector = MetricsCollector::new(tx).await?;

    // Simulate various task execution scenarios
    let scenarios = vec![
        ("fast_successful", 50, true, 20),
        ("fast_failed", 30, false, 5),
        ("medium_successful", 150, true, 15),
        ("medium_failed", 120, false, 3),
        ("slow_successful", 300, true, 8),
        ("slow_failed", 250, false, 2),
    ];

    for (scenario, base_time, success, count) in scenarios {
        for _ in 0..count {
            let execution_time = base_time + (rand::random::<i32>() % 50 - 25) as u64;
            let task_id = Uuid::new_v4();
            let agent_id = Uuid::new_v4();

            metrics_collector.record_task_completion(task_id, agent_id, success).await;
        }
    }

    // Get task metrics
    let metrics = metrics_collector.get_current_metrics().await;
    println!("Task Metrics: {}", serde_json::to_string_pretty(&metrics.task_metrics)?);

    // Get detailed analytics
    let analytics = metrics_collector.get_enhanced_metrics().await;
    println!("Task Performance Analytics: {}", serde_json::to_string_pretty(&analytics)?);

    Ok(())
}

/// Example: System Resource Monitoring
/// Description: Monitor system resources and performance
/// Use case: Resource usage tracking and capacity planning
async fn system_resource_monitoring() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let metrics_collector = MetricsCollector::new(tx).await?;

    // Simulate system metrics updates
    let system_updates = vec![
        serde_json::json!({"cpu_usage": 0.3, "memory_usage": 0.4}),
        serde_json::json!({"cpu_usage": 0.5, "memory_usage": 0.6}),
        serde_json::json!({"cpu_usage": 0.7, "memory_usage": 0.8}),
        serde_json::json!({"cpu_usage": 0.6, "memory_usage": 0.7}),
        serde_json::json!({"cpu_usage": 0.4, "memory_usage": 0.5}),
    ];

    for update in system_updates {
        metrics_collector.update_metrics(update).await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Collect periodic metrics
    metrics_collector.collect_periodic_metrics().await?;

    // Get system metrics
    let metrics = metrics_collector.get_current_metrics().await;
    println!("System Metrics: {}", serde_json::to_string_pretty(&metrics.system_metrics)?);

    // Get resource metrics
    println!("Resource Metrics: {}", serde_json::to_string_pretty(&metrics.resource_metrics)?);

    Ok(())
}

/// Example: Comprehensive System Dashboard
/// Description: Create a comprehensive dashboard view of system metrics
/// Use case: System monitoring dashboard and alerting
async fn comprehensive_system_dashboard() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let metrics_collector = MetricsCollector::new(tx).await?;

    // Simulate a comprehensive system state
    let agent_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

    // Register agents
    for &agent_id in &agent_ids {
        metrics_collector.record_agent_event("registered", agent_id).await;
    }

    // Simulate task executions
    for &agent_id in &agent_ids {
        for _ in 0..rand::random::<u32>() % 10 + 5 {
            let task_id = Uuid::new_v4();
            let success = rand::random::<f64>() < 0.9;
            metrics_collector.record_task_completion(task_id, agent_id, success).await;
        }
    }

    // Update system metrics
    let system_data = serde_json::json!({
        "cpu_usage": 0.65,
        "memory_usage": 0.72,
        "network_throughput_mbps": 150.5,
        "error_rate": 0.02,
        "response_time_ms": 45.2
    });
    metrics_collector.update_metrics(system_data).await;

    // Collect periodic metrics
    metrics_collector.collect_periodic_metrics().await?;

    // Generate comprehensive dashboard
    let current_metrics = metrics_collector.get_current_metrics().await;
    let summary = metrics_collector.get_metrics_summary().await;
    let enhanced = metrics_collector.get_enhanced_metrics().await;

    println!("=== SYSTEM DASHBOARD ===");
    println!("Current Metrics: {}", serde_json::to_string_pretty(&current_metrics)?);
    println!("Summary: {}", serde_json::to_string_pretty(&summary)?);
    println!("Enhanced Analytics: {}", serde_json::to_string_pretty(&enhanced)?);

    Ok(())
}

/// Example: Metrics Export and Integration
/// Description: Export metrics in different formats for external systems
/// Use case: Integration with monitoring systems and external tools
async fn metrics_export_and_integration() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let metrics_collector = MetricsCollector::new(tx).await?;

    // Add some sample data
    let agent_id = Uuid::new_v4();
    metrics_collector.record_agent_event("registered", agent_id).await;

    for _ in 0..5 {
        let task_id = Uuid::new_v4();
        metrics_collector.record_task_completion(task_id, agent_id, true).await;
    }

    metrics_collector.update_metrics(serde_json::json!({"cpu_usage": 0.5, "memory_usage": 0.6})).await;

    // Export in JSON format
    let json_export = metrics_collector.export_metrics("json").await?;
    println!("JSON Export: {}", json_export);

    // Export in Prometheus format
    let prometheus_export = metrics_collector.export_metrics("prometheus").await?;
    println!("Prometheus Export:");
    println!("{}", prometheus_export);

    // Test invalid format
    match metrics_collector.export_metrics("invalid").await {
        Ok(_) => println!("Unexpected success with invalid format"),
        Err(e) => println!("Expected error with invalid format: {}", e),
    }

    Ok(())
}

/// Example: Trend Analysis and Forecasting
/// Description: Analyze metrics trends and predict future performance
/// Use case: Capacity planning and performance optimization
async fn trend_analysis_and_forecasting() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let metrics_collector = MetricsCollector::new(tx).await?;

    // Simulate metrics over time
    let time_series_data = vec![
        (5, 0.3, 0.4), // agents, cpu, memory
        (7, 0.4, 0.5),
        (8, 0.5, 0.6),
        (6, 0.6, 0.7),
        (9, 0.4, 0.5),
        (10, 0.5, 0.6),
    ];

    for (agent_count, cpu_usage, memory_usage) in time_series_data {
        // Simulate agent changes
        for _ in 0..agent_count {
            let agent_id = Uuid::new_v4();
            metrics_collector.record_agent_event("registered", agent_id).await;
        }

        // Update system metrics
        metrics_collector.update_metrics(serde_json::json!({
            "cpu_usage": cpu_usage,
            "memory_usage": memory_usage
        })).await;

        // Collect periodic metrics to create history
        metrics_collector.collect_periodic_metrics().await?;
    }

    // Get enhanced metrics with trends
    let enhanced = metrics_collector.get_enhanced_metrics().await;
    let trends = enhanced.get("trends").unwrap();

    println!("Trend Analysis: {}", serde_json::to_string_pretty(trends)?);

    // Analyze trends
    if let Some(agent_growth) = trends.get("agent_growth_percent").and_then(|v| v.as_f64()) {
        println!("Agent Growth Trend: {:.2}%", agent_growth);
    }

    if let Some(task_growth) = trends.get("task_growth_percent").and_then(|v| v.as_f64()) {
        println!("Task Growth Trend: {:.2}%", task_growth);
    }

    if let Some(cpu_change) = trends.get("cpu_usage_change").and_then(|v| v.as_f64()) {
        println!("CPU Usage Change: {:.2}%", cpu_change * 100.0);
    }

    Ok(())
}

/// Example: Alerting and Threshold Monitoring
/// Description: Monitor metrics and trigger alerts based on thresholds
/// Use case: Proactive system monitoring and incident response
async fn alerting_and_threshold_monitoring() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let metrics_collector = MetricsCollector::new(tx).await?;

    // Define alert thresholds
    let cpu_threshold = 0.8;
    let memory_threshold = 0.9;
    let error_rate_threshold = 0.05;

    // Simulate system load scenarios
    let load_scenarios = vec![
        ("normal", 0.4, 0.5, 0.01),
        ("moderate", 0.6, 0.7, 0.02),
        ("high", 0.8, 0.85, 0.03),
        ("critical", 0.9, 0.95, 0.08),
        ("recovery", 0.5, 0.6, 0.02),
    ];

    for (scenario, cpu_usage, memory_usage, error_rate) in load_scenarios {
        println!("Scenario: {}", scenario);

        // Update metrics
        metrics_collector.update_metrics(serde_json::json!({
            "cpu_usage": cpu_usage,
            "memory_usage": memory_usage,
            "error_rate": error_rate
        })).await;

        // Check thresholds and generate alerts
        let metrics = metrics_collector.get_current_metrics().await;

        if metrics.system_metrics.cpu_usage_percent > cpu_threshold * 100.0 {
            println!("  🚨 ALERT: High CPU usage: {:.1}%", metrics.system_metrics.cpu_usage_percent);
        }

        if metrics.system_metrics.total_memory_usage_mb > memory_threshold {
            println!("  🚨 ALERT: High memory usage: {:.1}%", metrics.system_metrics.total_memory_usage_mb * 100.0);
        }

        if metrics.system_metrics.error_rate > error_rate_threshold {
            println!("  🚨 ALERT: High error rate: {:.1}%", metrics.system_metrics.error_rate * 100.0);
        }

        // Check agent health
        if metrics.agent_metrics.active_agents < metrics.agent_metrics.total_agents {
            println!("  ⚠️  WARNING: Some agents are inactive");
        }

        // Check task success rate
        if metrics.task_metrics.success_rate < 0.8 {
            println!("  ⚠️  WARNING: Task success rate below 80%: {:.1}%",
                     metrics.task_metrics.success_rate * 100.0);
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    Ok(())
}

/// Example: Performance Benchmarking with Metrics
/// Description: Use metrics to benchmark system performance
/// Use case: Performance testing and optimization
async fn performance_benchmarking_with_metrics() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let metrics_collector = MetricsCollector::new(tx).await?;

    // Benchmark different scenarios
    let benchmark_scenarios = vec![
        ("light_load", 5, 50),   // agents, tasks_per_agent
        ("medium_load", 10, 100),
        ("heavy_load", 20, 200),
    ];

    for (scenario_name, num_agents, tasks_per_agent) in benchmark_scenarios {
        println!("Benchmarking: {}", scenario_name);

        let benchmark_start = std::time::Instant::now();

        // Create agents
        let mut agent_ids = Vec::new();
        for _ in 0..num_agents {
            let agent_id = Uuid::new_v4();
            metrics_collector.record_agent_event("registered", agent_id).await;
            agent_ids.push(agent_id);
        }

        // Execute tasks
        for &agent_id in &agent_ids {
            for _ in 0..tasks_per_agent {
                let task_id = Uuid::new_v4();
                let success = rand::random::<f64>() < 0.95; // 95% success rate
                metrics_collector.record_task_completion(task_id, agent_id, success).await;
            }
        }

        let benchmark_duration = benchmark_start.elapsed();

        // Collect final metrics
        metrics_collector.collect_periodic_metrics().await?;
        let final_metrics = metrics_collector.get_current_metrics().await;

        println!("  Duration: {:.2}s", benchmark_duration.as_secs_f64());
        println!("  Agents: {}", final_metrics.agent_metrics.total_agents);
        println!("  Tasks: {}", final_metrics.task_metrics.total_tasks);
        println!("  Success Rate: {:.1}%", final_metrics.task_metrics.success_rate * 100.0);
        println!("  Tasks per Second: {:.1}",
                 final_metrics.task_metrics.total_tasks as f64 / benchmark_duration.as_secs_f64());

        // Reset for next benchmark
        metrics_collector.reset_daily_counters().await;
    }

    Ok(())
}

/// Example: Custom Metrics and Extensions
/// Description: Extend metrics collection with custom metrics
/// Use case: Domain-specific monitoring and analytics
async fn custom_metrics_and_extensions() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let metrics_collector = MetricsCollector::new(tx).await?;

    // Simulate custom domain-specific metrics
    let custom_metrics_scenarios = vec![
        ("user_sessions", 150, 0.02),
        ("api_requests", 2500, 0.005),
        ("database_queries", 5000, 0.01),
        ("cache_hits", 8500, 0.001),
    ];

    for (metric_name, throughput, error_rate) in custom_metrics_scenarios {
        println!("Custom Metric: {}", metric_name);

        // Update system metrics with custom context
        metrics_collector.update_metrics(serde_json::json!({
            "cpu_usage": 0.6,
            "memory_usage": 0.7,
            "custom_throughput": throughput,
            "custom_error_rate": error_rate,
            "custom_metric_name": metric_name
        })).await;

        // Add some agent and task activity
        let agent_id = Uuid::new_v4();
        metrics_collector.record_agent_event("registered", agent_id).await;

        for _ in 0..5 {
            let task_id = Uuid::new_v4();
            metrics_collector.record_task_completion(task_id, agent_id, true).await;
        }

        // Get enhanced metrics with custom data
        let enhanced = metrics_collector.get_enhanced_metrics().await;
        println!("  Enhanced Metrics: {}", serde_json::to_string_pretty(&enhanced)?);

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    Ok(())
}

/// Example: Metrics Persistence and Historical Analysis
/// Description: Persist metrics and analyze historical trends
/// Use case: Long-term performance analysis and capacity planning
async fn metrics_persistence_and_historical_analysis() -> HiveResult<()> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let metrics_collector = MetricsCollector::new(tx).await?;

    // Simulate metrics collection over time
    println!("Collecting metrics over time...");

    for hour in 0..24 {
        println!("Hour {}: Simulating system activity", hour);

        // Simulate varying load throughout the day
        let base_load = if hour >= 9 && hour <= 17 { 0.8 } else { 0.3 }; // Higher load during business hours
        let cpu_usage = base_load + (rand::random::<f64>() - 0.5) * 0.2;
        let memory_usage = base_load + (rand::random::<f64>() - 0.5) * 0.3;

        // Update system metrics
        metrics_collector.update_metrics(serde_json::json!({
            "cpu_usage": cpu_usage.max(0.0).min(1.0),
            "memory_usage": memory_usage.max(0.0).min(1.0)
        })).await;

        // Simulate agent activity
        let num_agents = (5 + (rand::random::<i32>() % 10)) as usize;
        for _ in 0..num_agents {
            let agent_id = Uuid::new_v4();
            metrics_collector.record_agent_event("registered", agent_id).await;

            // Simulate tasks for each agent
            let num_tasks = rand::random::<u32>() % 20;
            for _ in 0..num_tasks {
                let task_id = Uuid::new_v4();
                let success = rand::random::<f64>() < 0.9;
                metrics_collector.record_task_completion(task_id, agent_id, success).await;
            }
        }

        // Collect periodic metrics (creates historical snapshot)
        metrics_collector.collect_periodic_metrics().await?;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Analyze historical data
    let final_enhanced = metrics_collector.get_enhanced_metrics().await;
    let history_size = final_enhanced.get("history_size").unwrap().as_u64().unwrap();

    println!("Historical Analysis Complete:");
    println!("  Total snapshots: {}", history_size);
    println!("  Final metrics: {}", serde_json::to_string_pretty(&final_enhanced)?);

    // Reset daily counters for new day
    metrics_collector.reset_daily_counters().await;
    println!("Daily counters reset for new analysis period");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_metrics_collection_setup() {
        basic_metrics_collection_setup().await.unwrap();
    }

    #[tokio::test]
    async fn test_agent_metrics_tracking() {
        agent_metrics_tracking().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_performance_analytics() {
        task_performance_analytics().await.unwrap();
    }

    #[tokio::test]
    async fn test_system_resource_monitoring() {
        system_resource_monitoring().await.unwrap();
    }

    #[tokio::test]
    async fn test_comprehensive_system_dashboard() {
        comprehensive_system_dashboard().await.unwrap();
    }

    #[tokio::test]
    async fn test_metrics_export_and_integration() {
        metrics_export_and_integration().await.unwrap();
    }

    #[tokio::test]
    async fn test_trend_analysis_and_forecasting() {
        trend_analysis_and_forecasting().await.unwrap();
    }

    #[tokio::test]
    async fn test_alerting_and_threshold_monitoring() {
        alerting_and_threshold_monitoring().await.unwrap();
    }

    #[tokio::test]
    async fn test_performance_benchmarking_with_metrics() {
        performance_benchmarking_with_metrics().await.unwrap();
    }

    #[tokio::test]
    async fn test_custom_metrics_and_extensions() {
        custom_metrics_and_extensions().await.unwrap();
    }

    #[tokio::test]
    async fn test_metrics_persistence_and_historical_analysis() {
        metrics_persistence_and_historical_analysis().await.unwrap();
    }
}
