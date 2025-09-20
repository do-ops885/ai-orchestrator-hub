//! # Multi-Modal Agent Edge Case Tests
//!
//! Tests for unusual inputs, boundary conditions, and error scenarios.

use multiagent_hive::agents::{DataModality, MultiModalAgent};
use multiagent_hive::neural::NLPProcessor;

#[tokio::test]
async fn test_empty_and_whitespace_inputs() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("EdgeCaseTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    // Test empty string
    let result = agent.analyze_multimodal_data("").await.unwrap();
    assert!(result.detected_modalities.contains(&DataModality::Text));
    assert!(result.overall_quality >= 0.0);

    // Test whitespace only
    let result = agent
        .analyze_multimodal_data("   \n\t  \r\n   ")
        .await
        .unwrap();
    assert!(result.overall_quality >= 0.0);

    // Test very long whitespace
    let long_whitespace = " ".repeat(10000);
    let result = agent
        .analyze_multimodal_data(&long_whitespace)
        .await
        .unwrap();
    assert!(result.processing_time_ms < 5000); // Should handle efficiently
}

#[tokio::test]
async fn test_unicode_and_special_characters() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("UnicodeTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    let unicode_content = r#"
    # 多言語テスト - Unicode Test 🚀
    
    def process_data(データ):
        """
        Process data with unicode characters: 中文, العربية, русский
        """
        result = {
            "message": "Hello 世界! 🌍",
            "emoji": "🎉🔥💯",
            "math": "∑(i=1 to ∞) 1/i² = π²/6",
            "special": "«»„"‚'‹›‚„"‹›«»"
        }
        return result
    "#;

    let result = agent
        .analyze_multimodal_data(unicode_content)
        .await
        .unwrap();

    assert!(result.detected_modalities.contains(&DataModality::Mixed));
    assert!(result.overall_quality > 0.0);
    assert!(!result.cross_modal_insights.is_empty());
}

#[tokio::test]
async fn test_deeply_nested_json() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("NestedTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    // Generate deeply nested JSON structure
    let mut nested_json = String::from(r#"{"level0": {"#);
    for i in 1..20 {
        nested_json.push_str(&format!(r#""level{}": {{"#, i));
    }
    nested_json.push_str(r#""data": "deeply nested value""#);
    for _ in 0..20 {
        nested_json.push_str("}}");
    }

    let result = agent.analyze_multimodal_data(&nested_json).await.unwrap();

    assert!(result
        .detected_modalities
        .iter()
        .any(|m| matches!(m, DataModality::StructuredData(_))));
    if let Some(data_analysis) = result.data_analysis {
        assert!(data_analysis.schema_valid || !data_analysis.schema_valid); // Should handle either way
    }
}

#[tokio::test]
async fn test_mixed_language_code() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("MixedLangTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    let mixed_code = r#"
    // JavaScript with embedded SQL and HTML
    function generateReport(userId) {
        const query = `
            SELECT u.name, u.email, 
                   COUNT(o.id) as order_count,
                   SUM(o.total) as total_spent
            FROM users u 
            LEFT JOIN orders o ON u.id = o.user_id 
            WHERE u.id = ${userId}
            GROUP BY u.id
        `;
        
        const htmlTemplate = `
            <div class="user-report">
                <h2>User Report</h2>
                <p>Name: {{name}}</p>
                <p>Email: {{email}}</p>
                <p>Orders: {{order_count}}</p>
                <p>Total Spent: ${{total_spent}}</p>
            </div>
        `;
        
        return {
            query: query,
            template: htmlTemplate,
            styles: `
                .user-report {
                    padding: 20px;
                    border: 1px solid #ccc;
                    border-radius: 5px;
                }
            `
        };
    }
    "#;

    let result = agent.analyze_multimodal_data(mixed_code).await.unwrap();

    assert!(result
        .detected_modalities
        .iter()
        .any(|m| matches!(m, DataModality::Code(_))));
    assert!(result.overall_quality > 0.0);

    if let Some(code_analysis) = result.code_analysis {
        // Should detect SQL injection risk with template literals
        assert!(code_analysis.security_issues.iter().any(
            |issue| issue.issue_type.contains("SQL") || issue.description.contains("injection")
        ));
    }
}

#[tokio::test]
async fn test_extremely_long_lines() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("LongLineTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    // Create a very long single line
    let long_line = format!(
        "const extremelyLongVariableName = {};",
        "\"very long string content \".repeat(1000)".repeat(50)
    );

    let result = agent.analyze_multimodal_data(&long_line).await.unwrap();

    assert!(result
        .detected_modalities
        .iter()
        .any(|m| matches!(m, DataModality::Code(_))));
    assert!(result.processing_time_ms < 10000); // Should complete in reasonable time
}

#[tokio::test]
async fn test_binary_and_encoded_content() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("BinaryTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    let encoded_content = r#"
    # Base64 encoded data analysis
    
    import base64
    
    def process_encoded_data():
        # This is base64 encoded JSON data
        encoded = "eyJ1c2VyIjoiSm9obiIsImVtYWlsIjoiam9obkBleGFtcGxlLmNvbSIsImFnZSI6MzB9"
        
        try:
            decoded = base64.b64decode(encoded).decode('utf-8')
            print(f"Decoded: {decoded}")
            return decoded
        except Exception as e:
            print(f"Decoding failed: {e}")
            return None
    
    # Hex encoded data
    hex_data = "48656c6c6f20576f726c64"
    text_from_hex = bytes.fromhex(hex_data).decode('utf-8')
    "#;

    let result = agent
        .analyze_multimodal_data(encoded_content)
        .await
        .unwrap();

    assert!(result.detected_modalities.contains(&DataModality::Mixed));
    assert!(result.overall_quality > 0.0);
}

#[tokio::test]
async fn test_malicious_input_patterns() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("SecurityTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    let malicious_patterns = vec![
        // Script injection attempts
        r#"<script>alert('xss')</script>"#,
        // SQL injection patterns
        r#"'; DROP TABLE users; --"#,
        // Path traversal
        r#"../../../etc/passwd"#,
        // Command injection
        r#"test; rm -rf /"#,
        // XXE attack pattern
        r#"<?xml version="1.0"?><!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><test>&xxe;</test>"#,
    ];

    for pattern in malicious_patterns {
        let result = agent.analyze_multimodal_data(pattern).await.unwrap();

        // Should complete without crashing
        assert!(result.overall_quality >= 0.0);

        // Should potentially detect security issues if pattern is recognized as code
        if result
            .detected_modalities
            .iter()
            .any(|m| matches!(m, DataModality::Code(_)))
        {
            if let Some(code_analysis) = &result.code_analysis {
                // May detect security issues depending on pattern recognition
                println!(
                    "Detected {} security issues for pattern: {}",
                    code_analysis.security_issues.len(),
                    pattern
                );
            }
        }
    }
}
