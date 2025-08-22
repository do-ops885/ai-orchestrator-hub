#![allow(unused_imports)]

#![allow(missing_docs)]
#![allow(clippy::all)]

//! # Simple Verification System Demo
//!
//! This example demonstrates the lightweight verification system as an alternative
//! to the complex pair programming verification. Shows how to:
//!
//! 1. Create agents and tasks
//! 2. Execute tasks with simple verification
//! 3. Compare verification tiers (Quick, Standard, Thorough)
//! 4. Configure verification rules
//! 5. Monitor verification metrics
//!
//! Run with: `cargo run --example simple_verification_demo`

use multiagent_hive::{
    agents::{RuleType, VerificationRule},
    HiveCoordinator,
};
use serde_json::json;
use tokio;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Simple Verification System Demo");

    // Initialize the hive coordinator
    let hive = HiveCoordinator::new().await?;
    info!("✅ Hive coordinator initialized with ID: {}", hive.id);

    // Create some agents with different capabilities
    let agents = create_demo_agents(&hive).await?;
    info!("✅ Created {} agents", agents.len());

    // Create various types of tasks
    let tasks = create_demo_tasks(&hive).await?;
    info!("✅ Created {} tasks", tasks.len());

    // Demo 1: Basic simple verification
    info!("\n📋 Demo 1: Basic Simple Verification");
    demo_basic_verification(&hive, &tasks).await?;

    // Demo 2: Verification tiers comparison
    info!("\n📋 Demo 2: Verification Tiers Comparison");
    demo_verification_tiers(&hive, &tasks).await?;

    // Demo 3: Custom verification rules
    info!("\n📋 Demo 3: Custom Verification Rules");
    demo_custom_rules(&hive).await?;

    // Demo 4: Verification metrics
    info!("\n📋 Demo 4: Verification Metrics");
    demo_verification_metrics(&hive).await?;

    // Demo 5: Performance comparison
    info!("\n📋 Demo 5: Performance Comparison");
    demo_performance_comparison(&hive, &tasks).await?;

    info!("🎉 Simple Verification Demo completed successfully!");
    Ok(())
}

async fn create_demo_agents(hive: &HiveCoordinator) -> anyhow::Result<Vec<uuid::Uuid>> {
    let mut agents = Vec::new();

    // Create a data analyst agent
    let analyst_config = json!({
        "name": "DataAnalyst-1",
        "type": "specialist:data_analysis",
        "capabilities": [
            {
                "name": "data_analysis",
                "proficiency": 0.9,
                "learning_rate": 0.1
            },
            {
                "name": "report_writing",
                "proficiency": 0.8,
                "learning_rate": 0.15
            }
        ]
    });
    agents.push(hive.create_agent(analyst_config).await?);

    // Create a content writer agent
    let writer_config = json!({
        "name": "ContentWriter-1",
        "type": "specialist:content_writing",
        "capabilities": [
            {
                "name": "content_writing",
                "proficiency": 0.85,
                "learning_rate": 0.12
            },
            {
                "name": "editing",
                "proficiency": 0.75,
                "learning_rate": 0.1
            }
        ]
    });
    agents.push(hive.create_agent(writer_config).await?);

    // Create a general worker agent
    let worker_config = json!({
        "name": "GeneralWorker-1",
        "type": "worker",
        "capabilities": [
            {
                "name": "general_tasks",
                "proficiency": 0.7,
                "learning_rate": 0.2
            }
        ]
    });
    agents.push(hive.create_agent(worker_config).await?);

    // Create a coordinator agent (can be used as AI reviewer)
    let coordinator_config = json!({
        "name": "Coordinator-1",
        "type": "coordinator",
        "capabilities": [
            {
                "name": "coordination",
                "proficiency": 0.9,
                "learning_rate": 0.05
            },
            {
                "name": "quality_review",
                "proficiency": 0.85,
                "learning_rate": 0.08
            }
        ]
    });
    agents.push(hive.create_agent(coordinator_config).await?);

    Ok(agents)
}

async fn create_demo_tasks(hive: &HiveCoordinator) -> anyhow::Result<Vec<uuid::Uuid>> {
    let mut tasks = Vec::new();

    // Task 1: Data analysis task (should pass verification)
    let analysis_task = json!({
        "description": "Analyze customer satisfaction data and provide insights on improvement areas",
        "type": "data_analysis",
        "priority": 2,
        "required_capabilities": [
            {
                "name": "data_analysis",
                "min_proficiency": 0.7
            }
        ]
    });
    tasks.push(hive.create_task(analysis_task).await?);

    // Task 2: Content writing task (should pass with minor issues)
    let writing_task = json!({
        "description": "Write a brief article about renewable energy benefits",
        "type": "content_writing",
        "priority": 1,
        "required_capabilities": [
            {
                "name": "content_writing",
                "min_proficiency": 0.6
            }
        ]
    });
    tasks.push(hive.create_task(writing_task).await?);

    // Task 3: Critical task (should use thorough verification)
    let critical_task = json!({
        "description": "Generate safety protocol documentation for emergency procedures",
        "type": "documentation",
        "priority": 3, // Critical
        "required_capabilities": [
            {
                "name": "general_tasks",
                "min_proficiency": 0.5
            }
        ]
    });
    tasks.push(hive.create_task(critical_task).await?);

    // Task 4: Simple task (should use quick verification)
    let simple_task = json!({
        "description": "Create a simple status report",
        "type": "reporting",
        "priority": 0, // Low
        "required_capabilities": []
    });
    tasks.push(hive.create_task(simple_task).await?);

    Ok(tasks)
}

async fn demo_basic_verification(
    hive: &HiveCoordinator,
    tasks: &[uuid::Uuid],
) -> anyhow::Result<()> {
    if tasks.is_empty() {
        warn!("No tasks available for basic verification demo");
        return Ok(());
    }

    let task_id = tasks[0];
    let original_goal = Some("Provide actionable insights that help improve customer satisfaction");

    info!("Executing task with simple verification...");

    match hive
        .execute_task_with_simple_verification(task_id, original_goal)
        .await
    {
        Ok((execution_result, verification_result)) => {
            info!("✅ Task execution successful!");
            info!("   Execution success: {}", execution_result.success);
            info!(
                "   Verification status: {:?}",
                verification_result.verification_status
            );
            info!("   Overall score: {:.2}", verification_result.overall_score);
            info!(
                "   Goal alignment: {:.2}",
                verification_result.goal_alignment_score
            );
            info!(
                "   Verification tier: {:?}",
                verification_result.verification_tier
            );
            info!(
                "   Issues found: {}",
                verification_result.issues_found.len()
            );

            for issue in &verification_result.issues_found {
                info!(
                    "   - {:?} ({:?}): {}",
                    issue.severity, issue.issue_type, issue.description
                );
            }
        }
        Err(e) => {
            warn!("❌ Task execution failed: {}", e);
        }
    }

    Ok(())
}

async fn demo_verification_tiers(
    hive: &HiveCoordinator,
    tasks: &[uuid::Uuid],
) -> anyhow::Result<()> {
    info!("Demonstrating different verification tiers based on task priority...");

    for (i, &task_id) in tasks.iter().enumerate() {
        if i >= 3 {
            break;
        } // Limit to first 3 tasks

        info!("Task {}: Executing with automatic tier selection", i + 1);

        match hive
            .execute_task_with_simple_verification(task_id, None)
            .await
        {
            Ok((_, verification_result)) => {
                info!("   Tier used: {:?}", verification_result.verification_tier);
                info!(
                    "   Verification time: {}ms",
                    verification_result.verification_time_ms
                );
                info!("   Confidence: {:.2}", verification_result.confidence_score);
            }
            Err(e) => {
                warn!("   Failed: {}", e);
            }
        }
    }

    Ok(())
}

async fn demo_custom_rules(hive: &HiveCoordinator) -> anyhow::Result<()> {
    info!("Configuring custom verification rules...");

    let config = json!({
        "confidence_threshold": 0.8,
        "task_rules": {
            "data_analysis": {
                "required_keywords": ["analysis", "data", "insights"],
                "min_length": 100,
                "max_length": 2000
            },
            "content_writing": {
                "required_keywords": ["benefits", "renewable"],
                "min_sentiment": 0.1,
                "forbidden_words": ["bad", "terrible"]
            }
        }
    });

    hive.configure_simple_verification(config).await?;
    info!("✅ Custom verification rules configured");

    Ok(())
}

async fn demo_verification_metrics(hive: &HiveCoordinator) -> anyhow::Result<()> {
    info!("Retrieving verification system metrics...");

    let metrics = hive.get_simple_verification_stats().await;
    info!("📊 Verification Metrics:");
    info!("   Total verifications: {}", metrics["total_verifications"]);
    info!(
        "   Success rate: {:.1}%",
        metrics["success_rate"].as_f64().unwrap_or(0.0) * 100.0
    );
    info!(
        "   Average verification time: {:.1}ms",
        metrics["average_verification_time_ms"]
    );
    info!(
        "   Average confidence: {:.2}",
        metrics["average_confidence_score"]
    );

    if let Some(tier_usage) = metrics["tier_usage"].as_object() {
        info!("   Tier usage:");
        for (tier, count) in tier_usage {
            info!("     {}: {}", tier, count);
        }
    }

    Ok(())
}

async fn demo_performance_comparison(
    hive: &HiveCoordinator,
    tasks: &[uuid::Uuid],
) -> anyhow::Result<()> {
    if tasks.len() < 2 {
        warn!("Not enough tasks for performance comparison");
        return Ok(());
    }

    info!("Comparing simple verification vs pair programming verification...");

    // Simple verification timing
    let start_time = std::time::Instant::now();
    let task_id = tasks[tasks.len() - 1]; // Use last task

    match hive
        .execute_task_with_simple_verification(task_id, None)
        .await
    {
        Ok((_, verification_result)) => {
            let simple_duration = start_time.elapsed();
            info!("✅ Simple verification completed in {:?}", simple_duration);
            info!(
                "   Verification time: {}ms",
                verification_result.verification_time_ms
            );
            info!("   Total time: {:?}", simple_duration);
            info!("   Status: {:?}", verification_result.verification_status);
        }
        Err(e) => {
            warn!("❌ Simple verification failed: {}", e);
        }
    }

    // Note: Pair programming verification would require 2 agents and more complex setup
    info!("💡 Pair programming verification would require:");
    info!("   - 2x agent resources (primary + verifier)");
    info!("   - Additional coordination overhead");
    info!("   - Typically 2-5x longer execution time");
    info!("   - Higher confidence but at significant cost");

    Ok(())
}

/// Helper function to demonstrate verification rule creation
#[allow(dead_code)]
fn create_custom_verification_rules() -> Vec<VerificationRule> {
    vec![
        VerificationRule {
            rule_id: "content_length".to_string(),
            rule_type: RuleType::LengthCheck { min: 50, max: 1000 },
            threshold: 1.0,
            weight: 0.2,
            enabled: true,
        },
        VerificationRule {
            rule_id: "required_keywords".to_string(),
            rule_type: RuleType::KeywordPresence {
                keywords: vec!["analysis".to_string(), "insights".to_string()],
            },
            threshold: 0.5, // At least 50% of keywords must be present
            weight: 0.3,
            enabled: true,
        },
        VerificationRule {
            rule_id: "positive_tone".to_string(),
            rule_type: RuleType::SentimentCheck { min_sentiment: 0.0 },
            threshold: 1.0,
            weight: 0.1,
            enabled: true,
        },
        VerificationRule {
            rule_id: "no_profanity".to_string(),
            rule_type: RuleType::KeywordAbsence {
                forbidden_words: vec![
                    "bad".to_string(),
                    "terrible".to_string(),
                    "awful".to_string(),
                ],
            },
            threshold: 1.0,
            weight: 0.4,
            enabled: true,
        },
    ]
}
