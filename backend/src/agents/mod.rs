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
/// Agent evolution and genetic algorithms (temporarily disabled)
// pub mod agent_evolution;
/// Collaborative learning between agents (temporarily disabled)
// pub mod collaborative_learning;
/// Pair programming verification system
pub mod verification;
// Verification engine and coordination (temporarily disabled)
// pub mod verification_engine;
// Concrete verification strategies (temporarily disabled)
// pub mod verification_strategies;
#[cfg(test)]
mod communication_test;
pub use agent::*;
pub use recovery::*;
pub use self_healing_swarm::*;
pub use simple_verification::*;
pub use skill_evolution::*;
// pub use adaptive_verification::*;  // Currently unused
pub use verification::*;
// pub use verification_engine::*;
// pub use verification_strategies::*;
