//! # Simple Multi-Modal Agent Demo
//!
//! A simplified demonstration showing the Multi-Modal Agent in action

use multiagent_hive::agents::{DataModality, MultiModalAgent};
use multiagent_hive::neural::NLPProcessor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple Multi-Modal Agent Demo");
    println!("================================\n");

    // Create the agent
    let nlp_processor = NLPProcessor::new().await?;
    let mut agent = MultiModalAgent::new("SimpleDemo".to_string(), Some(nlp_processor)).await?;

    println!("✅ Created agent: {}", agent.base.name);

    // Test 1: Analyze some text
    println!("\n📝 Testing text analysis:");
    let text = "This is a positive message about our successful project implementation.";
    match agent.analyze_multimodal_data(text).await {
        Ok(analysis) => {
            println!("   Detected modalities: {:?}", analysis.detected_modalities);
            println!("   Quality score: {:.2}", analysis.overall_quality);
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Test 2: Analyze some code
    println!("\n💻 Testing code analysis:");
    let code = r#"
    function greet(name) {
        if (name) {
            return "Hello, " + name + "!";
        }
        return "Hello, World!";
    }
    "#;
    match agent.analyze_multimodal_data(code).await {
        Ok(analysis) => {
            println!("   Detected modalities: {:?}", analysis.detected_modalities);
            println!("   Quality score: {:.2}", analysis.overall_quality);
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Test 3: Analyze JSON data
    println!("\n📊 Testing data analysis:");
    let json = r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}], "total": 2}"#;
    match agent.analyze_multimodal_data(json).await {
        Ok(analysis) => {
            println!("   Detected modalities: {:?}", analysis.detected_modalities);
            println!("   Quality score: {:.2}", analysis.overall_quality);
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n🎉 Demo completed successfully!");
    Ok(())
}
