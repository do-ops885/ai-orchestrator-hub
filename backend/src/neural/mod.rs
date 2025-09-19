/// Adaptive learning algorithms and continuous improvement
pub mod adaptive_learning;
/// Core neural network implementations
pub mod core;
/// CPU-specific optimizations for neural computations
pub mod cpu_optimization;
/// Data processing and feature engineering
pub mod data;
/// Neural network experiments and A/B testing
pub mod experiments;
/// Migration helper for replacing ruv-fann usage
pub mod migration_helper;
/// Neural network monitoring and performance tracking
pub mod monitoring;
/// Natural language processing utilities
pub mod nlp;
/// Integration layer for seamless ruv-fann replacement
pub mod optimized_integration;
/// Optimized neural network implementation (replaces ruv-fann)
pub mod optimized_network;
/// Neural network training and optimization
pub mod training;

// Explicit re-exports to avoid ambiguous glob re-exports
pub use adaptive_learning::{AdaptiveLearningConfig, AdaptiveLearningSystem};
pub use core::{HybridNeuralProcessor, NetworkType};
pub use cpu_optimization::{CpuOptimizer, QuantizedOps, QuantizedWeights, VectorizedOps};
pub use data::{DataLoader, Dataset};
pub use experiments::{
    EarlyStoppingConfig, EarlyStoppingMode, Experiment, ExperimentComparison, ExperimentConfig,
    ExperimentMetadata, ExperimentRun, ExperimentStatus, ExperimentTracker,
};
pub use migration_helper::{ActivationFunction, FANNConfig, Network};
pub use monitoring::{
    AlertSeverity, AlertType, ConfusionMatrix, EvaluationMetric, EvaluationResults,
    FeatureImportance, MetricsSnapshot, ROCCurve,
};
pub use nlp::{NLPProcessor, ProcessedText};
pub use optimized_integration::{FastNeuralConfig, FastNeuralNetwork, FastNeuralProcessor};
pub use optimized_network::{
    NetworkSpecialization, OptimizedNeuralManager, OptimizedNeuralNetwork,
};
pub use training::{
    ArchitectureConfig, CNNConfig, DataConfig, GNNConfig, MemoryOptimization, ModelType,
    NeuralTrainingSystem, OptimizationConfig, RNNConfig, TrainingConfig, TrainingMetrics,
    TrainingParams, TrainingRecord, TrainingSession, TransformerConfig,
};

// Re-export advanced neural features if available
#[cfg(feature = "advanced-neural")]
pub use core::{FANNConfig, LSTMConfig};
