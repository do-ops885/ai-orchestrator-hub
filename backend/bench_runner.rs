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
        monitor_clone.start_monitoring().await;
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
