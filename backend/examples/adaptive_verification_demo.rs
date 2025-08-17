//! # Adaptive Verification System Demo
//!
//! Demonstrates the machine learning-enhanced verification system that automatically
//! optimizes verification thresholds based on historical performance data.

use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use multiagent_hive::{
    AgentMemory,
    agents::{
        Agent, AgentCapability, AgentState, AgentType,
        adaptive_verification::{
            AdaptationConfig, AdaptiveVerificationCapable, AdaptiveVerificationSystem,
        },
        simple_verification::SimpleVerificationSystem,
    },
    neural::{
        NLPProcessor,
        adaptive_learning::{AdaptiveLearningConfig, AdaptiveLearningSystem},
    },
    tasks::{Task, TaskPriority, TaskResult, TaskStatus},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🧠 Starting Adaptive Verification System Demo");
    info!("=====================================");

    // Initialize core systems
    let nlp_processor = Arc::new(NLPProcessor::new().await?);
    let learning_system = Arc::new(RwLock::new(
        AdaptiveLearningSystem::new(AdaptiveLearningConfig::default()).await?,
    ));

    // Create base verification system
    let base_verification = SimpleVerificationSystem::new(nlp_processor.clone());

    // Create adaptive verification system with custom configuration
    let adaptation_config = AdaptationConfig {
        learning_rate: 0.1,
        min_samples_for_adaptation: 5, // Lower for demo
        adaptation_window_hours: 1,    // Shorter for demo
        adaptation_frequency_hours: 1, // More frequent for demo
        ..Default::default()
    };

    let mut adaptive_verification = AdaptiveVerificationSystem::new(
        base_verification,
        learning_system.clone(),
        adaptation_config,
    )
    .await;

    info!("✅ Adaptive verification system initialized");

    // Create test agent
    let test_agent = create_test_agent();
    info!("✅ Created test agent: {}", test_agent.name);

    // Demo 1: Initial verification without adaptation
    info!("\n📋 Demo 1: Initial Verification (No Adaptation)");
    info!("-----------------------------------------------");

    let initial_tasks = create_demo_tasks();
    for (i, (task, result, expected_success)) in initial_tasks.iter().enumerate() {
        info!("Task {}: {}", i + 1, task.description);

        let verification_result = test_agent
            .adaptive_verify(
                task,
                result,
                Some(&task.description),
                &mut adaptive_verification,
            )
            .await?;

        info!(
            "  Verification: {:?} (confidence: {:.2}, overall: {:.2})",
            verification_result.verification_status,
            verification_result.confidence_score,
            verification_result.overall_score
        );

        info!(
            "  Expected success: {}, Actual result success: {}",
            expected_success, result.success
        );
    }

    // Demo 2: Show learning in progress
    info!("\n📋 Demo 2: Learning from Additional Samples");
    info!("--------------------------------------------");

    // Generate more training data
    let training_tasks = generate_training_data();
    for (i, (task, result, expected_success)) in training_tasks.iter().enumerate() {
        let verification_result = test_agent
            .adaptive_verify(
                task,
                result,
                Some(&task.description),
                &mut adaptive_verification,
            )
            .await?;

        if i % 3 == 0 {
            // Show progress every few samples
            info!(
                "Training sample {}: verification={:?}, expected={}, actual={}",
                i + 1,
                verification_result.verification_status,
                expected_success,
                result.success
            );
        }
    }

    // Demo 3: Show adaptation insights
    info!("\n📋 Demo 3: Adaptation Insights");
    info!("------------------------------");

    let insights = adaptive_verification.get_adaptation_insights().await;
    info!("📊 Adaptation Statistics:");
    info!("    Total adaptations: {}", insights.total_adaptations);
    info!(
        "    Current performance score: {:.3}",
        insights.current_performance_score
    );
    info!("    Recent sample count: {}", insights.recent_sample_count);
    info!("    Accuracy metrics:");
    info!(
        "      - Precision: {:.3}",
        insights.accuracy_metrics.precision
    );
    info!("      - Recall: {:.3}", insights.accuracy_metrics.recall);
    info!(
        "      - F1 Score: {:.3}",
        insights.accuracy_metrics.f1_score
    );
    info!(
        "    Next adaptation due: {}",
        insights.next_adaptation_due.format("%H:%M:%S")
    );

    // Demo 4: Test with adapted thresholds
    info!("\n📋 Demo 4: Verification with Adapted Thresholds");
    info!("-----------------------------------------------");

    let test_tasks = create_demo_tasks();
    for (i, (task, result, expected_success)) in test_tasks.iter().enumerate() {
        info!("Task {}: {}", i + 1, task.description);

        let verification_result = test_agent
            .adaptive_verify(
                task,
                result,
                Some(&task.description),
                &mut adaptive_verification,
            )
            .await?;

        info!(
            "  Verification: {:?} (confidence: {:.2}, overall: {:.2})",
            verification_result.verification_status,
            verification_result.confidence_score,
            verification_result.overall_score
        );

        let correct_prediction = match verification_result.verification_status {
            multiagent_hive::agents::simple_verification::SimpleVerificationStatus::Passed |
            multiagent_hive::agents::simple_verification::SimpleVerificationStatus::PassedWithIssues => result.success,
            _ => !result.success,
        };

        info!(
            "  Prediction accuracy: {} (expected: {}, got: {})",
            if correct_prediction {
                "✅ Correct"
            } else {
                "❌ Incorrect"
            },
            expected_success,
            result.success
        );
    }

    // Demo 5: Learning insights from the adaptive learning system
    info!("\n📋 Demo 5: Neural Learning Insights");
    info!("-----------------------------------");

    let learning_insights = learning_system
        .read()
        .await
        .get_learning_insights(test_agent.id)
        .await;

    info!("🧠 Learning System Statistics:");
    info!(
        "    Total patterns learned: {}",
        learning_insights.total_patterns
    );
    info!(
        "    High-confidence patterns: {}",
        learning_insights.high_confidence_patterns
    );
    info!(
        "    Average confidence: {:.3}",
        learning_insights.average_confidence
    );
    info!(
        "    Recent learning events: {}",
        learning_insights.recent_learning_events
    );
    info!(
        "    Learning velocity: {:.2} events/min",
        learning_insights.learning_velocity
    );

    // Demo 6: Performance comparison
    info!("\n📋 Demo 6: Performance Comparison");
    info!("---------------------------------");

    info!("💡 Adaptive Verification Benefits:");
    info!("   ✅ Automatic threshold optimization based on real performance");
    info!("   ✅ Continuous learning from verification outcomes");
    info!("   ✅ Improved accuracy through machine learning");
    info!("   ✅ Reduced false positives and false negatives");
    info!("   ✅ Performance tracking and insights");
    info!("   ✅ Configurable adaptation parameters");

    info!("\n🔄 Traditional vs Adaptive Verification:");
    info!("   Traditional: Fixed thresholds, manual tuning required");
    info!("   Adaptive:    Self-optimizing thresholds, automatic improvement");

    info!("\n🎉 Adaptive Verification Demo completed successfully!");
    info!(
        "💡 The system has learned from {} verification samples and adapted its thresholds for better performance.",
        insights.recent_sample_count
    );

    Ok(())
}

fn create_test_agent() -> Agent {
    Agent {
        id: Uuid::new_v4(),
        name: "AdaptiveTestAgent".to_string(),
        agent_type: AgentType::Specialist("verification_testing".to_string()),
        state: AgentState::Idle,
        capabilities: vec![
            AgentCapability {
                name: "text_analysis".to_string(),
                proficiency: 0.8,
                learning_rate: 0.1,
            },
            AgentCapability {
                name: "quality_assessment".to_string(),
                proficiency: 0.7,
                learning_rate: 0.15,
            },
        ],
        position: (0.0, 0.0),
        energy: 1.0,
        memory: AgentMemory::new(),
        created_at: Utc::now(),
        last_active: Utc::now(),
    }
}

fn create_demo_tasks() -> Vec<(Task, TaskResult, bool)> {
    vec![
        // High-quality successful task
        (
            Task {
                id: Uuid::new_v4(),
                title: "Analyze customer feedback and provide insights on improvement areas".to_string(),
                description: "Analyze customer feedback and provide insights on improvement areas".to_string(),
                task_type: "analysis".to_string(),
                priority: TaskPriority::High,
                status: TaskStatus::Completed,
                required_capabilities: Vec::new(),
                assigned_agent: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deadline: None,
                estimated_duration: None,
                context: std::collections::HashMap::new(),
                dependencies: Vec::new(),
            },
            TaskResult {
                task_id: Uuid::new_v4(),
                success: true,
                output: "Comprehensive analysis of customer feedback reveals three key improvement areas: 1) Response time - customers expect faster support responses, 2) Product documentation - users need clearer setup guides, 3) Feature requests - integration with popular tools is highly requested. Recommendations include implementing automated response systems, creating video tutorials, and prioritizing API development.".to_string(),
                error_message: None,
                execution_time: 2500,
                quality_score: Some(0.9),
                agent_id: Uuid::new_v4(),
                completed_at: Utc::now(),
                learned_insights: Vec::new(),
            },
            true,
        ),
        // Poor quality task that should fail
        (
            Task {
                id: Uuid::new_v4(),
                title: "Write a comprehensive report on renewable energy benefits".to_string(),
                description: "Write a comprehensive report on renewable energy benefits".to_string(),
                task_type: "content_writing".to_string(),
                priority: TaskPriority::Medium,
                status: TaskStatus::Completed,
                required_capabilities: Vec::new(),
                assigned_agent: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deadline: None,
                estimated_duration: None,
                context: std::collections::HashMap::new(),
                dependencies: Vec::new(),
            },
            TaskResult {
                task_id: Uuid::new_v4(),
                success: true,
                output: "Energy is good. Solar panels work. Wind turbines spin. The end.".to_string(),
                error_message: None,
                execution_time: 500,
                quality_score: Some(0.2),
                agent_id: Uuid::new_v4(),
                completed_at: Utc::now(),
                learned_insights: Vec::new(),
            },
            false, // Should be considered failed due to poor quality
        ),
        // Medium quality task
        (
            Task {
                id: Uuid::new_v4(),
                title: "Create a status report for the development team".to_string(),
                description: "Create a status report for the development team".to_string(),
                task_type: "reporting".to_string(),
                priority: TaskPriority::Low,
                status: TaskStatus::Completed,
                required_capabilities: Vec::new(),
                assigned_agent: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deadline: None,
                estimated_duration: None,
                context: std::collections::HashMap::new(),
                dependencies: Vec::new(),
            },
            TaskResult {
                task_id: Uuid::new_v4(),
                success: true,
                output: "Development Status Report: Current sprint progress is 75% complete. Three features delivered this week: user authentication, data export, and notification system. One blocker identified: database migration needs additional testing. Next week priorities: complete migration testing and begin UI improvements.".to_string(),
                error_message: None,
                execution_time: 1800,
                quality_score: Some(0.7),
                agent_id: Uuid::new_v4(),
                completed_at: Utc::now(),
                learned_insights: Vec::new(),
            },
            true,
        ),
    ]
}

fn generate_training_data() -> Vec<(Task, TaskResult, bool)> {
    let mut training_data = Vec::new();

    // Generate 15 training samples with varied quality and outcomes
    for i in 0..15 {
        let (quality_score, output_quality, expected_success) = match i % 3 {
            0 => (0.9, "high", true),   // High quality
            1 => (0.3, "low", false),   // Low quality
            _ => (0.7, "medium", true), // Medium quality
        };

        let task = Task {
            id: Uuid::new_v4(),
            title: format!(
                "Training task {} - {} quality expected",
                i + 1,
                output_quality
            ),
            description: format!(
                "Training task {} - {} quality expected",
                i + 1,
                output_quality
            ),
            task_type: "training".to_string(),
            priority: TaskPriority::Medium,
            status: TaskStatus::Completed,
            required_capabilities: Vec::new(),
            assigned_agent: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deadline: None,
            estimated_duration: None,
            context: std::collections::HashMap::new(),
            dependencies: Vec::new(),
        };

        let output = match output_quality {
            "high" => format!(
                "Comprehensive and detailed analysis for training task {}. This output demonstrates high quality with thorough coverage of the topic, clear structure, and actionable insights. The analysis includes multiple perspectives and well-reasoned conclusions.",
                i + 1
            ),
            "low" => format!("Task {} done. Short answer.", i + 1),
            _ => format!(
                "Training task {} completed with adequate detail. The analysis covers the main points and provides reasonable insights, though it could be more comprehensive.",
                i + 1
            ),
        };

        let result = TaskResult {
            task_id: task.id,
            success: true,
            output,
            error_message: None,
            execution_time: (1000 + i * 100) as u64,
            quality_score: Some(quality_score),
            agent_id: Uuid::new_v4(),
            completed_at: Utc::now(),
            learned_insights: Vec::new(),
        };

        training_data.push((task, result, expected_success));
    }

    training_data
}
