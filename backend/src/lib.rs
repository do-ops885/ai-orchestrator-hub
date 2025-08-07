pub mod core;
pub mod agents;
pub mod tasks;
pub mod neural;
pub mod communication;
pub mod infrastructure;
pub mod utils;

pub use core::*;
pub use agents::*;
pub use tasks::*;
pub use neural::*;
pub use communication::*;
pub use infrastructure::*;
pub use utils::*;

// Re-export AppState for communication module
#[derive(Clone)]
pub struct AppState {
    pub hive: std::sync::Arc<tokio::sync::RwLock<HiveCoordinator>>,
}