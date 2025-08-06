pub mod agent;
pub mod communication;
pub mod cpu_optimization;
pub mod hive;
pub mod mcp;
pub mod neural;
pub mod nlp;
pub mod optimized_agent;
pub mod resource_manager;
pub mod task;

pub use agent::*;
pub use hive::*;
pub use neural::*;
pub use nlp::*;
pub use task::*;

// Re-export AppState for communication module
#[derive(Clone)]
pub struct AppState {
    pub hive: std::sync::Arc<tokio::sync::RwLock<HiveCoordinator>>,
}