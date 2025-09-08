//! # Collaborative Learning System
//!
//! Enables agents to share knowledge and learn collectively through peer-to-peer
//! knowledge transfer, collective intelligence, and distributed problem solving.

use crate::agents::agent::{Agent, AgentCapability};
use crate::neural::ProcessedText;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, debug, warn};

/// Represents shared knowledge that can be transferred between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedKnowledge {
    pub id: Uuid,
    pub source_agent: Uuid,
    pub knowledge_type: KnowledgeType,
    pub content: String,
    pub confidence: f64,
    pub success_rate: f64,
    pub usage_count: u32,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub tags: Vec<String>,
}

/// Types of knowledge that can be shared between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeType {
    TaskSolution(String),      // Solution to a specific task type
    PatternRecognition(String), // Recognized patterns in data
    OptimizationStrategy(String), // Performance optimization techniques
    ErrorRecovery(String),     // How to recover from specific errors
    BestPractice(String),      // General best practices
    DomainExpertise(String),   // Domain-specific knowledge
}

/// Learning session where agents collaborate on a problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeLearningSession {
    pub session_id: Uuid,
    pub participants: Vec<Uuid>,
    pub problem_description: String,
    pub session_type: SessionType,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub knowledge_generated: Vec<Uuid>,
    pub performance_improvement: f64,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    ProblemSolving,    // Collaborative problem solving
    KnowledgeSharing,  // Sharing existing knowledge
    SkillTransfer,     // Teaching specific skills
    ExperimentDesign,  // Designing experiments together
    PerformanceReview, // Reviewing and improving performance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Planning,
    Active,
    Completed,
    Failed,
    Cancelled,
}

/// Manages collaborative learning between agents
pub struct CollaborativeLearningSystem {
    knowledge_base: HashMap<Uuid, SharedKnowledge>,
    active_sessions: HashMap<Uuid, CollaborativeLearningSession>,
    agent_knowledge_map: HashMap<Uuid, Vec<Uuid>>, // Agent -> Knowledge IDs
    knowledge_network: HashMap<Uuid, Vec<Uuid>>,   // Knowledge -> Related Knowledge
    learning_metrics: LearningMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct LearningMetrics {
    pub total_knowledge_items: usize,
    pub successful_transfers: u32,
    pub failed_transfers: u32,
    pub average_confidence: f64,
    pub knowledge_utilization_rate: f64,
    pub collaborative_sessions: u32,
    pub performance_improvements: Vec<f64>,
}

impl CollaborativeLearningSystem {
    pub fn new() -> Self {
        Self {
            knowledge_base: HashMap::new(),
            active_sessions: HashMap::new(),
            agent_knowledge_map: HashMap::new(),
            knowledge_network: HashMap::new(),
            learning_metrics: LearningMetrics::default(),
        }
    }

    /// Create a new collaborative learning session
    pub async fn start_learning_session(
        &mut self,
        participants: Vec<Uuid>,
        problem_description: String,
        session_type: SessionType,
    ) -> Result<Uuid> {
        let session_id = Uuid::new_v4();

        let session = CollaborativeLearningSession {
            session_id,
            participants: participants.clone(),
            problem_description: problem_description.clone(),
            session_type,
            start_time: Utc::now(),
            end_time: None,
            knowledge_generated: Vec::new(),
            performance_improvement: 0.0,
            status: SessionStatus::Planning,
        };

        self.active_sessions.insert(session_id, session);
        self.learning_metrics.collaborative_sessions += 1;

        info!("Started collaborative learning session {} with {} participants",
              session_id, participants.len());

        Ok(session_id)
    }

    /// Add knowledge to the shared knowledge base
    pub async fn contribute_knowledge(
        &mut self,
        source_agent: Uuid,
        knowledge_type: KnowledgeType,
        content: String,
        confidence: f64,
        tags: Vec<String>,
    ) -> Result<Uuid> {
        let knowledge_id = Uuid::new_v4();

        let knowledge = SharedKnowledge {
            id: knowledge_id,
            source_agent,
            knowledge_type,
            content,
            confidence,
            success_rate: 0.5, // Initial neutral success rate
            usage_count: 0,
            created_at: Utc::now(),
            last_used: Utc::now(),
            tags,
        };

        self.knowledge_base.insert(knowledge_id, knowledge);

        // Update agent knowledge mapping
        self.agent_knowledge_map
            .entry(source_agent)
            .or_insert_with(Vec::new)
            .push(knowledge_id);

        self.learning_metrics.total_knowledge_items = self.knowledge_base.len();

        debug!("Agent {} contributed knowledge item {}", source_agent, knowledge_id);
        Ok(knowledge_id)
    }

    /// Find relevant knowledge for a specific problem or task
    pub async fn find_relevant_knowledge(
        &self,
        query: &str,
        agent_id: Uuid,
        max_results: usize,
    ) -> Result<Vec<SharedKnowledge>> {
        let mut relevant_knowledge = Vec::new();
        let query_lower = query.to_lowercase();

        for knowledge in self.knowledge_base.values() {
            // Skip knowledge from the same agent (unless it's very successful)
            if knowledge.source_agent == agent_id && knowledge.success_rate < 0.8 {
                continue;
            }

            let mut relevance_score = 0.0;

            // Content matching
            if knowledge.content.to_lowercase().contains(&query_lower) {
                relevance_score += 0.4;
            }

            // Tag matching
            for tag in &knowledge.tags {
                if query_lower.contains(&tag.to_lowercase()) {
                    relevance_score += 0.2;
                }
            }

            // Knowledge type matching
            let type_relevance = match &knowledge.knowledge_type {
                KnowledgeType::TaskSolution(task_type) => {
                    if query_lower.contains(&task_type.to_lowercase()) { 0.5 } else { 0.1 }
                }
                KnowledgeType::PatternRecognition(pattern) => {
                    if query_lower.contains(&pattern.to_lowercase()) { 0.4 } else { 0.1 }
                }
                _ => 0.2,
            };
            relevance_score += type_relevance;

            // Boost by confidence and success rate
            relevance_score *= knowledge.confidence * knowledge.success_rate;

            if relevance_score > 0.3 {
                relevant_knowledge.push((knowledge.clone(), relevance_score));
            }
        }

        // Sort by relevance score
        relevant_knowledge.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or_else(|| {
                tracing::warn!("Failed to compare relevance scores, treating as equal");
                std::cmp::Ordering::Equal
            })
        });

        Ok(relevant_knowledge
            .into_iter()
            .take(max_results)
            .map(|(knowledge, _)| knowledge)
            .collect())
    }

    /// Transfer knowledge from one agent to another
    pub async fn transfer_knowledge(
        &mut self,
        knowledge_id: Uuid,
        from_agent: Uuid,
        to_agent: Uuid,
    ) -> Result<bool> {
        if let Some(knowledge) = self.knowledge_base.get_mut(&knowledge_id) {
            // Update usage statistics
            knowledge.usage_count += 1;
            knowledge.last_used = Utc::now();

            // Add to recipient's knowledge map
            self.agent_knowledge_map
                .entry(to_agent)
                .or_insert_with(Vec::new)
                .push(knowledge_id);

            self.learning_metrics.successful_transfers += 1;

            info!("Transferred knowledge {} from agent {} to agent {}",
                  knowledge_id, from_agent, to_agent);

            Ok(true)
        } else {
            self.learning_metrics.failed_transfers += 1;
            warn!("Failed to transfer knowledge {}: not found", knowledge_id);
            Ok(false)
        }
    }

    /// Update knowledge success rate based on usage outcomes
    pub async fn update_knowledge_success(
        &mut self,
        knowledge_id: Uuid,
        was_successful: bool,
    ) -> Result<()> {
        if let Some(knowledge) = self.knowledge_base.get_mut(&knowledge_id) {
            // Update success rate using exponential moving average
            let alpha = 0.1; // Learning rate
            let new_success = if was_successful { 1.0 } else { 0.0 };
            knowledge.success_rate = (1.0 - alpha) * knowledge.success_rate + alpha * new_success;

            debug!("Updated knowledge {} success rate to {:.3}",
                   knowledge_id, knowledge.success_rate);
        }

        Ok(())
    }

    /// Facilitate collaborative problem solving
    pub async fn collaborate_on_problem(
        &mut self,
        session_id: Uuid,
        problem_data: &ProcessedText,
        participating_agents: &[Agent],
    ) -> Result<CollaborationResult> {
        let mut session = self.active_sessions
            .get_mut(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?
            .clone();

        session.status = SessionStatus::Active;

        let mut collaboration_result = CollaborationResult {
            session_id,
            solutions: Vec::new(),
            knowledge_created: Vec::new(),
            performance_gain: 0.0,
            participant_contributions: HashMap::new(),
        };

        // Gather relevant knowledge from all participants
        let mut collective_knowledge = Vec::new();
        for agent in participating_agents {
            if let Some(agent_knowledge_ids) = self.agent_knowledge_map.get(&agent.id) {
                for &knowledge_id in agent_knowledge_ids {
                    if let Some(knowledge) = self.knowledge_base.get(&knowledge_id) {
                        collective_knowledge.push(knowledge.clone());
                    }
                }
            }
        }

        // Synthesize solutions from collective knowledge
        let synthesized_solution = self.synthesize_collective_solution(
            &problem_data.keywords,
            &collective_knowledge,
        ).await?;

        collaboration_result.solutions.push(synthesized_solution);

        // Create new knowledge from collaboration
        if !collective_knowledge.is_empty() {
            let new_knowledge_id = self.contribute_knowledge(
                session.participants[0], // Primary contributor
                KnowledgeType::TaskSolution("collaborative_solution".to_string()),
                format!("Collaborative solution for: {}", problem_data.keywords.join(", ")),
                0.8, // High confidence for collaborative solutions
                problem_data.keywords.clone(),
            ).await?;

            collaboration_result.knowledge_created.push(new_knowledge_id);
            session.knowledge_generated.push(new_knowledge_id);
        }

        // Calculate performance improvement
        let baseline_performance = participating_agents.iter()
            .map(|agent| agent.memory.experiences.len() as f64 / 100.0)
            .sum::<f64>() / participating_agents.len() as f64;

        collaboration_result.performance_gain = (collective_knowledge.len() as f64 * 0.1)
            .min(0.5); // Cap at 50% improvement

        session.performance_improvement = collaboration_result.performance_gain;
        session.status = SessionStatus::Completed;
        session.end_time = Some(Utc::now());

        self.active_sessions.insert(session_id, session);
        self.learning_metrics.performance_improvements.push(collaboration_result.performance_gain);

        info!("Completed collaborative session {} with {:.1}% performance gain",
              session_id, collaboration_result.performance_gain * 100.0);

        Ok(collaboration_result)
    }

    /// Synthesize a solution from collective knowledge
    async fn synthesize_collective_solution(
        &self,
        problem_keywords: &[String],
        collective_knowledge: &[SharedKnowledge],
    ) -> Result<String> {
        if collective_knowledge.is_empty() {
            return Ok("No relevant knowledge available for synthesis".to_string());
        }

        // Find most relevant knowledge items
        let mut relevant_items = Vec::new();
        for knowledge in collective_knowledge {
            let mut relevance = 0.0;

            for keyword in problem_keywords {
                if knowledge.content.to_lowercase().contains(&keyword.to_lowercase()) {
                    relevance += 0.2;
                }
                for tag in &knowledge.tags {
                    if tag.to_lowercase().contains(&keyword.to_lowercase()) {
                        relevance += 0.1;
                    }
                }
            }

            if relevance > 0.1 {
                relevant_items.push((knowledge, relevance * knowledge.confidence));
            }
        }

        // Sort by relevance
        relevant_items.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or_else(|| {
                tracing::warn!("Failed to compare relevance scores in synthesis, treating as equal");
                std::cmp::Ordering::Equal
            })
        });

        // Synthesize solution from top items
        let mut solution_parts = Vec::new();
        for (knowledge, _score) in relevant_items.iter().take(3) {
            solution_parts.push(format!("- {}", knowledge.content));
        }

        let synthesized = if solution_parts.is_empty() {
            "Collaborative analysis suggests exploring alternative approaches".to_string()
        } else {
            format!("Synthesized collaborative solution:\n{}", solution_parts.join("\n"))
        };

        Ok(synthesized)
    }

    /// Get learning metrics and statistics
    pub fn get_learning_metrics(&self) -> LearningMetrics {
        let mut metrics = self.learning_metrics.clone();

        // Calculate average confidence
        if !self.knowledge_base.is_empty() {
            metrics.average_confidence = self.knowledge_base.values()
                .map(|k| k.confidence)
                .sum::<f64>() / self.knowledge_base.len() as f64;
        }

        // Calculate utilization rate
        let total_usage: u32 = self.knowledge_base.values()
            .map(|k| k.usage_count)
            .sum();

        if !self.knowledge_base.is_empty() {
            metrics.knowledge_utilization_rate = total_usage as f64 / self.knowledge_base.len() as f64;
        }

        metrics
    }

    /// Get knowledge network visualization data
    pub fn get_knowledge_network(&self) -> HashMap<Uuid, Vec<Uuid>> {
        self.knowledge_network.clone()
    }

    /// Get active learning sessions
    pub fn get_active_sessions(&self) -> Vec<&CollaborativeLearningSession> {
        self.active_sessions.values()
            .filter(|session| matches!(session.status, SessionStatus::Active | SessionStatus::Planning))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct CollaborationResult {
    pub session_id: Uuid,
    pub solutions: Vec<String>,
    pub knowledge_created: Vec<Uuid>,
    pub performance_gain: f64,
    pub participant_contributions: HashMap<Uuid, f64>,
}

impl Default for CollaborativeLearningSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_knowledge_contribution() -> Result<(), Box<dyn std::error::Error>> {
        let mut system = CollaborativeLearningSystem::new();
        let agent_id = Uuid::new_v4();

        let knowledge_id = system.contribute_knowledge(
            agent_id,
            KnowledgeType::TaskSolution("test_task".to_string()),
            "Test solution content".to_string(),
            0.8,
            vec!["test".to_string(), "solution".to_string()],
        ).await?;

        assert!(system.knowledge_base.contains_key(&knowledge_id));
        assert_eq!(system.learning_metrics.total_knowledge_items, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_knowledge_search() -> Result<(), Box<dyn std::error::Error>> {
        let mut system = CollaborativeLearningSystem::new();
        let agent_id = Uuid::new_v4();

        let _knowledge_id = system.contribute_knowledge(
            agent_id,
            KnowledgeType::TaskSolution("optimization".to_string()),
            "Performance optimization techniques".to_string(),
            0.9,
            vec!["performance".to_string(), "optimization".to_string()],
        ).await?;

        let results = system.find_relevant_knowledge(
            "performance optimization",
            Uuid::new_v4(), // Different agent
            5,
        ).await?;

        assert!(!results.is_empty());
        assert!(results[0].content.contains("optimization"));
        Ok(())
    }

    #[tokio::test]
    async fn test_collaborative_session() -> Result<(), Box<dyn std::error::Error>> {
        let mut system = CollaborativeLearningSystem::new();
        let participants = vec![Uuid::new_v4(), Uuid::new_v4()];

        let session_id = system.start_learning_session(
            participants.clone(),
            "Test problem".to_string(),
            SessionType::ProblemSolving,
        ).await?;

        assert!(system.active_sessions.contains_key(&session_id));
        assert_eq!(system.learning_metrics.collaborative_sessions, 1);
        Ok(())
    }
}
