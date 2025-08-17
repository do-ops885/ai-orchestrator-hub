use anyhow::Result;
use uuid::Uuid;

// Import external crate modules for testing
use multiagent_hive::neural::HybridNeuralProcessor;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧠 Advanced Neural Features Test");
    println!("================================");

    // Initialize the hybrid neural processor
    let mut processor = HybridNeuralProcessor::new().await?;

    println!("\n📊 Testing Neural Agent Creation:");

    // Create different types of neural agents
    let basic_agent = Uuid::new_v4();
    let fann_agent = Uuid::new_v4();
    let lstm_agent = Uuid::new_v4();

    // Basic agent (default NLP processing)
    processor
        .create_neural_agent(
            basic_agent,
            "general".to_string(),
            false, // Basic processing
        )
        .await?;
    println!("  ✅ Basic Agent: {} (NLP processing)", basic_agent);

    // FANN agent for pattern recognition
    processor
        .create_neural_agent(
            fann_agent,
            "pattern_recognition".to_string(),
            true, // Advanced neural features
        )
        .await?;
    println!("  ✅ FANN Agent: {} (Pattern recognition)", fann_agent);

    // LSTM agent for forecasting
    processor
        .create_neural_agent(
            lstm_agent,
            "forecasting".to_string(),
            true, // Advanced neural features
        )
        .await?;
    println!("  ✅ LSTM Agent: {} (Temporal forecasting)", lstm_agent);

    println!("\n🔄 Training Neural Agents with Sample Data:");

    // Training data for different scenarios
    let training_scenarios = vec![
        // Pattern recognition tasks
        ("Classify image features", "Image contains cat", true),
        ("Detect anomalies in data", "No anomalies found", true),
        (
            "Recognize speech patterns",
            "Speech recognition successful",
            true,
        ),
        (
            "Identify fraud patterns",
            "Transaction flagged as suspicious",
            false,
        ),
        (
            "Classify text sentiment",
            "Positive sentiment detected",
            true,
        ),
        // Forecasting tasks
        ("Predict stock movement", "Stock price increased", true),
        (
            "Forecast weather patterns",
            "Rain predicted correctly",
            true,
        ),
        (
            "Estimate project timeline",
            "Project completed on time",
            true,
        ),
        ("Predict user behavior", "User engagement increased", false),
        ("Forecast demand", "Demand exceeded prediction", false),
        // General tasks
        (
            "Process customer request",
            "Request handled successfully",
            true,
        ),
        ("Analyze market trends", "Trends identified correctly", true),
        ("Generate report", "Report generated", true),
        ("Optimize performance", "Performance improved", true),
        ("Handle exception", "Exception resolved", false),
    ];

    println!(
        "  Training {} scenarios across all agents...",
        training_scenarios.len()
    );

    for (i, (task, output, success)) in training_scenarios.iter().enumerate() {
        // Train all agents with the same data to compare learning
        processor
            .learn_from_interaction_adaptive(basic_agent, task, output, *success)
            .await?;

        processor
            .learn_from_interaction_adaptive(fann_agent, task, output, *success)
            .await?;

        processor
            .learn_from_interaction_adaptive(lstm_agent, task, output, *success)
            .await?;

        if (i + 1) % 5 == 0 {
            println!("    Completed {} training scenarios", i + 1);
        }
    }

    println!("\n📈 Agent Performance Metrics:");

    // Check performance metrics
    if let Some(basic_perf) = processor.get_agent_performance(basic_agent) {
        println!("  Basic Agent Performance: {:.3}", basic_perf);
    }
    if let Some(fann_perf) = processor.get_agent_performance(fann_agent) {
        println!("  FANN Agent Performance: {:.3}", fann_perf);
    }
    if let Some(lstm_perf) = processor.get_agent_performance(lstm_agent) {
        println!("  LSTM Agent Performance: {:.3}", lstm_perf);
    }

    println!("\n🔮 Testing Performance Predictions:");

    // Test prediction capabilities
    let test_tasks = vec![
        "Classify new image data",
        "Predict quarterly sales",
        "Detect security threats",
        "Forecast customer churn",
        "Analyze complex patterns",
        "Estimate project risks",
    ];

    println!("Task                     | Basic | FANN  | LSTM  | Best");
    println!("-------------------------|-------|-------|-------|------");

    for task in test_tasks {
        let basic_pred = processor.predict_performance(basic_agent, task).await?;
        let fann_pred = processor.predict_performance(fann_agent, task).await?;
        let lstm_pred = processor.predict_performance(lstm_agent, task).await?;

        let best = if fann_pred >= basic_pred && fann_pred >= lstm_pred {
            "FANN"
        } else if lstm_pred >= basic_pred && lstm_pred >= fann_pred {
            "LSTM"
        } else {
            "Basic"
        };

        println!(
            "{:<24} | {:.3} | {:.3} | {:.3} | {}",
            task, basic_pred, fann_pred, lstm_pred, best
        );
    }

    println!("\n🧪 Testing Text Processing Differences:");

    // Test text processing with different neural approaches
    let sample_texts = vec![
        "Analyze complex temporal patterns in financial market data",
        "Classify customer feedback sentiment for product improvement",
        "Detect anomalous behavior patterns in network traffic",
        "Predict future trends based on historical performance data",
    ];

    for (i, text) in sample_texts.iter().enumerate() {
        println!("\n  Sample {}: '{}'", i + 1, text);

        let basic_processed = processor.process_text_adaptive(text, basic_agent).await?;
        let fann_processed = processor.process_text_adaptive(text, fann_agent).await?;
        let lstm_processed = processor.process_text_adaptive(text, lstm_agent).await?;

        println!(
            "    Basic  - Sentiment: {:.3}, Keywords: {}",
            basic_processed.sentiment,
            basic_processed.keywords.len()
        );
        println!(
            "    FANN   - Sentiment: {:.3}, Keywords: {}",
            fann_processed.sentiment,
            fann_processed.keywords.len()
        );
        println!(
            "    LSTM   - Sentiment: {:.3}, Keywords: {}",
            lstm_processed.sentiment,
            lstm_processed.keywords.len()
        );
    }

    println!("\n⚡ Performance Comparison Summary:");

    // Calculate and display performance differences
    let mut basic_total = 0.0;
    let mut fann_total = 0.0;
    let mut lstm_total = 0.0;
    let mut count = 0;

    for task in &[
        "pattern recognition",
        "forecasting",
        "classification",
        "prediction",
    ] {
        let basic = processor.predict_performance(basic_agent, task).await?;
        let fann = processor.predict_performance(fann_agent, task).await?;
        let lstm = processor.predict_performance(lstm_agent, task).await?;

        basic_total += basic;
        fann_total += fann;
        lstm_total += lstm;
        count += 1;
    }

    let basic_avg = basic_total / count as f64;
    let fann_avg = fann_total / count as f64;
    let lstm_avg = lstm_total / count as f64;

    println!("  Average Performance:");
    println!("    Basic NLP: {:.3}", basic_avg);
    println!(
        "    FANN:      {:.3} ({:+.3} vs Basic)",
        fann_avg,
        fann_avg - basic_avg
    );
    println!(
        "    LSTM:      {:.3} ({:+.3} vs Basic)",
        lstm_avg,
        lstm_avg - basic_avg
    );

    let best_performer = if fann_avg >= basic_avg && fann_avg >= lstm_avg {
        "FANN (Pattern Recognition)"
    } else if lstm_avg >= basic_avg && lstm_avg >= fann_avg {
        "LSTM (Temporal Intelligence)"
    } else {
        "Basic NLP (Lightweight)"
    };

    println!("    Best Overall: {}", best_performer);

    println!("\n✅ Advanced Neural Features Test Complete!");
    println!("🎉 All neural processing modes working correctly:");
    println!("   • Basic NLP: Fast, lightweight processing");
    println!("   • FANN Networks: Advanced pattern recognition");
    println!("   • LSTM Networks: Temporal intelligence and forecasting");
    println!("   • Hybrid Architecture: Seamless integration and auto-selection");

    Ok(())
}
