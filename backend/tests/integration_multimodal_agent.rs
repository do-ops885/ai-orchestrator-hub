//! # Multi-Modal Agent Integration Tests
//!
//! Comprehensive integration tests that validate the Multi-Modal Agent's
//! functionality in real-world scenarios and edge cases.

use multiagent_hive::agents::{AgentBehavior, DataModality, MultiModalAgent};
use multiagent_hive::neural::NLPProcessor;
use multiagent_hive::tasks::{Task, TaskPriority, TaskRequiredCapability};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_large_scale_text_processing() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("LargeScaleTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    // Generate large text content (10KB+)
    let large_text = (0..1000)
        .map(|i| format!("This is sentence number {} in a very large document that tests the agent's ability to handle substantial text content efficiently. The system should maintain performance and accuracy even with large inputs.", i))
        .collect::<Vec<_>>()
        .join(" ");

    let result = timeout(
        Duration::from_secs(30),
        agent.analyze_multimodal_data(&large_text),
    )
    .await
    .expect("Analysis should complete within 30 seconds")
    .expect("Analysis should succeed");

    assert!(result.detected_modalities.contains(&DataModality::Text));
    assert!(result.overall_quality > 0.0);
    assert!(result.processing_time_ms < 30000); // Should be much faster than timeout
}

#[tokio::test]
async fn test_complex_code_security_analysis() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("SecurityTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    let vulnerable_code = r#"
    import hashlib
    import sqlite3
    
    # SECURITY ISSUES INTENTIONALLY INCLUDED FOR TESTING
    
    class UserManager:
        def __init__(self):
            # Hardcoded database credentials
            self.db_password = "admin123"
            self.secret_key = "secretkey123"
            
        def authenticate_user(self, username, password):
            # SQL injection vulnerability
            query = f"SELECT * FROM users WHERE username = '{username}' AND password = '{password}'"
            
            conn = sqlite3.connect('users.db')
            cursor = conn.cursor()
            cursor.execute(query)
            result = cursor.fetchone()
            
            if result:
                # Weak password hashing
                hashed = hashlib.md5(password.encode()).hexdigest()
                return {"user_id": result[0], "hash": hashed}
            return None
            
        def create_user(self, username, password):
            # More SQL injection
            query = f"INSERT INTO users (username, password) VALUES ('{username}', '{password}')"
            
            conn = sqlite3.connect('users.db')
            cursor = conn.cursor()
            cursor.execute(query)
            conn.commit()
            
        def admin_access(self, token):
            # Hardcoded admin token
            if token == "admin_token_123":
                return True
            return False
    "#;

    let result = agent
        .analyze_multimodal_data(vulnerable_code)
        .await
        .unwrap();

    assert!(result
        .detected_modalities
        .iter()
        .any(|m| matches!(m, DataModality::Code(_))));
    assert!(result.code_analysis.is_some());

    let code_analysis = result.code_analysis.unwrap();
    assert!(
        !code_analysis.security_issues.is_empty(),
        "Should detect security vulnerabilities"
    );
    assert!(
        code_analysis.security_issues.len() >= 3,
        "Should detect multiple security issues"
    );

    // Check for specific vulnerability types
    let issue_types: Vec<String> = code_analysis
        .security_issues
        .iter()
        .map(|issue| issue.issue_type.clone())
        .collect();

    assert!(issue_types.iter().any(|t| t.contains("SQL Injection")));
    assert!(issue_types.iter().any(|t| t.contains("Hardcoded")));
}

#[tokio::test]
async fn test_malformed_data_handling() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("MalformedTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    let malformed_json = r#"{
        "users": [
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob", "email": "bob@example.com",
            {"id": 3, "name": "Charlie" // Missing closing brace and comma
        ],
        "metadata": {
            "version": "1.0"
            "created": "2024-01-01" // Missing comma
        }
    }"#;

    let result = agent.analyze_multimodal_data(malformed_json).await.unwrap();

    // Should still detect as structured data attempt
    assert!(result
        .detected_modalities
        .iter()
        .any(|m| matches!(m, DataModality::StructuredData(_))));

    if let Some(data_analysis) = result.data_analysis {
        assert!(!data_analysis.schema_valid, "Should detect malformed JSON");
        assert!(
            data_analysis.quality_score < 0.5,
            "Quality should be low for malformed data"
        );
    }
}
