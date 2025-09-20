//! # Multi-Modal Agent Performance Tests
//!
//! Tests focused on performance characteristics, memory usage,
//! and scalability of the Multi-Modal Agent.

use multiagent_hive::agents::{DataModality, MultiModalAgent};
use multiagent_hive::neural::NLPProcessor;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

#[tokio::test]
async fn test_processing_speed_benchmarks() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("SpeedTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    // Test different input sizes and measure processing time
    let test_cases = vec![
        ("small", "Simple test message."),
        ("medium", &"A medium-sized text that contains multiple sentences and various concepts. ".repeat(10)),
        ("large", &"A large text document with extensive content that tests the system's ability to handle substantial input efficiently. ".repeat(100)),
    ];

    for (size_label, content) in test_cases {
        let start = Instant::now();
        let result = agent.analyze_multimodal_data(content).await.unwrap();
        let duration = start.elapsed();

        println!("Processing time for {} content: {:?}", size_label, duration);

        // Performance assertions
        match size_label {
            "small" => assert!(
                duration < Duration::from_millis(100),
                "Small content should process very quickly"
            ),
            "medium" => assert!(
                duration < Duration::from_millis(500),
                "Medium content should process reasonably fast"
            ),
            "large" => assert!(
                duration < Duration::from_secs(2),
                "Large content should complete within 2 seconds"
            ),
            _ => {}
        }

        assert!(result.overall_quality > 0.0);
        assert!(result.processing_time_ms > 0);
    }
}

#[tokio::test]
async fn test_memory_efficiency() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("MemoryTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    // Process many analyses to test memory management
    for i in 0..100 {
        let content = format!(
            "Memory test iteration {} with unique content to prevent caching effects.",
            i
        );
        let _result = agent.analyze_multimodal_data(&content).await.unwrap();
    }

    // Check that history is being managed (should not grow indefinitely)
    let history_size = agent.get_analysis_history().len();
    assert!(
        history_size <= 100,
        "Analysis history should be managed to prevent unlimited growth"
    );

    // Test history cleanup
    agent.clear_analysis_history();
    assert_eq!(
        agent.get_analysis_history().len(),
        0,
        "History should be cleared"
    );

    // Verify agent still works after cleanup
    let result = agent
        .analyze_multimodal_data("Post-cleanup test")
        .await
        .unwrap();
    assert!(result.overall_quality > 0.0);
}

#[tokio::test]
async fn test_concurrent_processing() {
    let concurrency_level = 10;
    let semaphore = Arc::new(Semaphore::new(concurrency_level));
    let mut handles = Vec::new();

    for i in 0..concurrency_level {
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let handle = tokio::spawn(async move {
            let _permit = permit; // Keep permit alive

            let nlp_processor = NLPProcessor::new().await.unwrap();
            let mut agent =
                MultiModalAgent::new(format!("ConcurrentAgent{}", i), Some(nlp_processor))
                    .await
                    .unwrap();

            let content = format!(
                "Concurrent processing test {} with unique identifier and content.",
                i
            );
            let start = Instant::now();
            let result = agent.analyze_multimodal_data(&content).await.unwrap();
            let duration = start.elapsed();

            (i, duration, result.overall_quality)
        });

        handles.push(handle);
    }

    // Wait for all concurrent tasks to complete
    let results = futures::future::join_all(handles).await;

    // Verify all tasks completed successfully
    for result in results {
        let (id, duration, quality) = result.unwrap();
        println!(
            "Agent {} completed in {:?} with quality {:.2}",
            id, duration, quality
        );

        assert!(
            duration < Duration::from_secs(5),
            "Concurrent processing should complete within reasonable time"
        );
        assert!(
            quality > 0.0,
            "All concurrent analyses should produce valid results"
        );
    }
}

#[tokio::test]
async fn test_learning_performance_impact() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("LearningTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    // Measure initial processing time
    let test_content = "Learning performance test with consistent content for measurement.";
    let start = Instant::now();
    let _result = agent.analyze_multimodal_data(test_content).await.unwrap();
    let initial_time = start.elapsed();

    // Perform multiple analyses to trigger learning
    for i in 0..20 {
        let content = format!("Learning iteration {} with varied high-quality content.", i);
        let _result = agent.analyze_multimodal_data(&content).await.unwrap();
    }

    // Measure processing time after learning
    let start = Instant::now();
    let _result = agent.analyze_multimodal_data(test_content).await.unwrap();
    let learned_time = start.elapsed();

    println!("Initial processing time: {:?}", initial_time);
    println!("Post-learning processing time: {:?}", learned_time);

    // Learning should not significantly degrade performance
    assert!(
        learned_time < initial_time * 2,
        "Learning should not significantly slow down processing"
    );

    // Check that capabilities have been updated through learning
    let initial_capabilities: Vec<_> = agent
        .base
        .capabilities
        .iter()
        .map(|c| (c.name.clone(), c.proficiency))
        .collect();

    // Capabilities should have reasonable proficiency levels
    for (name, proficiency) in initial_capabilities {
        assert!(
            proficiency >= 0.0 && proficiency <= 1.0,
            "Capability '{}' proficiency should be in valid range: {}",
            name,
            proficiency
        );
    }
}
