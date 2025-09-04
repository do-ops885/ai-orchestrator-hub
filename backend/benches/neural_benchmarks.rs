use criterion::{black_box, criterion_group, criterion_main, Criterion};
use multiagent_hive::neural::core::NeuralNetwork;
use multiagent_hive::neural::data::TrainingData;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark neural network creation
fn neural_network_creation_benchmark(c: &mut Criterion) {
    c.bench_function("neural_network_creation", |b| {
        b.iter(|| {
            let config = multiagent_hive::neural::core::NeuralConfig {
                input_size: 10,
                hidden_layers: vec![20, 15],
                output_size: 5,
                learning_rate: 0.01,
                activation: multiagent_hive::neural::core::ActivationFunction::ReLU,
                loss_function: multiagent_hive::neural::core::LossFunction::MSE,
            };

            let network = NeuralNetwork::new(config);
            black_box(network);
        });
    });
}

/// Benchmark neural network forward pass
fn neural_forward_pass_benchmark(c: &mut Criterion) {
    let config = multiagent_hive::neural::core::NeuralConfig {
        input_size: 10,
        hidden_layers: vec![20, 15],
        output_size: 5,
        learning_rate: 0.01,
        activation: multiagent_hive::neural::core::ActivationFunction::ReLU,
        loss_function: multiagent_hive::neural::core::LossFunction::MSE,
    };

    let network = NeuralNetwork::new(config);

    c.bench_function("neural_forward_pass", |b| {
        b.iter(|| {
            let input = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
            let output = network.forward(&input);
            black_box(output);
        });
    });
}

/// Benchmark neural network training
fn neural_training_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("neural_training", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = multiagent_hive::neural::core::NeuralConfig {
                    input_size: 10,
                    hidden_layers: vec![20, 15],
                    output_size: 5,
                    learning_rate: 0.01,
                    activation: multiagent_hive::neural::core::ActivationFunction::ReLU,
                    loss_function: multiagent_hive::neural::core::LossFunction::MSE,
                };

                let mut network = NeuralNetwork::new(config);

                // Create training data
                let training_data = vec![
                    TrainingData {
                        input: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
                        target: vec![0.0, 1.0, 0.0, 0.0, 0.0],
                    },
                    TrainingData {
                        input: vec![1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1],
                        target: vec![0.0, 0.0, 1.0, 0.0, 0.0],
                    },
                ];

                // Train for a few epochs
                for _ in 0..5 {
                    for data in &training_data {
                        network.train(&data.input, &data.target);
                    }
                }

                black_box(network);
            });
        });
    });
}

/// Benchmark NLP processing
fn nlp_processing_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("nlp_processing", |b| {
        b.iter(|| {
            rt.block_on(async {
                let text = "This is a benchmark test for natural language processing performance in the AI Orchestrator Hub system.";
                let result = multiagent_hive::neural::nlp::process_text(text).await;
                black_box(result);
            });
        });
    });
}

/// Benchmark concurrent neural operations
fn concurrent_neural_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_neural_ops", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = vec![];

                for i in 0..5 {
                    let handle = tokio::spawn(async move {
                        let config = multiagent_hive::neural::core::NeuralConfig {
                            input_size: 10,
                            hidden_layers: vec![20],
                            output_size: 5,
                            learning_rate: 0.01,
                            activation: multiagent_hive::neural::core::ActivationFunction::ReLU,
                            loss_function: multiagent_hive::neural::core::LossFunction::MSE,
                        };

                        let mut network = NeuralNetwork::new(config);

                        let input = vec![
                            0.1 * (i as f64),
                            0.2,
                            0.3,
                            0.4,
                            0.5,
                            0.6,
                            0.7,
                            0.8,
                            0.9,
                            1.0,
                        ];
                        let output = network.forward(&input);

                        // Simulate some training
                        let target = vec![0.0, 1.0, 0.0, 0.0, 0.0];
                        network.train(&input, &target);

                        output
                    });

                    handles.push(handle);
                }

                for handle in handles {
                    let _result = handle.await.unwrap();
                }
            });
        });
    });
}

/// Benchmark neural network serialization
fn neural_serialization_benchmark(c: &mut Criterion) {
    let config = multiagent_hive::neural::core::NeuralConfig {
        input_size: 10,
        hidden_layers: vec![20, 15],
        output_size: 5,
        learning_rate: 0.01,
        activation: multiagent_hive::neural::core::ActivationFunction::ReLU,
        loss_function: multiagent_hive::neural::core::LossFunction::MSE,
    };

    let network = NeuralNetwork::new(config);

    c.bench_function("neural_serialization", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(&network).unwrap();
            let deserialized: NeuralNetwork = serde_json::from_str(&serialized).unwrap();
            black_box(deserialized);
        });
    });
}

criterion_group!(
    benches,
    neural_network_creation_benchmark,
    neural_forward_pass_benchmark,
    neural_training_benchmark,
    nlp_processing_benchmark,
    concurrent_neural_operations_benchmark,
    neural_serialization_benchmark
);
criterion_main!(benches);
