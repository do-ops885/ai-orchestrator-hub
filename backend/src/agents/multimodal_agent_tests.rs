//! # Multi-Modal Agent Tests
//!
//! Comprehensive tests for the Multi-Modal Intelligence Agent to verify
//! its capabilities across text, code, and data processing modalities.

#[cfg(test)]
mod tests {
    use super::super::multimodal_agent::*;
    use crate::neural::NLPProcessor;
    use crate::tasks::{Task, TaskPriority, TaskRequiredCapability};
    use crate::agents::AgentBehavior;

    async fn create_test_multimodal_agent() -> MultiModalAgent {
        let nlp_processor = NLPProcessor::new().await.expect("Failed to create NLP processor");
        MultiModalAgent::new("TestMultiModalAgent".to_string(), Some(nlp_processor))
            .await
            .expect("Failed to create MultiModalAgent")
    }

    #[tokio::test]
    async fn test_text_modality_detection() {
        let mut agent = create_test_multimodal_agent().await;
        
        let text_input = "This is a simple text message with some natural language content. It contains multiple sentences and should be detected as text.";
        
        let analysis = agent.analyze_multimodal_data(text_input).await.unwrap();
        
        assert!(analysis.detected_modalities.contains(&DataModality::Text));
        assert!(analysis.text_analysis.is_some());
        assert!(analysis.overall_quality > 0.0);
    }

    #[tokio::test]
    async fn test_code_modality_detection() {
        let mut agent = create_test_multimodal_agent().await;
        
        let code_input = r#"
        function calculateSum(a, b) {
            if (a > 0 && b > 0) {
                return a + b;
            }
            return 0;
        }
        
        // This is a test function
        const result = calculateSum(5, 10);
        "#;
        
        let analysis = agent.analyze_multimodal_data(code_input).await.unwrap();
        
        assert!(analysis.detected_modalities.iter().any(|m| matches!(m, DataModality::Code(_))));
        assert!(analysis.code_analysis.is_some());
        
        let code_analysis = analysis.code_analysis.unwrap();
        assert_eq!(code_analysis.language, "javascript");
        assert!(code_analysis.complexity_score > 0.0);
        assert!(code_analysis.quality_metrics.loc > 0);
    }

    #[tokio::test]
    async fn test_json_data_modality_detection() {
        let mut agent = create_test_multimodal_agent().await;
        
        let json_input = r#"
        {
            "users": [
                {"id": 1, "name": "Alice", "email": "alice@example.com"},
                {"id": 2, "name": "Bob", "email": "bob@example.com"},
                {"id": 3, "name": "Charlie", "email": "charlie@example.com"}
            ],
            "total": 3,
            "metadata": {
                "version": "1.0",
                "created": "2024-01-01"
            }
        }
        "#;
        
        let analysis = agent.analyze_multimodal_data(json_input).await.unwrap();
        
        assert!(analysis.detected_modalities.iter().any(|m| matches!(m, DataModality::StructuredData(_))));
        assert!(analysis.data_analysis.is_some());
        
        let data_analysis = analysis.data_analysis.unwrap();
        assert_eq!(data_analysis.format, "json");
        assert!(data_analysis.schema_valid);
        assert!(data_analysis.statistics.record_count > 0);
    }

    #[tokio::test]
    async fn test_mixed_modality_detection() {
        let mut agent = create_test_multimodal_agent().await;
        
        let mixed_input = r#"
        Here's a simple Python function for data processing:
        
        def process_data(data):
            """Process user data and return statistics."""
            if not data:
                return {"error": "No data provided"}
            
            results = []
            for item in data:
                if item.get("active", False):
                    results.append(item)
            
            return {
                "total_items": len(data),
                "active_items": len(results),
                "success": True
            }
        
        This function takes JSON data like:
        {"users": [{"id": 1, "active": true}, {"id": 2, "active": false}]}
        "#;
        
        let analysis = agent.analyze_multimodal_data(mixed_input).await.unwrap();
        
        // Should detect multiple modalities
        assert!(analysis.detected_modalities.len() > 1);
        assert!(analysis.detected_modalities.contains(&DataModality::Mixed));
        assert!(analysis.detected_modalities.contains(&DataModality::Text));
        assert!(analysis.detected_modalities.iter().any(|m| matches!(m, DataModality::Code(_))));
        
        // Should have cross-modal insights
        assert!(!analysis.cross_modal_insights.is_empty());
    }

    #[tokio::test]
    async fn test_security_issue_detection() {
        let mut agent = create_test_multimodal_agent().await;
        
        let insecure_code = r#"
        const password = "hardcoded123";
        const query = "SELECT * FROM users WHERE name = '" + userName + "'";
        "#;
        
        let analysis = agent.analyze_multimodal_data(insecure_code).await.unwrap();
        
        assert!(analysis.code_analysis.is_some());
        let code_analysis = analysis.code_analysis.unwrap();
        assert!(!code_analysis.security_issues.is_empty());
        
        // Should detect hardcoded password
        assert!(code_analysis.security_issues.iter().any(|issue| 
            issue.issue_type == "Hardcoded Credentials"));
        
        // Should detect SQL injection risk
        assert!(code_analysis.security_issues.iter().any(|issue| 
            issue.issue_type == "SQL Injection"));
    }

    #[tokio::test]
    async fn test_rust_code_analysis() {
        let mut agent = create_test_multimodal_agent().await;
        
        let rust_code = r#"
        /// Calculate the factorial of a number
        pub fn factorial(n: u64) -> u64 {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        
        #[cfg(test)]
        mod tests {
            use super::*;
            
            #[test]
            fn test_factorial() {
                assert_eq!(factorial(5), 120);
            }
        }
        "#;
        
        let analysis = agent.analyze_multimodal_data(rust_code).await.unwrap();
        
        assert!(analysis.code_analysis.is_some());
        let code_analysis = analysis.code_analysis.unwrap();
        assert_eq!(code_analysis.language, "rust");
        assert!(code_analysis.quality_metrics.documentation_coverage > 0.0);
        
        // Should suggest improvements or indicate good quality
        assert!(!code_analysis.improvements.is_empty());
    }

    #[tokio::test]
    async fn test_task_execution_with_multimodal_capabilities() {
        let mut agent = create_test_multimodal_agent().await;
        
        let task = Task::new(
            "Analyze Code Quality".to_string(),
            "Analyze the following code for quality and security issues".to_string(),
            "code_analysis".to_string(),
            TaskPriority::High,
            vec![
                TaskRequiredCapability {
                    name: "code_analysis".to_string(),
                    minimum_proficiency: 0.6,
                },
                TaskRequiredCapability {
                    name: "pattern_recognition".to_string(),
                    minimum_proficiency: 0.5,
                },
            ],
        )
        .with_context("code".to_string(), r#"
            function processUser(user) {
                if (user.password === "admin123") {
                    return "Welcome admin!";
                }
                return "Access denied";
            }
        "#.to_string());
        
        let result = agent.execute_task(task).await.unwrap();
        
        assert!(result.success);
        assert!(result.output.contains("Multi-modal analysis completed"));
        assert!(result.quality_score.is_some());
        assert!(!result.learned_insights.is_empty());
    }

    #[tokio::test]
    async fn test_learning_from_analysis() {
        let mut agent = create_test_multimodal_agent().await;
        
        // Get initial proficiency
        let initial_code_proficiency = agent.base.get_capability_score("code_analysis");
        
        // Perform several successful analyses
        for _ in 0..3 {
            let high_quality_code = r#"
                /// Well-documented function with proper error handling
                pub fn divide(a: f64, b: f64) -> Result<f64, &'static str> {
                    if b == 0.0 {
                        Err("Division by zero")
                    } else {
                        Ok(a / b)
                    }
                }
            "#;
            
            let _analysis = agent.analyze_multimodal_data(high_quality_code).await.unwrap();
        }
        
        // Check if proficiency improved
        let final_code_proficiency = agent.base.get_capability_score("code_analysis");
        
        // Should have learned from successful analyses
        assert!(agent.analysis_history.len() == 3);
        assert!(agent.performance_metrics.total_analyses == 3);
        
        // Proficiency should remain stable or improve slightly
        assert!(final_code_proficiency >= initial_code_proficiency - 0.1);
    }

    #[tokio::test]
    async fn test_performance_metrics_tracking() {
        let mut agent = create_test_multimodal_agent().await;
        
        // Perform analyses of different modalities
        let _text_analysis = agent.analyze_multimodal_data("This is a text message.").await.unwrap();
        let _code_analysis = agent.analyze_multimodal_data("function test() { return true; }").await.unwrap();
        let _data_analysis = agent.analyze_multimodal_data(r#"{"test": "data"}"#).await.unwrap();
        
        let metrics = agent.get_performance_metrics();
        
        assert_eq!(metrics.total_analyses, 3);
        assert!(metrics.analyses_by_modality.len() > 0);
        assert!(metrics.learning_curve.len() == 3);
        
        // Should track different modalities
        assert!(metrics.analyses_by_modality.values().sum::<u64>() >= 3);
    }

    #[tokio::test]
    async fn test_modality_expertise_evolution() {
        let mut agent = create_test_multimodal_agent().await;
        
        // Get initial expertise levels
        let initial_text_expertise = agent.modality_expertise.get(&DataModality::Text).copied().unwrap_or(0.0);
        
        // Perform high-quality text analysis
        let high_quality_text = "This comprehensive analysis demonstrates excellent natural language processing capabilities with sophisticated vocabulary and complex sentence structures.";
        let _analysis = agent.analyze_multimodal_data(high_quality_text).await.unwrap();
        
        // Check if expertise evolved
        let final_text_expertise = agent.modality_expertise.get(&DataModality::Text).copied().unwrap_or(0.0);
        
        // Expertise should be maintained or improved
        assert!(final_text_expertise >= initial_text_expertise - 0.1);
    }

    #[tokio::test]
    async fn test_cross_modal_insights_generation() {
        let mut agent = create_test_multimodal_agent().await;
        
        let complex_input = r#"
        This code has quality issues that need attention:
        
        def bad_function():
            password = "secret123"  # Hardcoded password - security risk!
            # No documentation or error handling
            return password
        
        The data shows concerning patterns:
        {"security_score": 0.2, "quality_issues": 5}
        "#;
        
        let analysis = agent.analyze_multimodal_data(complex_input).await.unwrap();
        
        // Should generate meaningful cross-modal insights
        assert!(!analysis.cross_modal_insights.is_empty());
        assert!(analysis.detected_modalities.len() >= 2);
        
        // Quality score should reflect the issues found
        assert!(analysis.overall_quality < 0.7);
    }
}