//! # Multi-Modal Agent Demonstration
//!
//! This example showcases the capabilities of the Multi-Modal Intelligence Agent
//! by processing various types of content and demonstrating cross-modal analysis.

use multiagent_hive::agents::{AgentBehavior, DataModality, MultiModalAgent};
use multiagent_hive::neural::NLPProcessor;
use multiagent_hive::tasks::{Task, TaskPriority, TaskRequiredCapability};
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Multi-Modal Intelligence Agent Demo");
    println!("=====================================\n");

    // Create NLP processor and multi-modal agent
    let nlp_processor = NLPProcessor::new().await?;
    let mut agent =
        MultiModalAgent::new("DemoMultiModalAgent".to_string(), Some(nlp_processor)).await?;

    println!("✅ Created Multi-Modal Agent: {}", agent.base.name);
    println!("📊 Initial capabilities:");
    for capability in &agent.base.capabilities {
        println!("   - {}: {:.2}", capability.name, capability.proficiency);
    }
    println!();

    // Demo 1: Text Analysis
    println!("🔤 Demo 1: Text Analysis");
    println!("-----------------------");
    let text_content = "The quarterly report shows excellent performance with 95% customer satisfaction. Our team delivered innovative solutions that exceeded expectations and improved operational efficiency by 30%.";

    let text_analysis = agent.analyze_multimodal_data(text_content).await?;
    println!("📝 Analyzed text content:");
    println!("   Primary modality: {:?}", text_analysis.primary_modality);
    println!("   Overall quality: {:.2}", text_analysis.overall_quality);
    if let Some(text_result) = &text_analysis.text_analysis {
        println!("   Sentiment: {:.2}", text_result.sentiment);
        println!("   Keywords: {:?}", text_result.keywords);
    }
    println!(
        "   Processing time: {}ms\n",
        text_analysis.processing_time_ms
    );

    // Demo 2: Code Analysis
    println!("💻 Demo 2: Code Analysis");
    println!("------------------------");
    let code_content = r#"
    /// Calculate user account balance with security checks
    pub async fn calculate_balance(user_id: u64, account_type: &str) -> Result<f64, String> {
        // Input validation
        if user_id == 0 {
            return Err("Invalid user ID".to_string());
        }
        
        // Fetch account data securely
        let account = fetch_account_data(user_id, account_type).await
            .map_err(|e| format!("Database error: {}", e))?;
        
        // Apply business logic
        let balance = match account_type {
            "savings" => account.balance * 1.02, // 2% interest
            "checking" => account.balance,
            _ => return Err("Unknown account type".to_string()),
        };
        
        Ok(balance)
    }
    "#;

    let code_analysis = agent.analyze_multimodal_data(code_content).await?;
    println!("🔍 Analyzed Rust code:");
    println!("   Primary modality: {:?}", code_analysis.primary_modality);
    println!("   Overall quality: {:.2}", code_analysis.overall_quality);
    if let Some(code_result) = &code_analysis.code_analysis {
        println!("   Language: {}", code_result.language);
        println!("   Complexity score: {:.2}", code_result.complexity_score);
        println!("   Lines of code: {}", code_result.quality_metrics.loc);
        println!(
            "   Documentation coverage: {:.2}",
            code_result.quality_metrics.documentation_coverage
        );
        println!("   Security issues: {}", code_result.security_issues.len());
        println!("   Improvements: {:?}", code_result.improvements);
    }
    println!(
        "   Processing time: {}ms\n",
        code_analysis.processing_time_ms
    );

    // Demo 3: JSON Data Analysis
    println!("📊 Demo 3: JSON Data Analysis");
    println!("-----------------------------");
    let json_content = r#"
    {
        "api_metrics": {
            "endpoints": [
                {"path": "/users", "requests": 1250, "avg_response_time": 45, "errors": 2},
                {"path": "/orders", "requests": 890, "avg_response_time": 120, "errors": 0},
                {"path": "/products", "requests": 2100, "avg_response_time": 30, "errors": 5}
            ],
            "total_requests": 4240,
            "uptime": 99.8,
            "peak_concurrent_users": 150
        },
        "performance_summary": {
            "status": "excellent",
            "recommendations": ["optimize /orders endpoint", "monitor error rates"]
        }
    }
    "#;

    let data_analysis = agent.analyze_multimodal_data(json_content).await?;
    println!("📈 Analyzed JSON data:");
    println!("   Primary modality: {:?}", data_analysis.primary_modality);
    println!("   Overall quality: {:.2}", data_analysis.overall_quality);
    if let Some(data_result) = &data_analysis.data_analysis {
        println!("   Format: {}", data_result.format);
        println!("   Schema valid: {}", data_result.schema_valid);
        println!("   Data quality: {:.2}", data_result.quality_score);
        println!("   Record count: {}", data_result.statistics.record_count);
        println!("   Field count: {}", data_result.statistics.field_count);
    }
    println!(
        "   Processing time: {}ms\n",
        data_analysis.processing_time_ms
    );

    // Demo 4: Mixed Content Analysis
    println!("🌐 Demo 4: Mixed Content Analysis");
    println!("---------------------------------");
    let mixed_content = r#"
    Our API security audit revealed critical issues in the authentication system:
    
    function authenticateUser(username, password) {
        // SECURITY ISSUE: Hardcoded admin credentials
        if (username === "admin" && password === "admin123") {
            return { success: true, role: "admin" };
        }
        
        // ISSUE: SQL injection vulnerability
        const query = `SELECT * FROM users WHERE username = '${username}' AND password = '${password}'`;
        return database.query(query);
    }
    
    The vulnerability report shows:
    {
        "critical_issues": 2,
        "high_risk_endpoints": ["/login", "/admin"],
        "security_score": 0.3,
        "recommendation": "Immediate remediation required"
    }
    "#;

    let mixed_analysis = agent.analyze_multimodal_data(mixed_content).await?;
    println!("🔀 Analyzed mixed content:");
    println!(
        "   Detected modalities: {:?}",
        mixed_analysis.detected_modalities
    );
    println!("   Overall quality: {:.2}", mixed_analysis.overall_quality);
    println!("   Cross-modal insights:");
    for insight in &mixed_analysis.cross_modal_insights {
        println!("     • {}", insight);
    }
    println!(
        "   Processing time: {}ms\n",
        mixed_analysis.processing_time_ms
    );

    // Demo 5: Task Execution
    println!("⚡ Demo 5: Task Execution with Multi-Modal Analysis");
    println!("-------------------------------------------------");
    let analysis_task = Task::new(
        "Security Code Review".to_string(),
        "Perform comprehensive security analysis of authentication code".to_string(),
        "security_analysis".to_string(),
        TaskPriority::Critical,
        vec![
            TaskRequiredCapability {
                name: "code_analysis".to_string(),
                minimum_proficiency: 0.7,
            },
            TaskRequiredCapability {
                name: "pattern_recognition".to_string(),
                minimum_proficiency: 0.6,
            },
        ],
    )
    .with_context(
        "code".to_string(),
        r#"
        const jwt = require('jsonwebtoken');
        
        function generateToken(user) {
            // ISSUE: Hardcoded secret
            const secret = "mysecret123";
            return jwt.sign(user, secret);
        }
        
        function validateInput(data) {
            // ISSUE: No input validation
            return data;
        }
    "#
        .to_string(),
    );

    let task_result = agent.execute_task(analysis_task).await?;
    println!("📋 Task execution result:");
    println!("   Success: {}", task_result.success);
    println!(
        "   Quality score: {:.2}",
        task_result.quality_score.unwrap_or(0.0)
    );
    println!("   Execution time: {}ms", task_result.execution_time);
    println!("   Learned insights: {:?}", task_result.learned_insights);
    println!();

    // Display final agent metrics
    println!("📊 Final Agent Performance Metrics");
    println!("==================================");
    let metrics = agent.get_performance_metrics();
    println!("Total analyses performed: {}", metrics.total_analyses);
    println!("Analyses by modality:");
    for (modality, count) in &metrics.analyses_by_modality {
        println!("   - {}: {}", modality, count);
    }

    println!("\nModality expertise levels:");
    for (modality, expertise) in agent.get_modality_expertise() {
        println!("   - {:?}: {:.2}", modality, expertise);
    }

    println!("\nUpdated capabilities:");
    for capability in &agent.base.capabilities {
        println!("   - {}: {:.2}", capability.name, capability.proficiency);
    }

    println!("\n🎉 Multi-Modal Agent Demo completed successfully!");
    println!("The agent demonstrated advanced capabilities in:");
    println!("   ✅ Text sentiment analysis and keyword extraction");
    println!("   ✅ Code quality assessment and security scanning");
    println!("   ✅ Structured data validation and statistics");
    println!("   ✅ Cross-modal insight generation");
    println!("   ✅ Adaptive learning and performance tracking");

    Ok(())
}
