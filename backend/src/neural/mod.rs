/// Adaptive learning algorithms and continuous improvement
pub mod adaptive_learning;
/// CPU-specific optimizations for neural computations
pub mod cpu_optimization;
/// Data processing and feature engineering
pub mod data;
/// Neural network experiments and A/B testing
pub mod experiments;
/// Neural network monitoring and performance tracking
pub mod monitoring;
/// Core neural network implementations
pub mod neural;
/// Natural language processing utilities
pub mod nlp;
/// Neural network training and optimization
pub mod training;

// Explicit re-exports to avoid ambiguous glob re-exports
pub use adaptive_learning::{AdaptiveLearningConfig, AdaptiveLearningSystem};
pub use cpu_optimization::{CpuOptimizer, QuantizedOps, QuantizedWeights, VectorizedOps};
pub use data::{DataLoader, Dataset};
pub use experiments::{
    EarlyStoppingConfig, EarlyStoppingMode, Experiment, ExperimentComparison, ExperimentConfig,
    ExperimentMetadata, ExperimentRun, ExperimentStatus, ExperimentTracker,
};
pub use monitoring::{
    AlertSeverity, AlertType, ConfusionMatrix, EvaluationMetric, EvaluationResults,
    FeatureImportance, MetricsSnapshot, ROCCurve,
};
pub use neural::{HybridNeuralProcessor, NetworkType};
pub use nlp::{NLPProcessor, ProcessedText};
pub use training::{
    ArchitectureConfig, CNNConfig, DataConfig, GNNConfig, MemoryOptimization, ModelType,
    NeuralTrainingSystem, OptimizationConfig, RNNConfig, TrainingConfig, TrainingMetrics,
    TrainingParams, TrainingRecord, TrainingSession, TransformerConfig,
};

// Re-export advanced neural features if available
#[cfg(feature = "advanced-neural")]
pub use neural::{FANNConfig, LSTMConfig, NeuralConfig};
