//! Load Test Runner Binary
//!
//! Standalone binary for executing comprehensive load tests
//! to validate performance optimizations and scalability.

use multiagent_hive::infrastructure::load_testing::{LoadTestConfig, LoadTestEngine, LoadTestOperation};
use serde_json;
use std::fs;
use tokio;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 Starting AI Orchestrator Hub Load Testing...");

    // Load test configurations
    let configs = vec![
        create_light_load_config(),
        create_medium_load_config(),
        create_heavy_load_config(),
        create_stress_test_config(),
    ];

    let mut all_results = Vec::new();

    for (i, config) in configs.into_iter().enumerate() {
        info!("📊 Running load test {} of 4: {}", i + 1, get_test_name(i));
        info!("⚙️  Config: {} users, {:.1} RPS/user, {} seconds", 
            config.concurrent_users, 
            config.requests_per_second_per_user,
            config.duration_secs
        );

        let engine = LoadTestEngine::new(config);
        
        match engine.execute_load_test().await {
            Ok(result) => {
                info!("✅ Test completed successfully");
                info!("📈 Results: {:.1} ops/sec avg, {:.1}% success rate", 
                    result.summary.avg_requests_per_second,
                    result.summary.success_rate_percent
                );
                info!("⚡ Optimization effectiveness: {:.1}%", 
                    result.optimization_effectiveness.optimization_impact.overall_optimization_effectiveness
                );
                
                all_results.push(result);
            }
            Err(e) => {
                eprintln!("❌ Load test {} failed: {}", i + 1, e);
            }
        }

        // Brief pause between tests
        if i < 3 {
            info!("⏳ Pausing 30 seconds before next test...");
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    }

    // Generate comprehensive report
    generate_load_test_report(&all_results).await?;

    info!("🎉 Load testing completed! Check load_test_report.json for detailed results.");

    Ok(())
}

fn create_light_load_config() -> LoadTestConfig {
    LoadTestConfig {
        duration_secs: 120, // 2 minutes
        concurrent_users: 25,
        requests_per_second_per_user: 5.0,
        ramp_up_secs: 15,
        ramp_down_secs: 15,
        operation_types: vec![
            LoadTestOperation::TaskSubmission,
            LoadTestOperation::MessageSending,
            LoadTestOperation::MemoryPoolOperations,
        ],
        collect_detailed_metrics: true,
        enable_memory_pressure: false,
        enable_cpu_stress: false,
    }
}

fn create_medium_load_config() -> LoadTestConfig {
    LoadTestConfig {
        duration_secs: 180, // 3 minutes
        concurrent_users: 50,
        requests_per_second_per_user: 10.0,
        ramp_up_secs: 30,
        ramp_down_secs: 20,
        operation_types: vec![
            LoadTestOperation::TaskSubmission,
            LoadTestOperation::MessageSending,
            LoadTestOperation::MemoryPoolOperations,
            LoadTestOperation::LoadBalancerOperations,
        ],
        collect_detailed_metrics: true,
        enable_memory_pressure: true,
        enable_cpu_stress: false,
    }
}

fn create_heavy_load_config() -> LoadTestConfig {
    LoadTestConfig {
        duration_secs: 240, // 4 minutes
        concurrent_users: 100,
        requests_per_second_per_user: 15.0,
        ramp_up_secs: 45,
        ramp_down_secs: 30,
        operation_types: vec![
            LoadTestOperation::TaskSubmission,
            LoadTestOperation::MessageSending,
            LoadTestOperation::MemoryPoolOperations,
            LoadTestOperation::LoadBalancerOperations,
            LoadTestOperation::OptimizedCommunication,
        ],
        collect_detailed_metrics: true,
        enable_memory_pressure: true,
        enable_cpu_stress: true,
    }
}

fn create_stress_test_config() -> LoadTestConfig {
    LoadTestConfig {
        duration_secs: 300, // 5 minutes
        concurrent_users: 200,
        requests_per_second_per_user: 20.0,
        ramp_up_secs: 60,
        ramp_down_secs: 40,
        operation_types: vec![
            LoadTestOperation::TaskSubmission,
            LoadTestOperation::MessageSending,
            LoadTestOperation::MemoryPoolOperations,
            LoadTestOperation::LoadBalancerOperations,
            LoadTestOperation::OptimizedCommunication,
            LoadTestOperation::HealthChecks,
        ],
        collect_detailed_metrics: true,
        enable_memory_pressure: true,
        enable_cpu_stress: true,
    }
}

fn get_test_name(index: usize) -> &'static str {
    match index {
        0 => "Light Load (25 users)",
        1 => "Medium Load (50 users)",
        2 => "Heavy Load (100 users)",
        3 => "Stress Test (200 users)",
        _ => "Unknown Test",
    }
}

async fn generate_load_test_report(results: &[multiagent_hive::infrastructure::load_testing::LoadTestResult]) -> Result<(), Box<dyn std::error::Error>> {
    let report = serde_json::to_string_pretty(results)?;
    fs::write("load_test_report.json", report)?;
    
    // Generate summary report
    let mut summary_lines = vec![
        "# Load Test Summary Report".to_string(),
        "".to_string(),
        "## Test Results Overview".to_string(),
        "".to_string(),
    ];
    
    for (i, result) in results.iter().enumerate() {
        summary_lines.push(format!("### {} - {}", i + 1, get_test_name(i)));
        summary_lines.push("".to_string());
        summary_lines.push(format!("- **Concurrent Users**: {}", result.config.concurrent_users));
        summary_lines.push(format!("- **Duration**: {} seconds", result.config.duration_secs));
        summary_lines.push(format!("- **Total Requests**: {}", result.summary.total_requests));
        summary_lines.push(format!("- **Success Rate**: {:.1}%", result.summary.success_rate_percent));
        summary_lines.push(format!("- **Avg Throughput**: {:.1} ops/sec", result.summary.avg_requests_per_second));
        summary_lines.push(format!("- **P95 Latency**: {:.1}ms", result.performance_metrics.response_times.p95_ms));
        summary_lines.push(format!("- **Optimization Effectiveness**: {:.1}%", 
            result.optimization_effectiveness.optimization_impact.overall_optimization_effectiveness));
        summary_lines.push("".to_string());
    }
    
    fs::write("load_test_summary.md", summary_lines.join("\n"))?;
    
    Ok(())
}