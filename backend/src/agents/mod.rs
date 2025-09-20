/// Adaptive verification system with machine learning threshold optimization
pub mod adaptive_verification;
/// Agent implementations and behaviors
pub mod agent;
/// Agent memory and learning systems
pub mod memory;
/// Performance-optimized agent implementations
pub mod optimized_agent;
/// Agent recovery and error handling
pub mod recovery;
/// Self-healing swarm agent for system resilience
pub mod self_healing_swarm;
/// Simple verification system for lightweight task validation
pub mod simple_verification;
/// Agent skill evolution and learning system
pub mod skill_evolution;

/// Pair programming verification system
pub mod verification;

#[cfg(test)]
mod communication_test;
pub use agent::*;
pub use recovery::*;
pub use self_healing_swarm::*;
pub use simple_verification::*;
pub use skill_evolution::*;
pub use verification::*;
