//! # AI Orchestrator Hub Benchmark Runner
//!
//! This crate provides a comprehensive benchmark runner for the AI Orchestrator Hub,
//! designed to execute performance benchmarks and display detailed results.
//!
//! ## Overview
//!
//! The benchmark runner performs systematic performance testing of the AI Orchestrator Hub's
//! core components, including:
//! - Agent communication throughput
//! - Neural network processing performance
//! - Swarm coordination efficiency
//! - Memory usage patterns
//! - CPU utilization metrics
//!
//! ## Features
//!
//! - **Comprehensive Benchmarking**: Executes a full suite of performance tests
//! - **Real-time Monitoring**: Tracks system resources during benchmark execution
//! - **Detailed Results**: Provides throughput, memory usage, and custom metrics
//! - **Performance Statistics**: Aggregates system-wide performance data
//! - **Error Handling**: Robust error reporting for failed benchmarks
//!
//! ## Usage
//!
//! Run the benchmark suite with default configuration:
//!
//! ```bash
//! cargo run --bin bench_runner
//! ```
//!
//! The runner will:
//! 1. Initialize performance monitoring
//! 2. Execute all configured benchmarks
//! 3. Display detailed results for each benchmark
//! 4. Show aggregated performance statistics
//!
//! ## Output
//!
//! Results include:
//! - Execution duration in milliseconds
//! - Operations per second throughput
//! - Memory usage statistics
//! - Custom performance metrics
//! - System-wide performance statistics
//!
//! ## Dependencies
//!
//! This crate depends on:
//! - `multiagent_hive`: Core AI Orchestrator Hub functionality
//! - `tokio`: Async runtime for concurrent operations

use multiagent_hive::infrastructure::benchmarks::{
    create_default_benchmark_suite, PerformanceConfig, PerformanceMonitor,
};
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting AI Orchestrator Hub Performance Benchmarks");
    println!("====================================================");

    // Create performance monitor
    let config = PerformanceConfig::default();
    let monitor = PerformanceMonitor::new(config);

    // Start monitoring
    let monitor_clone = monitor.clone();
    tokio::spawn(async move {
        monitor_clone.start_monitoring();
    });

    // Create and run benchmark suite
    let suite = create_default_benchmark_suite();
    println!("📊 Running benchmark suite: {}", suite.name);
    println!("📝 Description: {}", suite.description);
    println!("🎯 Number of benchmarks: {}", suite.benchmarks.len());
    println!();

    let results = monitor.run_benchmark_suite(&suite).await?;

    // Display results
    println!("📈 Benchmark Results:");
    println!("====================");

    for result in &results {
        println!("✅ {}:", result.test_id);
        println!("   Duration: {:.2}ms", result.duration_ms);
        println!(
            "   Throughput: {:.2} ops/sec",
            result.throughput_ops_per_sec
        );
        println!(
            "   Memory Usage: {:.2} MB",
            result.memory_usage.average_memory_mb
        );
        println!(
            "   Memory Growth: {:.2} MB",
            result.memory_usage.memory_growth_mb
        );
        println!("   Iterations: {}", result.iterations_completed);
        println!("   Success: {}", result.success);

        if !result.custom_metrics.is_empty() {
            println!("   Custom Metrics:");
            for (key, value) in &result.custom_metrics {
                println!("     {}: {:.2}", key, value);
            }
        }

        if let Some(error) = &result.error_message {
            println!("   ❌ Error: {}", error);
        }
        println!();
    }

    // Get performance stats
    let stats = monitor.get_performance_stats().await;
    println!("📊 Performance Statistics:");
    println!("==========================");
    println!("Total snapshots: {}", stats.total_snapshots);
    println!("Total benchmarks: {}", stats.total_benchmarks);
    println!("Active alerts: {}", stats.active_alerts);
    println!("Average memory usage: {:.2} MB", stats.average_memory_usage);
    println!("Average CPU usage: {:.2}%", stats.average_cpu_usage);
    println!("Memory leak detected: {}", stats.memory_leak_detected);
    println!("Uptime: {:.2} hours", stats.uptime_hours);

    println!();
    println!("🎉 Benchmarking completed successfully!");

    Ok(())
}
