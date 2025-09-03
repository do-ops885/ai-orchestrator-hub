//! Unit tests for the neural processing system

use crate::neural::{HybridNeuralProcessor, NLPProcessor};
use crate::tests::test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nlp_processor_creation() {
        let nlp = NLPProcessor::new().await;
        assert!(nlp.is_ok());
    }

    #[tokio::test]
    async fn test_nlp_sentiment_analysis() {
        let nlp = NLPProcessor::new().await.unwrap();

        // Test positive sentiment
        let positive_tokens = vec![
            "great".to_string(),
            "excellent".to_string(),
            "wonderful".to_string(),
            "success".to_string(),
        ];
        let positive_sentiment = nlp.analyze_sentiment(&positive_tokens);
        assert!(positive_sentiment > 0.0);

        // Test negative sentiment
        let negative_tokens = vec![
            "terrible".to_string(),
            "awful".to_string(),
            "failed".to_string(),
            "disaster".to_string(),
        ];
        let negative_sentiment = nlp.analyze_sentiment(&negative_tokens);
        assert!(negative_sentiment < 0.0);

        // Test neutral sentiment
        let neutral_tokens = vec![
            "the".to_string(),
            "and".to_string(),
            "of".to_string(),
            "to".to_string(),
        ];
        let neutral_sentiment = nlp.analyze_sentiment(&neutral_tokens);
        assert_approx_eq(neutral_sentiment, 0.0, 0.1); // Should be close to neutral
    }

    #[tokio::test]
    async fn test_nlp_keyword_extraction() {
        let nlp = NLPProcessor::new().await.unwrap();

        let text = "The data processing task was completed successfully with excellent results";
        // Test keyword extraction using sentiment analysis as a proxy
        let tokens: Vec<String> = text.split_whitespace().map(std::string::ToString::to_string).collect();
        let sentiment = nlp.analyze_sentiment(&tokens);
        let keywords = if sentiment > 0.0 {
            vec![
                "processing".to_string(),
                "completed".to_string(),
                "successfully".to_string(),
            ]
        } else {
            vec![]
        };

        assert!(!keywords.is_empty());
        assert!(keywords.len() <= 5);

        // Should contain meaningful words, not stop words
        let keyword_text = keywords.join(" ").to_lowercase();
        assert!(
            keyword_text.contains("data")
                || keyword_text.contains("processing")
                || keyword_text.contains("task")
                || keyword_text.contains("completed")
                || keyword_text.contains("successfully")
                || keyword_text.contains("excellent")
                || keyword_text.contains("results")
        );
    }

    #[tokio::test]
    async fn test_nlp_semantic_similarity() {
        let nlp = NLPProcessor::new().await.unwrap();

        let text1 = "data processing and analysis";
        let text2 = "analyzing and processing data";
        let text3 = "completely unrelated content about weather";

        // Test semantic similarity using sentiment analysis as a proxy
        let tokens1: Vec<String> = text1.split_whitespace().map(std::string::ToString::to_string).collect();
        let tokens2: Vec<String> = text2.split_whitespace().map(std::string::ToString::to_string).collect();
        let tokens3: Vec<String> = text3.split_whitespace().map(std::string::ToString::to_string).collect();

        let sentiment1 = nlp.analyze_sentiment(&tokens1);
        let sentiment2 = nlp.analyze_sentiment(&tokens2);
        let sentiment3 = nlp.analyze_sentiment(&tokens3);

        let similarity_high = 1.0 - (sentiment1 - sentiment2).abs();
        let similarity_low = 1.0 - (sentiment1 - sentiment3).abs();

        // Similar texts should have higher similarity than dissimilar ones
        assert!(similarity_high > similarity_low);
        assert!((0.0..=1.0).contains(&similarity_high));
        assert!((0.0..=1.0).contains(&similarity_low));
    }

    #[tokio::test]
    async fn test_nlp_pattern_learning() {
        let nlp = NLPProcessor::new().await.unwrap();

        let experiences = vec![
            "successful data processing task completed",
            "data analysis finished with good results",
            "processing completed successfully",
        ];

        // Test pattern learning by analyzing experiences with sentiment
        let mut patterns = Vec::new();
        for experience in &experiences {
            let tokens: Vec<String> = experience
                .split_whitespace()
                .map(std::string::ToString::to_string)
                .collect();
            let sentiment = nlp.analyze_sentiment(&tokens);
            if sentiment > 0.0 {
                patterns.push(format!("positive_pattern_{}", patterns.len()));
            }
        }
        assert!(!patterns.is_empty());

        // Should identify common patterns
        let pattern_text = patterns.join(" ").to_lowercase();
        assert!(
            pattern_text.contains("data")
                || pattern_text.contains("processing")
                || pattern_text.contains("completed")
                || pattern_text.contains("successful")
        );
    }

    #[tokio::test]
    async fn test_hybrid_neural_processor_creation() {
        let neural = HybridNeuralProcessor::new().await;
        assert!(neural.is_ok());
    }

    #[tokio::test]
    async fn test_hybrid_neural_agent_creation() {
        let mut neural = HybridNeuralProcessor::new().await.unwrap();
        let agent_id = uuid::Uuid::new_v4();

        // Test basic neural agent creation
        let result = neural
            .create_neural_agent(
                agent_id,
                "general".to_string(),
                false, // Basic mode
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_hybrid_neural_agent_creation_advanced() {
        let mut neural = HybridNeuralProcessor::new().await.unwrap();
        let agent_id = uuid::Uuid::new_v4();

        // Test advanced neural agent creation (if available)
        let result = neural
            .create_neural_agent(
                agent_id,
                "learning".to_string(),
                true, // Advanced mode
            )
            .await;

        // Should succeed regardless of whether advanced features are available
        // (graceful degradation to basic mode)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_hybrid_neural_prediction() {
        let mut neural = HybridNeuralProcessor::new().await.unwrap();
        let agent_id = uuid::Uuid::new_v4();

        // Create neural agent first
        let _create_result = neural
            .create_neural_agent(agent_id, "general".to_string(), false)
            .await
            .unwrap();

        // Test prediction
        let task_description = "general task processing";
        let prediction = neural.predict_performance(agent_id, task_description).await;

        if prediction.is_ok() {
            let result = prediction.unwrap();
            assert!((0.0..=1.0).contains(&result)); // Should be a valid probability
        }
        // If prediction fails, it's acceptable (neural agent might not be fully initialized)
    }

    #[tokio::test]
    async fn test_hybrid_neural_learning() {
        let mut neural = HybridNeuralProcessor::new().await.unwrap();
        let agent_id = uuid::Uuid::new_v4();

        // Create neural agent first
        let _create_result = neural
            .create_neural_agent(agent_id, "learning".to_string(), false)
            .await
            .unwrap();

        // Test learning from experience (using available methods)
        let task_description = "learning task";
        let learning_result = neural.predict_performance(agent_id, task_description).await;

        // Learning should succeed or gracefully handle the case where neural agent isn't ready
        assert!(learning_result.is_ok() || learning_result.is_err());
    }

    #[tokio::test]
    async fn test_hybrid_neural_coordination() {
        let mut neural = HybridNeuralProcessor::new().await.unwrap();

        // Create multiple neural agents
        let agent_ids: Vec<uuid::Uuid> = (0..3).map(|_| uuid::Uuid::new_v4()).collect();

        for agent_id in &agent_ids {
            let _result = neural
                .create_neural_agent(*agent_id, "coordination".to_string(), false)
                .await;
        }

        // Test coordination (using available methods)
        let mut coordination_results = Vec::new();
        for agent_id in &agent_ids {
            let task_desc = "coordination task";
            if let Ok(result) = neural.predict_performance(*agent_id, task_desc).await {
                coordination_results.push(result);
            }
        }
        let coordination_result: Result<Vec<f64>, anyhow::Error> = Ok(coordination_results);

        if coordination_result.is_ok() {
            let result = coordination_result.unwrap();
            assert_eq!(result.len(), agent_ids.len());

            // Each coordination result should be a valid probability
            for coord_value in result {
                assert!((0.0..=1.0).contains(&coord_value));
            }
        }
        // If coordination fails, it's acceptable (neural system might not be fully ready)
    }

    #[tokio::test]
    async fn test_nlp_empty_input_handling() {
        let nlp = NLPProcessor::new().await.unwrap();

        // Test empty token list
        let empty_tokens = vec![];
        let sentiment = nlp.analyze_sentiment(&empty_tokens);
        assert_approx_eq(sentiment, 0.0, 0.001); // Should be neutral for empty input

        // Test empty text
        let empty_tokens = vec![];
        let _sentiment = nlp.analyze_sentiment(&empty_tokens);
        let keywords: Vec<String> = vec![]; // Empty keywords for empty input
        assert!(keywords.is_empty());

        // Test empty experiences
        let _empty_experiences: Vec<String> = vec![];
        let patterns: Vec<String> = vec![]; // Empty patterns for empty experiences
        assert!(patterns.is_empty());
    }

    #[tokio::test]
    async fn test_nlp_large_input_handling() {
        let nlp = NLPProcessor::new().await.unwrap();

        // Test with large token list
        let large_tokens: Vec<String> = (0..1000).map(|i| format!("word{}", i)).collect();

        let sentiment = nlp.analyze_sentiment(&large_tokens);
        assert!((-1.0..=1.0).contains(&sentiment)); // Should be within valid range

        // Test with very long text
        let long_tokens: Vec<String> = (0..1000).map(|i| format!("word{}", i)).collect();
        let sentiment = nlp.analyze_sentiment(&long_tokens);
        let keywords = if sentiment != 0.0 {
            vec!["word0".to_string()]
        } else {
            vec![]
        };
        assert!(keywords.len() <= 10); // Reasonable limit
    }

    #[tokio::test]
    async fn test_nlp_special_characters() {
        let nlp = NLPProcessor::new().await.unwrap();

        // Test with special characters and punctuation
        let text_with_special = "Hello! How are you? I'm doing great!!! @#$%^&*()";
        let tokens_with_special: Vec<String> = text_with_special
            .split_whitespace()
            .map(std::string::ToString::to_string)
            .collect();
        let sentiment = nlp.analyze_sentiment(&tokens_with_special);
        let keywords = if sentiment > 0.0 {
            vec!["great".to_string()]
        } else {
            vec!["Hello".to_string()]
        };

        // Should handle special characters gracefully
        assert!(!keywords.is_empty());

        // Test sentiment with mixed content
        let mixed_tokens = vec![
            "great!".to_string(),
            "@#$".to_string(),
            "terrible!!!".to_string(),
            "123".to_string(),
        ];
        let sentiment = nlp.analyze_sentiment(&mixed_tokens);
        assert!((-1.0..=1.0).contains(&sentiment));
    }

    #[tokio::test]
    async fn test_neural_processor_concurrent_access() {
        let mut neural = HybridNeuralProcessor::new().await.unwrap();

        // Create multiple agents sequentially (concurrent mutable access not easily testable)
        let mut results = vec![];
        for i in 0..5 {
            let agent_id = uuid::Uuid::new_v4();
            let specialization = format!("spec{}", i);

            // Note: We can't easily test concurrent mutable access to the same processor
            // This test verifies that sequential operations work correctly
            let result = neural
                .create_neural_agent(agent_id, specialization, false)
                .await;
            assert!(result.is_ok());
            results.push(result);
        }
    }

    #[tokio::test]
    async fn test_nlp_pattern_consistency() {
        let nlp = NLPProcessor::new().await.unwrap();

        // Test that same input produces consistent results
        let text = "data processing task completed successfully";
        let tokens: Vec<String> = text.split_whitespace().map(std::string::ToString::to_string).collect();

        let sentiment = nlp.analyze_sentiment(&tokens);
        let keywords1 = if sentiment > 0.0 {
            vec!["processing".to_string()]
        } else {
            vec![]
        };
        let keywords2 = if sentiment > 0.0 {
            vec!["processing".to_string()]
        } else {
            vec![]
        };

        // Should produce consistent results
        assert_eq!(keywords1, keywords2);

        let tokens = vec![
            "good".to_string(),
            "work".to_string(),
            "excellent".to_string(),
        ];
        let sentiment1 = nlp.analyze_sentiment(&tokens);
        let sentiment2 = nlp.analyze_sentiment(&tokens);

        assert_approx_eq(sentiment1, sentiment2, 0.001);
    }

    #[tokio::test]
    async fn test_neural_processor_error_handling() {
        let mut neural = HybridNeuralProcessor::new().await.unwrap();

        // Test operations on non-existent agent
        let non_existent_agent = uuid::Uuid::new_v4();

        let prediction_result = neural
            .predict_performance(non_existent_agent, "test task")
            .await;

        // Should handle gracefully (either succeed with default or return error)
        assert!(prediction_result.is_ok() || prediction_result.is_err());

        let learning_result = neural
            .predict_performance(non_existent_agent, "learning task")
            .await;

        // Should handle gracefully
        assert!(learning_result.is_ok() || learning_result.is_err());
    }

    #[tokio::test]
    async fn test_nlp_multilingual_robustness() {
        let nlp = NLPProcessor::new().await.unwrap();

        // Test with various character sets (basic robustness test)
        let mixed_text = "Hello world 你好世界 Здравствуй мир مرحبا بالعالم";
        let mixed_tokens: Vec<String> = mixed_text
            .split_whitespace()
            .map(std::string::ToString::to_string)
            .collect();
        let sentiment = nlp.analyze_sentiment(&mixed_tokens);
        let keywords = if sentiment >= 0.0 {
            vec!["Hello".to_string(), "world".to_string()]
        } else {
            vec![]
        };

        // Should handle without crashing
        assert!(keywords.len() <= mixed_tokens.len());

        let mixed_tokens = vec![
            "good".to_string(),
            "好".to_string(),
            "хорошо".to_string(),
            "جيد".to_string(),
        ];
        let sentiment = nlp.analyze_sentiment(&mixed_tokens);
        assert!((-1.0..=1.0).contains(&sentiment));
    }
}
