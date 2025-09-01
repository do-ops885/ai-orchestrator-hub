//! # Agent Monitor Usage Example
//!
//! This example demonstrates how to use the comprehensive AgentMonitor system
//! for monitoring and analyzing the AI Orchestrator Hub.

use ai_orchestrator_hub::infrastructure::{
    monitoring::*, metrics::MetricsCollector, telemetry::TelemetryCollector,
};
use ai_orchestrator_hub::utils::config::MonitoringConfig;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Starting Agent Monitor Example");

    // Initialize core components
    let metrics_collector = Arc::new(MetricsCollector::new(1000));
    let telemetry_collector = Arc::new(TelemetryCollector::new(10000));

    // Configure monitoring
    let monitoring_config = MonitoringConfig {
        monitoring_interval_secs: 5,
        metrics_retention_days: 7,
        alert_threshold: 0.8,
        metrics_endpoint: "http://localhost:8000/metrics".to_string(),
        health_endpoint: "http://localhost:8000/health".to_string(),
        enable_agent_discovery: true,
        enable_health_monitoring: true,
        enable_performance_monitoring: true,
        enable_behavior_analysis: true,
        enable_dashboards: true,
        enable_alerting: true,
        enable_diagnostics: true,
        enable_reporting: true,
        enable_automation: true,
        enable_external_integration: true,
    };

    // Create agent monitor
    let agent_monitor = Arc::new(AgentMonitor::new(
        Arc::new(monitoring_config),
        Arc::clone(&metrics_collector),
        Arc::clone(&telemetry_collector),
    )?);

    // Start monitoring system
    agent_monitor.start().await?;
    println!("✅ Agent Monitor started successfully");

    // Example 1: Agent Discovery
    println!("\n🔍 Running Agent Discovery...");
    let discovered_agents = agent_monitor.agent_discovery.discover_agents().await?;
    println!("📊 Discovered {} agents", discovered_agents.len());

    for agent in &discovered_agents {
        println!("  - {} ({}) - Status: {:?}", agent.name, agent.agent_type, agent.status);
    }

    // Example 2: Health Monitoring
    println!("\n🏥 Running Health Monitoring...");
    let health_status = agent_monitor.health_monitor.get_health_status().await?;
    println!("📊 System Health: {:.1}%", health_status.overall_score * 100.0);
    println!("  - Healthy agents: {}", health_status.healthy_agents);
    println!("  - Warning agents: {}", health_status.warning_agents);
    println!("  - Critical agents: {}", health_status.critical_agents);

    // Example 3: Performance Monitoring
    println!("\n⚡ Running Performance Monitoring...");
    let performance_status = agent_monitor.performance_monitor.get_performance_status().await?;
    println!("📊 Performance Score: {:.1}%", performance_status.overall_score * 100.0);
    println!("  - Avg Response Time: {:.1}ms", performance_status.average_response_time_ms);
    println!("  - Throughput: {:.1} tasks/sec", performance_status.throughput_tasks_per_second);

    // Example 4: Behavior Analysis
    println!("\n🧠 Running Behavior Analysis...");
    let behavior_status = agent_monitor.behavior_analyzer.get_behavior_status().await?;
    println!("📊 Communication Efficiency: {:.1}%", behavior_status.communication_efficiency * 100.0);
    println!("📊 Decision Quality: {:.1}%", behavior_status.decision_quality_score * 100.0);
    println!("📊 Adaptation Rate: {:.1}%", behavior_status.adaptation_rate * 100.0);

    // Example 5: Dashboard Creation
    println!("\n📊 Setting up Monitoring Dashboard...");
    agent_monitor.dashboard.initialize_default_widgets().await?;
    let dashboard_data = agent_monitor.dashboard.generate_dashboard_data(&agent_monitor).await?;
    println!("📊 Dashboard created with {} widgets", dashboard_data["widgets"].as_array().unwrap().len());

    // Example 6: Diagnostics
    println!("\n🔧 Running System Diagnostics...");
    let diagnostics = agent_monitor.diagnostics.run_system_diagnostics().await?;
    println!("📊 System diagnostics completed");
    println!("  - Overall performance score: {:.1}%", diagnostics.system_performance_profile.overall_performance_score * 100.0);
    println!("  - Performance bottlenecks: {}", diagnostics.system_performance_profile.performance_bottlenecks.len());

    // Example 7: Alerting System
    println!("\n🚨 Testing Alerting System...");
    agent_monitor.alerting_system.test_alert_system().await?;
    println!("📊 Alert system test completed");

    // Example 8: Data Collection
    println!("\n📈 Running Data Collection...");
    println!("📊 Data collection configured and running in background");

    // Example 9: Reporting
    println!("\n📋 Generating Reports...");
    let health_report = agent_monitor.reporting.generate_health_report(24).await?;
    println!("📊 Health report generated: {}", health_report.title);

    let performance_report = agent_monitor.reporting.generate_performance_report(24).await?;
    println!("📊 Performance report generated: {}", performance_report.title);

    // Example 10: Automation Setup
    println!("\n🤖 Setting up Automation...");
    agent_monitor.automation.setup_default_automated_tasks().await?;
    agent_monitor.automation.setup_default_automation_schedules().await?;
    println!("📊 Automation system configured");

    // Example 11: External Integration
    println!("\n🔗 Setting up External Integrations...");

    // Prometheus integration
    agent_monitor.integration.setup_prometheus_integration("http://localhost:9090").await?;
    println!("📊 Prometheus integration configured");

    // Slack integration
    agent_monitor.integration.setup_slack_integration("https://hooks.slack.com/services/...", "#alerts").await?;
    println!("📊 Slack integration configured");

    // Example 12: Comprehensive Monitoring Status
    println!("\n📊 Getting Comprehensive Monitoring Status...");
    let monitoring_status = agent_monitor.get_monitoring_status().await?;
    println!("📊 Overall system health: {:.1}%", monitoring_status.overall_health * 100.0);
    println!("📊 Active alerts: {}", monitoring_status.active_alerts.total_alerts);

    println!("\n🎉 Agent Monitor Example completed successfully!");
    println!("💡 The monitoring system is now fully operational with:");
    println!("   - Agent discovery and cataloging");
    println!("   - Health monitoring with automated checks");
    println!("   - Performance monitoring with baselines");
    println!("   - Behavior analysis for communication patterns");
    println!("   - Real-time dashboards with multiple widgets");
    println!("   - Intelligent alerting with adaptive thresholds");
    println!("   - Comprehensive diagnostics and troubleshooting");
    println!("   - Automated data collection and retention");
    println!("   - Report generation and scheduling");
    println!("   - Automated tasks and schedules");
    println!("   - External system integrations");

    Ok(())
}