#![allow(unused_imports)]

#![allow(missing_docs)]
#![allow(clippy::all)]

use multiagent_hive::neural::HybridNeuralProcessor;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the hybrid neural processor
    let mut neural_processor = HybridNeuralProcessor::new().await?;

    // Create two agents: one basic, one advanced
    let basic_agent = Uuid::new_v4();
    let advanced_agent = Uuid::new_v4();

    println!("🧠 Neural Processing Comparison Demo");
    println!("=====================================");

    // Create basic agent (uses current NLP.rs)
    neural_processor
        .create_neural_agent(
            basic_agent,
            "general".to_string(),
            false, // Use basic NLP
        )
        .await?;

    println!("✅ Created basic agent: {}", basic_agent);

    // Create advanced agent (uses ruv-FANN if available)
    #[cfg(feature = "advanced-neural")]
    {
        neural_processor
            .create_neural_agent(
                advanced_agent,
                "pattern_recognition".to_string(),
                true, // Use advanced neural networks
            )
            .await?;

        println!("🚀 Created advanced agent: {}", advanced_agent);
    }

    #[cfg(not(feature = "advanced-neural"))]
    {
        neural_processor
            .create_neural_agent(
                advanced_agent,
                "pattern_recognition".to_string(),
                false, // Falls back to basic NLP
            )
            .await?;

        println!(
            "📝 Created agent (advanced features not enabled): {}",
            advanced_agent
        );
    }

    // Test scenarios
    let test_scenarios = vec![
        ("Analyze customer sentiment in reviews", true),
        ("Process natural language commands", true),
        ("Coordinate with other agents", false),
        ("Learn from previous interactions", true),
        ("Handle complex pattern recognition", false),
    ];

    println!("\n🧪 Testing Scenarios:");
    println!("---------------------");

    for (scenario, expected_success) in test_scenarios {
        println!("\n📋 Scenario: {}", scenario);

        // Process with basic agent
        let basic_result = neural_processor
            .process_text_adaptive(scenario, basic_agent)
            .await?;
        println!("  📊 Basic Agent:");
        println!("    - Sentiment: {:.2}", basic_result.sentiment);
        println!("    - Keywords: {:?}", basic_result.keywords);

        // Process with advanced agent
        let advanced_result = neural_processor
            .process_text_adaptive(scenario, advanced_agent)
            .await?;
        println!("  🚀 Advanced Agent:");
        println!("    - Sentiment: {:.2}", advanced_result.sentiment);
        println!("    - Keywords: {:?}", advanced_result.keywords);

        // Learn from the interaction
        neural_processor
            .learn_from_interaction_adaptive(
                basic_agent,
                scenario,
                "Task completed successfully",
                expected_success,
            )
            .await?;

        neural_processor
            .learn_from_interaction_adaptive(
                advanced_agent,
                scenario,
                "Task completed successfully",
                expected_success,
            )
            .await?;

        // Get performance predictions
        let basic_performance = neural_processor
            .predict_performance(basic_agent, scenario)
            .await?;
        let advanced_performance = neural_processor
            .predict_performance(advanced_agent, scenario)
            .await?;

        println!("  📈 Performance Predictions:");
        println!("    - Basic: {:.2}%", basic_performance * 100.0);
        println!("    - Advanced: {:.2}%", advanced_performance * 100.0);
    }

    // Show final performance metrics
    println!("\n📊 Final Performance Metrics:");
    println!("-----------------------------");

    if let Some(basic_perf) = neural_processor.get_agent_performance(basic_agent) {
        println!(
            "📝 Basic Agent Overall Performance: {:.2}%",
            basic_perf * 100.0
        );
    }

    if let Some(advanced_perf) = neural_processor.get_agent_performance(advanced_agent) {
        println!(
            "🚀 Advanced Agent Overall Performance: {:.2}%",
            advanced_perf * 100.0
        );
    }

    println!("\n🎯 Recommendation:");
    println!("------------------");

    #[cfg(feature = "advanced-neural")]
    println!("✅ Advanced neural features are available!");
    println!("   Use advanced agents for:");
    println!("   - Complex pattern recognition");
    println!("   - Time series forecasting");
    println!("   - Large-scale swarm coordination");
    println!("   - Performance-critical tasks");

    #[cfg(not(feature = "advanced-neural"))]
    println!("📝 Using basic NLP features (recommended for most use cases)");
    println!("   Current system is excellent for:");
    println!("   - Real-time agent communication");
    println!("   - Lightweight swarm coordination");
    println!("   - Basic pattern learning");
    println!("   - Sentiment analysis");
    println!("\n   To enable advanced features, compile with:");
    println!("   cargo run --features advanced-neural --example neural_comparison");

    Ok(())
}
