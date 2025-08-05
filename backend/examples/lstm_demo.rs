// Internal module imports for example
use uuid::Uuid;
use anyhow::Result;

// Since this is an example within the crate, we need to reference the modules directly
mod neural;
mod nlp;
mod cpu_optimization;

use neural::HybridNeuralProcessor;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧠 LSTM Neural Processing Demo");
    println!("==============================");
    
    // Initialize the hybrid neural processor
    let mut processor = HybridNeuralProcessor::new().await?;
    
    // Create agents with different neural capabilities
    let basic_agent = Uuid::new_v4();
    let lstm_agent = Uuid::new_v4();
    
    // Create basic agent
    processor.create_neural_agent(
        basic_agent,
        "general".to_string(),
        false, // Use basic processing
    ).await?;
    
    // Create LSTM agent for forecasting
    processor.create_neural_agent(
        lstm_agent,
        "forecasting".to_string(),
        true, // Use advanced LSTM processing
    ).await?;
    
    println!("\n📊 Created agents:");
    println!("  • Basic Agent: {}", basic_agent);
    println!("  • LSTM Agent: {}", lstm_agent);
    
    // Simulate a sequence of tasks and learning
    let tasks = vec![
        ("Analyze market trends", true),
        ("Predict stock movement", false),
        ("Forecast weather patterns", true),
        ("Estimate project timeline", true),
        ("Predict user behavior", false),
        ("Analyze sales data", true),
        ("Forecast demand", true),
        ("Predict system load", true),
    ];
    
    println!("\n🔄 Training sequence:");
    for (i, (task, success)) in tasks.iter().enumerate() {
        println!("  {}. Task: '{}' -> {}", 
                i + 1, task, if *success { "✅ Success" } else { "❌ Failure" });
        
        // Train both agents
        processor.learn_from_interaction_adaptive(
            basic_agent,
            task,
            "task completed",
            *success,
        ).await?;
        
        processor.learn_from_interaction_adaptive(
            lstm_agent,
            task,
            "task completed",
            *success,
        ).await?;
    }
    
    // Test predictions
    let test_tasks = vec![
        "Predict quarterly revenue",
        "Analyze customer churn",
        "Forecast inventory needs",
        "Estimate project risks",
    ];
    
    println!("\n🔮 Performance Predictions:");
    println!("Task                     | Basic Agent | LSTM Agent | Difference");
    println!("-------------------------|-------------|------------|----------");
    
    for task in test_tasks {
        let basic_prediction = processor.predict_performance(basic_agent, task).await?;
        let lstm_prediction = processor.predict_performance(lstm_agent, task).await?;
        let difference = lstm_prediction - basic_prediction;
        
        println!("{:<24} | {:<11.3} | {:<10.3} | {:+.3}",
                task, basic_prediction, lstm_prediction, difference);
    }
    
    // Show agent performance metrics
    println!("\n📈 Agent Performance Metrics:");
    if let Some(basic_perf) = processor.get_agent_performance(basic_agent) {
        println!("  • Basic Agent: {:.3}", basic_perf);
    }
    if let Some(lstm_perf) = processor.get_agent_performance(lstm_agent) {
        println!("  • LSTM Agent: {:.3}", lstm_perf);
    }
    
    // Demonstrate text processing differences
    println!("\n🔍 Text Processing Comparison:");
    let sample_text = "Analyze complex temporal patterns in financial data";
    
    let basic_processed = processor.process_text_adaptive(sample_text, basic_agent).await?;
    let lstm_processed = processor.process_text_adaptive(sample_text, lstm_agent).await?;
    
    println!("Sample text: '{}'", sample_text);
    println!("Basic processing:");
    println!("  • Sentiment: {:.3}", basic_processed.sentiment);
    println!("  • Keywords: {:?}", basic_processed.keywords);
    println!("  • Vector magnitude: {:.3}", basic_processed.semantic_vector.magnitude);
    
    println!("LSTM processing:");
    println!("  • Sentiment: {:.3}", lstm_processed.sentiment);
    println!("  • Keywords: {:?}", lstm_processed.keywords);
    println!("  • Vector magnitude: {:.3}", lstm_processed.semantic_vector.magnitude);
    
    println!("\n✅ LSTM Demo completed!");
    println!("The LSTM agent shows sequence-aware learning and prediction capabilities.");
    
    Ok(())
}