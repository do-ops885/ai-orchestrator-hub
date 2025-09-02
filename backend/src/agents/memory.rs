//! # Agent Memory System
//!
//! This module implements a sophisticated memory architecture for agents, featuring:
//! - Short-term and long-term memory separation
//! - Pattern recognition and learning
//! - Social memory for inter-agent relationships
//! - Memory consolidation and forgetting mechanisms
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │  Working Memory │───▶│  Short-term      │───▶│  Long-term      │
//! │  (immediate)    │    │  Memory          │    │  Memory         │
//! │  - Current task │    │  - Recent events │    │  - Patterns     │
//! │  - Active goals │    │  - Temp patterns │    │  - Relationships│
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//!                                 │                        │
//!                                 ▼                        ▼
//!                        ┌─────────────────┐    ┌─────────────────┐
//!                        │  Consolidation  │    │  Pattern Store  │
//!                        │  - Importance   │    │  - Learned      │
//!                        │  - Frequency    │    │  - Validated    │
//!                        └─────────────────┘    └─────────────────┘
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Maximum number of items in short-term memory before consolidation
const SHORT_TERM_CAPACITY: usize = 50;

/// Maximum number of items in long-term memory
const LONG_TERM_CAPACITY: usize = 1000;

/// Threshold for memory importance to be consolidated to long-term
const CONSOLIDATION_THRESHOLD: f64 = 0.6;

/// Time after which short-term memories start to decay (in hours)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: Uuid,
    pub content: MemoryContent,
    pub timestamp: DateTime<Utc>,
    pub importance: f64,        // 0.0 to 1.0
    pub access_count: u32,      // How often this memory has been accessed
    pub emotional_valence: f64, // -1.0 (negative) to 1.0 (positive)
    pub context_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryContent {
    TaskExperience {
        task_id: Uuid,
        task_type: String,
        outcome: TaskOutcome,
        learned_strategy: Option<String>,
    },
    SocialInteraction {
        agent_id: Uuid,
        interaction_type: String,
        success: bool,
        trust_change: f64,
    },
    PatternObservation {
        pattern_type: String,
        pattern_data: serde_json::Value,
        confidence: f64,
    },
    EnvironmentalChange {
        change_type: String,
        impact_level: f64,
        adaptation_required: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutcome {
    Success {
        efficiency: f64,
        quality: f64,
    },
    Failure {
        reason: String,
        lessons: Vec<String>,
    },
    Partial {
        completion: f64,
        obstacles: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub pattern_id: Uuid,
    pub pattern_type: String,
    pub trigger_conditions: Vec<String>,
    pub expected_outcome: f64,
    pub confidence: f64,
    pub usage_count: u32,
    pub last_validated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMemory {
    pub agent_id: Uuid,
    pub trust_level: f64, // 0.0 to 1.0
    pub collaboration_history: Vec<CollaborationRecord>,
    pub communication_style: CommunicationStyle,
    pub reliability_score: f64, // Based on past interactions
    pub last_interaction: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationRecord {
    pub task_id: Uuid,
    pub role: String,
    pub success: bool,
    pub efficiency: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationStyle {
    Direct,
    Collaborative,
    Supportive,
    Analytical,
    Creative,
}

/// Advanced memory system for agents with multi-layered architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemorySystem {
    /// Immediate working memory for current tasks and active goals
    pub working_memory: HashMap<String, serde_json::Value>,

    /// Short-term memory for recent experiences (last 24-48 hours)
    pub short_term_memory: VecDeque<MemoryItem>,

    /// Long-term memory for important patterns and experiences
    pub long_term_memory: Vec<MemoryItem>,

    /// Learned patterns and strategies
    pub pattern_store: HashMap<String, LearnedPattern>,

    /// Social relationships and interaction history
    pub social_memory: HashMap<Uuid, SocialMemory>,

    /// Memory statistics for optimization
    pub memory_stats: MemoryStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    pub total_memories: usize,
    pub consolidation_events: u32,
    pub pattern_discoveries: u32,
    pub social_connections: usize,
    pub average_memory_importance: f64,
    pub last_consolidation: DateTime<Utc>,
}

impl Default for AgentMemorySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentMemorySystem {
    /// Create a new memory system with empty stores
    pub fn new() -> Self {
        Self {
            working_memory: HashMap::new(),
            short_term_memory: VecDeque::new(),
            long_term_memory: Vec::new(),
            pattern_store: HashMap::new(),
            social_memory: HashMap::new(),
            memory_stats: MemoryStatistics {
                total_memories: 0,
                consolidation_events: 0,
                pattern_discoveries: 0,
                social_connections: 0,
                average_memory_importance: 0.0,
                last_consolidation: Utc::now(),
            },
        }
    }

    /// Store a new memory item in short-term memory
    pub fn store_memory(
        &mut self,
        content: MemoryContent,
        importance: f64,
        context_tags: Vec<String>,
    ) -> Uuid {
        let memory_id = Uuid::new_v4();
        let emotional_valence = self.calculate_emotional_valence(&content);

        let memory_item = MemoryItem {
            id: memory_id,
            content,
            timestamp: Utc::now(),
            importance,
            access_count: 0,
            emotional_valence,
            context_tags,
        };

        self.short_term_memory.push_back(memory_item);
        self.memory_stats.total_memories += 1;

        // Trigger consolidation if short-term memory is full
        if self.short_term_memory.len() > SHORT_TERM_CAPACITY {
            self.consolidate_memories();
        }

        memory_id
    }

    /// Retrieve memories based on context and relevance
    pub fn recall_memories(&mut self, context: &str, limit: usize) -> Vec<MemoryItem> {
        let mut relevant_memories = Vec::new();

        // Search short-term memory
        for memory in &self.short_term_memory {
            if self.is_memory_relevant(memory, context) {
                relevant_memories.push(memory.clone());
            }
        }

        // Search long-term memory
        for memory in &self.long_term_memory {
            if self.is_memory_relevant(memory, context) {
                relevant_memories.push(memory.clone());
            }
        }

        // Sort by relevance (importance + recency + access count)
        relevant_memories.sort_by(|a, b| {
            let score_a = self.calculate_relevance_score(a, context);
            let score_b = self.calculate_relevance_score(b, context);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Collect memory IDs for access count updates
        let memory_ids: Vec<Uuid> = relevant_memories.iter().map(|m| m.id).collect();

        // Update access counts for retrieved memories
        for memory_id in memory_ids {
            self.increment_access_count(memory_id);
        }

        relevant_memories.into_iter().take(limit).collect()
    }

    /// Learn a new pattern from experiences
    pub fn learn_pattern(
        &mut self,
        pattern_type: String,
        trigger_conditions: Vec<String>,
        expected_outcome: f64,
    ) -> Uuid {
        let pattern_id = Uuid::new_v4();
        let pattern = LearnedPattern {
            pattern_id,
            pattern_type: pattern_type.clone(),
            trigger_conditions,
            expected_outcome,
            confidence: 0.5, // Start with moderate confidence
            usage_count: 0,
            last_validated: Utc::now(),
        };

        self.pattern_store.insert(pattern_type, pattern);
        self.memory_stats.pattern_discoveries += 1;

        pattern_id
    }

    /// Update social memory based on interaction
    pub fn update_social_memory(
        &mut self,
        agent_id: Uuid,
        interaction_success: bool,
        task_id: Option<Uuid>,
    ) {
        // Create new collaboration record if task-related
        let new_collaboration = task_id.map(|task_id| CollaborationRecord {
            task_id,
            role: "collaborator".to_string(), // Could be more specific
            success: interaction_success,
            efficiency: if interaction_success { 0.8 } else { 0.3 },
            timestamp: Utc::now(),
        });

        // Get current collaboration history and calculate new reliability score
        let mut updated_history = if let Some(entry) = self.social_memory.get(&agent_id) {
            entry.collaboration_history.clone()
        } else {
            Vec::new()
        };

        if let Some(collaboration) = &new_collaboration {
            updated_history.push(collaboration.clone());
        }

        let reliability_score = self.calculate_reliability_score(&updated_history);

        let social_entry = self.social_memory.entry(agent_id).or_insert_with(|| {
            self.memory_stats.social_connections += 1;
            SocialMemory {
                agent_id,
                trust_level: 0.5,
                collaboration_history: Vec::new(),
                communication_style: CommunicationStyle::Direct,
                reliability_score: 0.5,
                last_interaction: Utc::now(),
            }
        });

        // Update trust level
        let trust_adjustment = if interaction_success { 0.1 } else { -0.15 };
        social_entry.trust_level = (social_entry.trust_level + trust_adjustment).clamp(0.0, 1.0);

        // Add collaboration record if task-related
        if let Some(collaboration) = new_collaboration {
            social_entry.collaboration_history.push(collaboration);
        }

        // Update reliability score and timestamp
        social_entry.reliability_score = reliability_score;
        social_entry.last_interaction = Utc::now();

        // Store social interaction in memory
        let memory_content = MemoryContent::SocialInteraction {
            agent_id,
            interaction_type: "collaboration".to_string(),
            success: interaction_success,
            trust_change: trust_adjustment,
        };

        self.store_memory(
            memory_content,
            0.6,
            vec!["social".to_string(), "collaboration".to_string()],
        );
    }

    /// Consolidate short-term memories to long-term storage
    pub fn consolidate_memories(&mut self) {
        let mut consolidated_count = 0;

        // Process memories for consolidation
        while let Some(memory) = self.short_term_memory.pop_front() {
            if self.should_consolidate(&memory) {
                self.long_term_memory.push(memory);
                consolidated_count += 1;
            }
            // Memories that don't meet consolidation criteria are forgotten
        }

        // Limit long-term memory size
        if self.long_term_memory.len() > LONG_TERM_CAPACITY {
            // Remove least important memories
            self.long_term_memory.sort_by(|a, b| {
                b.importance
                    .partial_cmp(&a.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.long_term_memory.truncate(LONG_TERM_CAPACITY);
        }

        self.memory_stats.consolidation_events += 1;
        self.memory_stats.last_consolidation = Utc::now();
        self.update_memory_statistics();

        tracing::info!(
            "Consolidated {} memories to long-term storage",
            consolidated_count
        );
    }

    /// Get social trust level for another agent
    pub fn get_trust_level(&self, agent_id: Uuid) -> f64 {
        self.social_memory
            .get(&agent_id)
            .map_or(0.5, |social| social.trust_level) // Default neutral trust
    }

    /// Get learned patterns matching specific conditions
    pub fn get_matching_patterns(&self, conditions: &[String]) -> Vec<&LearnedPattern> {
        self.pattern_store
            .values()
            .filter(|pattern| {
                conditions
                    .iter()
                    .any(|condition| pattern.trigger_conditions.contains(condition))
            })
            .collect()
    }

    /// Private helper methods
    #[allow(clippy::unused_self)]
    fn calculate_emotional_valence(&self, content: &MemoryContent) -> f64 {
        match content {
            MemoryContent::TaskExperience { outcome, .. } => match outcome {
                TaskOutcome::Success { .. } => 0.8,
                TaskOutcome::Failure { .. } => -0.6,
                TaskOutcome::Partial { completion, .. } => completion * 0.4 - 0.2,
            },
            MemoryContent::SocialInteraction { success, .. } => {
                if *success {
                    0.6
                } else {
                    -0.4
                }
            }
            MemoryContent::PatternObservation { confidence, .. } => confidence * 0.5,
            MemoryContent::EnvironmentalChange { impact_level, .. } => -impact_level * 0.3,
        }
    }

    #[allow(clippy::unused_self)]
    fn is_memory_relevant(&self, memory: &MemoryItem, context: &str) -> bool {
        // Check context tags
        if memory.context_tags.iter().any(|tag| context.contains(tag)) {
            return true;
        }

        // Check content relevance
        match &memory.content {
            MemoryContent::TaskExperience { task_type, .. } => context.contains(task_type),
            MemoryContent::PatternObservation { pattern_type, .. } => {
                context.contains(pattern_type)
            }
            _ => false,
        }
    }

    fn calculate_relevance_score(&self, memory: &MemoryItem, context: &str) -> f64 {
        let mut score = memory.importance;

        // Boost score for recent memories
        let age_hours = (Utc::now() - memory.timestamp).num_hours() as f64;
        let recency_factor = (-age_hours / 168.0).exp(); // Decay over a week
        score += recency_factor * 0.3;

        // Boost score for frequently accessed memories
        let access_factor = f64::from(memory.access_count).ln() * 0.1;
        score += access_factor;

        // Context relevance boost
        if self.is_memory_relevant(memory, context) {
            score += 0.4;
        }

        score
    }

    #[allow(clippy::unused_self)]
    fn should_consolidate(&self, memory: &MemoryItem) -> bool {
        // High importance memories are always consolidated
        if memory.importance >= CONSOLIDATION_THRESHOLD {
            return true;
        }

        // Frequently accessed memories are consolidated
        if memory.access_count >= 3 {
            return true;
        }

        // Strong emotional memories are consolidated
        if memory.emotional_valence.abs() >= 0.7 {
            return true;
        }

        false
    }

    fn increment_access_count(&mut self, memory_id: Uuid) {
        // Update access count in short-term memory
        for memory in &mut self.short_term_memory {
            if memory.id == memory_id {
                memory.access_count += 1;
                return;
            }
        }

        // Update access count in long-term memory
        for memory in &mut self.long_term_memory {
            if memory.id == memory_id {
                memory.access_count += 1;
                return;
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn calculate_reliability_score(&self, history: &[CollaborationRecord]) -> f64 {
        if history.is_empty() {
            return 0.5;
        }

        let recent_history: Vec<_> = history
            .iter()
            .filter(|record| {
                (Utc::now() - record.timestamp).num_days() <= 30 // Last 30 days
            })
            .collect();

        if recent_history.is_empty() {
            return 0.5;
        }

        let success_rate = recent_history
            .iter()
            .map(|record| if record.success { 1.0 } else { 0.0 })
            .sum::<f64>()
            / recent_history.len() as f64;

        let avg_efficiency = recent_history
            .iter()
            .map(|record| record.efficiency)
            .sum::<f64>()
            / recent_history.len() as f64;

        (success_rate * 0.7 + avg_efficiency * 0.3).clamp(0.0, 1.0)
    }

    fn update_memory_statistics(&mut self) {
        self.memory_stats.total_memories =
            self.short_term_memory.len() + self.long_term_memory.len();
        self.memory_stats.social_connections = self.social_memory.len();

        let all_memories: Vec<_> = self
            .short_term_memory
            .iter()
            .chain(self.long_term_memory.iter())
            .collect();

        if !all_memories.is_empty() {
            self.memory_stats.average_memory_importance =
                all_memories.iter().map(|m| m.importance).sum::<f64>() / all_memories.len() as f64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_system_creation() {
        let memory_system = AgentMemorySystem::new();
        assert_eq!(memory_system.short_term_memory.len(), 0);
        assert_eq!(memory_system.long_term_memory.len(), 0);
        assert_eq!(memory_system.pattern_store.len(), 0);
    }

    #[test]
    fn test_memory_storage_and_recall() {
        let mut memory_system = AgentMemorySystem::new();

        let content = MemoryContent::TaskExperience {
            task_id: Uuid::new_v4(),
            task_type: "analysis".to_string(),
            outcome: TaskOutcome::Success {
                efficiency: 0.9,
                quality: 0.8,
            },
            learned_strategy: Some("focus on key patterns".to_string()),
        };

        let memory_id = memory_system.store_memory(
            content,
            0.8,
            vec!["task".to_string(), "analysis".to_string()],
        );

        let recalled = memory_system.recall_memories("analysis", 5);
        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].id, memory_id);
    }

    #[test]
    fn test_social_memory_updates() {
        let mut memory_system = AgentMemorySystem::new();
        let agent_id = Uuid::new_v4();

        // Initial interaction
        memory_system.update_social_memory(agent_id, true, Some(Uuid::new_v4()));
        assert!(memory_system.get_trust_level(agent_id) > 0.5);

        // Failed interaction
        memory_system.update_social_memory(agent_id, false, Some(Uuid::new_v4()));
        let trust_after_failure = memory_system.get_trust_level(agent_id);
        assert!(trust_after_failure < 0.6); // Should decrease but not below initial + first success
    }

    #[test]
    fn test_pattern_learning() {
        let mut memory_system = AgentMemorySystem::new();

        let pattern_id = memory_system.learn_pattern(
            "task_optimization".to_string(),
            vec!["high_workload".to_string(), "time_pressure".to_string()],
            0.85,
        );

        let patterns = memory_system.get_matching_patterns(&["high_workload".to_string()]);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_id, pattern_id);
    }
}
