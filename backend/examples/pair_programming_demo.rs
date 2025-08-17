//! # Pair Programming Verification Demo
//!
//! Demonstrates the pair programming verification system where every task
//! is executed by a primary agent and independently verified by a verification agent.

use anyhow::Result;
use multiagent_hive::{
    Agent, AgentCapability, AgentType, HiveCoordinator, OverallTaskStatus, VerificationLevel,
    VerifiedTaskResult,
};
use serde_json::json;
use tracing::{error, info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Pair Programming Verification Demo");

    // Create hive coordinator
    let hive = HiveCoordinator::new().await?;
    info!("✅ Hive coordinator initialized");

    // Create agents for pair programming
    let primary_agent_id = create_primary_agent(&hive).await?;
    let verification_agent_id = create_verification_agent(&hive).await?;
    let backup_agent_id = create_backup_agent(&hive).await?;

    info!("✅ Created 3 agents for demonstration");

    // Create agent pairs
    let pair_id = hive
        .create_agent_pair(primary_agent_id, verification_agent_id)
        .await?;
    info!("✅ Created agent pair: {}", pair_id);

    // Demo 1: Simple task with verification
    info!("\n🔍 Demo 1: Simple Task Verification");
    let simple_task_result = demo_simple_task_verification(&hive).await?;
    print_verification_result("Simple Task", &simple_task_result);

    // Demo 2: Complex task with comprehensive verification
    info!("\n🔍 Demo 2: Complex Task with Comprehensive Verification");
    let complex_task_result = demo_complex_task_verification(&hive).await?;
    print_verification_result("Complex Task", &complex_task_result);

    // Demo 3: Task that should fail verification
    info!("\n🔍 Demo 3: Task Designed to Fail Verification");
    let failing_task_result = demo_failing_task_verification(&hive).await?;
    print_verification_result("Failing Task", &failing_task_result);

    // Demo 4: Show pair programming statistics
    info!("\n📊 Pair Programming Statistics");
    let stats = hive.get_pair_programming_stats().await;
    println!(
        "Pair Programming Stats:\n{}",
        serde_json::to_string_pretty(&stats)?
    );

    // Demo 5: Show overall hive status
    info!("\n🏠 Overall Hive Status");
    let hive_status = hive.get_status().await;
    println!(
        "Hive Status:\n{}",
        serde_json::to_string_pretty(&hive_status)?
    );

    info!("🎉 Pair Programming Verification Demo completed successfully!");

    Ok(())
}

async fn create_primary_agent(hive: &HiveCoordinator) -> Result<Uuid> {
    let config = json!({
        "name": "Primary-Worker-Alpha",
        "type": "worker",
        "capabilities": [
            {
                "name": "data_processing",
                "proficiency": 0.85,
                "learning_rate": 0.1
            },
            {
                "name": "analysis",
                "proficiency": 0.75,
                "learning_rate": 0.12
            }
        ]
    });

    hive.create_agent(config).await
}

async fn create_verification_agent(hive: &HiveCoordinator) -> Result<Uuid> {
    let config = json!({
        "name": "Verifier-Beta",
        "type": "learner",
        "capabilities": [
            {
                "name": "verification",
                "proficiency": 0.90,
                "learning_rate": 0.08
            },
            {
                "name": "quality_assessment",
                "proficiency": 0.88,
                "learning_rate": 0.09
            },
            {
                "name": "goal_alignment",
                "proficiency": 0.82,
                "learning_rate": 0.11
            }
        ]
    });

    hive.create_agent(config).await
}

async fn create_backup_agent(hive: &HiveCoordinator) -> Result<Uuid> {
    let config = json!({
        "name": "Backup-Gamma",
        "type": "coordinator",
        "capabilities": [
            {
                "name": "coordination",
                "proficiency": 0.80,
                "learning_rate": 0.10
            },
            {
                "name": "oversight",
                "proficiency": 0.85,
                "learning_rate": 0.08
            }
        ]
    });

    hive.create_agent(config).await
}

async fn demo_simple_task_verification(hive: &HiveCoordinator) -> Result<VerifiedTaskResult> {
    let task_config = json!({
        "description": "Process customer data and generate summary report",
        "type": "data_processing",
        "priority": 1,
        "original_goal": "Create a comprehensive summary of customer data that highlights key insights and trends",
        "verification_level": "standard",
        "required_capabilities": [
            {
                "name": "data_processing",
                "min_proficiency": 0.7
            }
        ]
    });

    let task_id = hive.create_verifiable_task(task_config).await?;
    hive.execute_task_with_verification(task_id).await
}

async fn demo_complex_task_verification(hive: &HiveCoordinator) -> Result<VerifiedTaskResult> {
    let task_config = json!({
        "description": "Analyze market trends and provide strategic recommendations for Q4",
        "type": "analysis",
        "priority": 2,
        "original_goal": "Deliver actionable strategic recommendations based on thorough market trend analysis that will guide Q4 business decisions",
        "verification_level": "comprehensive",
        "required_capabilities": [
            {
                "name": "analysis",
                "min_proficiency": 0.8
            }
        ]
    });

    let task_id = hive.create_verifiable_task(task_config).await?;
    hive.execute_task_with_verification(task_id).await
}

async fn demo_failing_task_verification(hive: &HiveCoordinator) -> Result<VerifiedTaskResult> {
    let task_config = json!({
        "description": "Perform advanced quantum computing simulation",
        "type": "quantum_simulation",
        "priority": 3,
        "original_goal": "Execute a comprehensive quantum computing simulation that demonstrates quantum supremacy and provides detailed analysis of quantum algorithms",
        "verification_level": "comprehensive",
        "required_capabilities": [
            {
                "name": "quantum_computing",
                "min_proficiency": 0.95
            }
        ]
    });

    let task_id = hive.create_verifiable_task(task_config).await?;

    // This should fail because no agent has quantum computing capabilities
    match hive.execute_task_with_verification(task_id).await {
        Ok(result) => Ok(result),
        Err(e) => {
            warn!("Task failed as expected: {}", e);
            // Create a mock failed result for demonstration
            create_mock_failed_result(task_id)
        }
    }
}

fn create_mock_failed_result(task_id: Uuid) -> Result<VerifiedTaskResult> {
    use chrono::Utc;
    use multiagent_hive::{
        Discrepancy, DiscrepancySeverity, TaskResult, VerificationDetails, VerificationMethod,
        VerificationResult, VerificationStatus,
    };
    use std::collections::HashMap;

    let execution_result = TaskResult {
        task_id,
        agent_id: Uuid::new_v4(),
        success: false,
        output: "Failed to execute quantum simulation - insufficient capabilities".to_string(),
        error_message: Some("Agent lacks required quantum computing capabilities".to_string()),
        execution_time: 5000,
        completed_at: Utc::now(),
        quality_score: Some(0.1),
        learned_insights: vec!["Need specialized quantum computing agents".to_string()],
    };

    let verification_result = VerificationResult {
        verification_id: Uuid::new_v4(),
        task_id,
        verifier_agent: Uuid::new_v4(),
        verification_status: VerificationStatus::Failed,
        goal_alignment_score: 0.0,
        quality_score: 0.1,
        independent_assessment: "Task execution failed due to capability mismatch. Agent could not perform quantum computing simulation.".to_string(),
        discrepancies_found: vec![
            Discrepancy {
                discrepancy_id: Uuid::new_v4(),
                criterion_id: Uuid::new_v4(),
                expected: "Quantum computing simulation execution".to_string(),
                actual: "Capability not available".to_string(),
                severity: DiscrepancySeverity::Critical,
                description: "Required quantum computing capability not present in any available agent".to_string(),
            }
        ],
        verification_confidence: 0.95,
        method_used: VerificationMethod::GoalAlignment,
        timestamp: Utc::now(),
        verification_details: VerificationDetails {
            criteria_scores: HashMap::new(),
            method_specific_data: HashMap::new(),
            reasoning: "Verification confirmed task failure due to missing capabilities".to_string(),
            alternative_approaches_considered: vec!["Capability substitution".to_string()],
            confidence_factors: vec!["Clear capability mismatch".to_string()],
        },
    };

    Ok(VerifiedTaskResult::new(
        execution_result,
        verification_result,
    ))
}

fn print_verification_result(task_name: &str, result: &VerifiedTaskResult) {
    println!("\n📋 {} Results:", task_name);
    println!("   Overall Status: {:?}", result.overall_status);
    println!(
        "   Final Confidence: {:.1}%",
        result.final_confidence * 100.0
    );
    println!("   Meets Requirements: {}", result.meets_requirements);

    println!("\n   Execution Result:");
    println!("     Success: {}", result.execution_result.success);
    println!(
        "     Output: {}",
        result
            .execution_result
            .output
            .chars()
            .take(100)
            .collect::<String>()
    );
    if let Some(error) = &result.execution_result.error_message {
        println!("     Error: {}", error);
    }

    println!("\n   Verification Result:");
    println!(
        "     Status: {:?}",
        result.verification_result.verification_status
    );
    println!(
        "     Goal Alignment: {:.1}%",
        result.verification_result.goal_alignment_score * 100.0
    );
    println!(
        "     Quality Score: {:.1}%",
        result.verification_result.quality_score * 100.0
    );
    println!(
        "     Verification Confidence: {:.1}%",
        result.verification_result.verification_confidence * 100.0
    );
    println!(
        "     Assessment: {}",
        result.verification_result.independent_assessment
    );

    if !result.verification_result.discrepancies_found.is_empty() {
        println!("\n   Discrepancies Found:");
        for (i, discrepancy) in result
            .verification_result
            .discrepancies_found
            .iter()
            .enumerate()
        {
            println!(
                "     {}. {} (Severity: {:?})",
                i + 1,
                discrepancy.description,
                discrepancy.severity
            );
        }
    }

    // Color-coded status indicator
    let status_indicator = match result.overall_status {
        OverallTaskStatus::FullyVerified => "✅ FULLY VERIFIED",
        OverallTaskStatus::ExecutedButUnverified => "⚠️  EXECUTED BUT UNVERIFIED",
        OverallTaskStatus::ExecutionFailed => "❌ EXECUTION FAILED",
        OverallTaskStatus::VerificationFailed => "🔍 VERIFICATION FAILED",
        OverallTaskStatus::VerificationError => "⚡ VERIFICATION ERROR",
        OverallTaskStatus::RequiresReview => "👁️  REQUIRES REVIEW",
    };

    println!("\n   Status: {}", status_indicator);
    println!("   {}", "=".repeat(60));
}
