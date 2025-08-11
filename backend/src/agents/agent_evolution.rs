//! # Agent Evolution System
//! 
//! Implements genetic algorithm-inspired evolution for agents, allowing them to
//! adapt, mutate, and evolve their capabilities over time based on performance.

use crate::agents::agent::{Agent, AgentCapability, AgentType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rand::Rng;
use tracing::{info, debug};

/// Represents the genetic makeup of an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGenome {
    pub agent_id: Uuid,
    pub generation: u32,
    pub parent_ids: Vec<Uuid>,
    pub genes: Vec<Gene>,
    pub fitness_score: f64,
    pub mutation_rate: f64,
    pub created_at: DateTime<Utc>,
    pub evolution_history: Vec<EvolutionEvent>,
}

/// Individual genes that define agent characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gene {
    pub gene_type: GeneType,
    pub value: f64,
    pub expression_strength: f64,
    pub mutation_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeneType {
    LearningRate,
    Adaptability,
    Cooperation,
    Specialization(String),
    EnergyEfficiency,
    CommunicationSkill,
    ProblemSolving,
    Creativity,
    Resilience,
    Leadership,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEvent {
    pub event_type: EvolutionEventType,
    pub timestamp: DateTime<Utc>,
    pub fitness_before: f64,
    pub fitness_after: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionEventType {
    Mutation,
    Crossover,
    Selection,
    FitnessEvaluation,
    SpecializationChange,
}

/// Manages the evolution of agents in the hive
pub struct AgentEvolutionSystem {
    genomes: HashMap<Uuid, AgentGenome>,
    evolution_config: EvolutionConfig,
    generation_counter: u32,
    fitness_history: Vec<GenerationFitness>,
    species_registry: HashMap<String, Vec<Uuid>>,
}

#[derive(Debug, Clone)]
pub struct EvolutionConfig {
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub selection_pressure: f64,
    pub elite_preservation_rate: f64,
    pub max_generations: u32,
    pub fitness_threshold: f64,
    pub diversity_bonus: f64,
}

#[derive(Debug, Clone)]
pub struct GenerationFitness {
    pub generation: u32,
    pub average_fitness: f64,
    pub best_fitness: f64,
    pub diversity_score: f64,
    pub population_size: usize,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            selection_pressure: 0.8,
            elite_preservation_rate: 0.2,
            max_generations: 100,
            fitness_threshold: 0.95,
            diversity_bonus: 0.1,
        }
    }
}

impl AgentEvolutionSystem {
    pub fn new() -> Self {
        Self::with_config(EvolutionConfig::default())
    }

    pub fn with_config(config: EvolutionConfig) -> Self {
        Self {
            genomes: HashMap::new(),
            evolution_config: config,
            generation_counter: 0,
            fitness_history: Vec::new(),
            species_registry: HashMap::new(),
        }
    }

    /// Initialize genome for a new agent
    pub async fn initialize_agent_genome(&mut self, agent: &Agent) -> Result<AgentGenome> {
        let mut genes = Vec::new();
        let mut rng = rand::thread_rng();

        // Generate initial genes based on agent type and capabilities
        genes.push(Gene {
            gene_type: GeneType::LearningRate,
            value: rng.gen_range(0.01..0.3),
            expression_strength: 1.0,
            mutation_probability: 0.1,
        });

        genes.push(Gene {
            gene_type: GeneType::Adaptability,
            value: rng.gen_range(0.3..1.0),
            expression_strength: 1.0,
            mutation_probability: 0.15,
        });

        genes.push(Gene {
            gene_type: GeneType::EnergyEfficiency,
            value: agent.energy,
            expression_strength: 0.8,
            mutation_probability: 0.05,
        });

        // Add specialization genes based on agent type
        match &agent.agent_type {
            AgentType::Coordinator => {
                genes.push(Gene {
                    gene_type: GeneType::Leadership,
                    value: rng.gen_range(0.7..1.0),
                    expression_strength: 1.0,
                    mutation_probability: 0.08,
                });
                genes.push(Gene {
                    gene_type: GeneType::CommunicationSkill,
                    value: rng.gen_range(0.6..1.0),
                    expression_strength: 0.9,
                    mutation_probability: 0.1,
                });
            }
            AgentType::Specialist(specialization) => {
                genes.push(Gene {
                    gene_type: GeneType::Specialization(specialization.clone()),
                    value: rng.gen_range(0.8..1.0),
                    expression_strength: 1.0,
                    mutation_probability: 0.05,
                });
                genes.push(Gene {
                    gene_type: GeneType::ProblemSolving,
                    value: rng.gen_range(0.6..0.9),
                    expression_strength: 0.8,
                    mutation_probability: 0.12,
                });
            }
            AgentType::Worker => {
                genes.push(Gene {
                    gene_type: GeneType::Cooperation,
                    value: rng.gen_range(0.5..0.9),
                    expression_strength: 0.9,
                    mutation_probability: 0.1,
                });
                genes.push(Gene {
                    gene_type: GeneType::Resilience,
                    value: rng.gen_range(0.4..0.8),
                    expression_strength: 0.7,
                    mutation_probability: 0.15,
                });
            }
        }

        let genome = AgentGenome {
            agent_id: agent.id,
            generation: 0,
            parent_ids: Vec::new(),
            genes,
            fitness_score: 0.5, // Initial neutral fitness
            mutation_rate: self.evolution_config.mutation_rate,
            created_at: Utc::now(),
            evolution_history: Vec::new(),
        };

        self.genomes.insert(agent.id, genome.clone());
        
        // Register in species
        let species_key = self.determine_species(&genome);
        self.species_registry
            .entry(species_key)
            .or_insert_with(Vec::new)
            .push(agent.id);

        info!("Initialized genome for agent {} with {} genes", agent.id, genome.genes.len());
        Ok(genome)
    }

    /// Evaluate fitness of an agent based on performance metrics
    pub async fn evaluate_fitness(&mut self, agent_id: Uuid, performance_metrics: &PerformanceMetrics) -> Result<f64> {
        if let Some(genome) = self.genomes.get_mut(&agent_id) {
            let mut fitness = 0.0;

            // Base fitness from task performance
            fitness += performance_metrics.task_success_rate * 0.4;
            
            // Energy efficiency bonus
            fitness += performance_metrics.energy_efficiency * 0.2;
            
            // Collaboration effectiveness
            fitness += performance_metrics.collaboration_score * 0.2;
            
            // Learning progress
            fitness += performance_metrics.learning_progress * 0.1;
            
            // Adaptability bonus
            fitness += performance_metrics.adaptability_score * 0.1;

            // Apply diversity bonus
            let species_key = self.determine_species(genome);
            let species_size = self.species_registry.get(&species_key).map(|v| v.len()).unwrap_or(1);
            let diversity_bonus = self.evolution_config.diversity_bonus / (species_size as f64).sqrt();
            fitness += diversity_bonus;

            genome.fitness_score = fitness.clamp(0.0, 1.0);
            
            // Record fitness evaluation event
            genome.evolution_history.push(EvolutionEvent {
                event_type: EvolutionEventType::FitnessEvaluation,
                timestamp: Utc::now(),
                fitness_before: genome.fitness_score,
                fitness_after: genome.fitness_score,
                description: format!("Fitness evaluation: {:.3}", genome.fitness_score),
            });

            debug!("Evaluated fitness for agent {}: {:.3}", agent_id, genome.fitness_score);
            Ok(genome.fitness_score)
        } else {
            Err(anyhow::anyhow!("Agent genome not found"))
        }
    }

    /// Mutate an agent's genome
    pub async fn mutate_agent(&mut self, agent_id: Uuid) -> Result<bool> {
        if let Some(genome) = self.genomes.get_mut(&agent_id) {
            let mut rng = rand::thread_rng();
            let mut mutations_applied = 0;
            let fitness_before = genome.fitness_score;

            for gene in &mut genome.genes {
                if rng.gen::<f64>() < gene.mutation_probability {
                    let old_value = gene.value;
                    
                    // Apply mutation based on gene type
                    match gene.gene_type {
                        GeneType::LearningRate => {
                            gene.value = (gene.value + rng.gen_range(-0.05..0.05)).clamp(0.001, 0.5);
                        }
                        GeneType::Adaptability | GeneType::Cooperation | GeneType::EnergyEfficiency => {
                            gene.value = (gene.value + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
                        }
                        GeneType::Specialization(_) => {
                            gene.value = (gene.value + rng.gen_range(-0.05..0.05)).clamp(0.5, 1.0);
                        }
                        _ => {
                            gene.value = (gene.value + rng.gen_range(-0.15..0.15)).clamp(0.0, 1.0);
                        }
                    }

                    if (gene.value - old_value).abs() > 0.01 {
                        mutations_applied += 1;
                    }
                }
            }

            if mutations_applied > 0 {
                genome.evolution_history.push(EvolutionEvent {
                    event_type: EvolutionEventType::Mutation,
                    timestamp: Utc::now(),
                    fitness_before,
                    fitness_after: genome.fitness_score,
                    description: format!("Applied {} mutations", mutations_applied),
                });

                info!("Mutated agent {} with {} gene changes", agent_id, mutations_applied);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(anyhow::anyhow!("Agent genome not found"))
        }
    }

    /// Create offspring through crossover of two parent agents
    pub async fn crossover_agents(
        &mut self,
        parent1_id: Uuid,
        parent2_id: Uuid,
        offspring_agent: &Agent,
    ) -> Result<AgentGenome> {
        let parent1 = self.genomes.get(&parent1_id)
            .ok_or_else(|| anyhow::anyhow!("Parent 1 genome not found"))?
            .clone();
        
        let parent2 = self.genomes.get(&parent2_id)
            .ok_or_else(|| anyhow::anyhow!("Parent 2 genome not found"))?
            .clone();

        let mut rng = rand::thread_rng();
        let mut offspring_genes = Vec::new();

        // Combine genes from both parents
        let max_genes = parent1.genes.len().max(parent2.genes.len());
        
        for i in 0..max_genes {
            let gene = if i < parent1.genes.len() && i < parent2.genes.len() {
                // Both parents have this gene - blend them
                let p1_gene = &parent1.genes[i];
                let p2_gene = &parent2.genes[i];
                
                let blend_ratio = rng.gen::<f64>();
                Gene {
                    gene_type: p1_gene.gene_type.clone(),
                    value: p1_gene.value * blend_ratio + p2_gene.value * (1.0 - blend_ratio),
                    expression_strength: (p1_gene.expression_strength + p2_gene.expression_strength) / 2.0,
                    mutation_probability: (p1_gene.mutation_probability + p2_gene.mutation_probability) / 2.0,
                }
            } else if i < parent1.genes.len() {
                // Only parent 1 has this gene
                parent1.genes[i].clone()
            } else {
                // Only parent 2 has this gene
                parent2.genes[i].clone()
            };

            offspring_genes.push(gene);
        }

        let offspring_genome = AgentGenome {
            agent_id: offspring_agent.id,
            generation: parent1.generation.max(parent2.generation) + 1,
            parent_ids: vec![parent1_id, parent2_id],
            genes: offspring_genes,
            fitness_score: (parent1.fitness_score + parent2.fitness_score) / 2.0,
            mutation_rate: self.evolution_config.mutation_rate,
            created_at: Utc::now(),
            evolution_history: vec![EvolutionEvent {
                event_type: EvolutionEventType::Crossover,
                timestamp: Utc::now(),
                fitness_before: 0.0,
                fitness_after: (parent1.fitness_score + parent2.fitness_score) / 2.0,
                description: format!("Crossover between {} and {}", parent1_id, parent2_id),
            }],
        };

        self.genomes.insert(offspring_agent.id, offspring_genome.clone());

        info!("Created offspring agent {} from parents {} and {}", 
              offspring_agent.id, parent1_id, parent2_id);

        Ok(offspring_genome)
    }

    /// Select agents for reproduction based on fitness
    pub async fn select_for_reproduction(&self, population_size: usize) -> Result<Vec<Uuid>> {
        let mut candidates: Vec<(Uuid, f64)> = self.genomes.iter()
            .map(|(id, genome)| (*id, genome.fitness_score))
            .collect();

        // Sort by fitness (descending)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Select top performers and some random candidates for diversity
        let elite_count = (population_size as f64 * self.evolution_config.elite_preservation_rate) as usize;
        let mut selected = Vec::new();

        // Add elite performers
        for i in 0..elite_count.min(candidates.len()) {
            selected.push(candidates[i].0);
        }

        // Add tournament selection for remaining slots
        let mut rng = rand::thread_rng();
        while selected.len() < population_size && selected.len() < candidates.len() {
            let tournament_size = 3;
            let mut tournament: Vec<_> = (0..tournament_size)
                .map(|_| &candidates[rng.gen_range(0..candidates.len())])
                .collect();
            
            tournament.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            let winner = tournament[0].0;
            if !selected.contains(&winner) {
                selected.push(winner);
            }
        }

        Ok(selected)
    }

    /// Get evolution statistics
    pub fn get_evolution_stats(&self) -> EvolutionStats {
        let total_agents = self.genomes.len();
        let average_fitness = if total_agents > 0 {
            self.genomes.values().map(|g| g.fitness_score).sum::<f64>() / total_agents as f64
        } else {
            0.0
        };

        let best_fitness = self.genomes.values()
            .map(|g| g.fitness_score)
            .fold(0.0, f64::max);

        let average_generation = if total_agents > 0 {
            self.genomes.values().map(|g| g.generation as f64).sum::<f64>() / total_agents as f64
        } else {
            0.0
        };

        let species_count = self.species_registry.len();

        EvolutionStats {
            total_agents,
            average_fitness,
            best_fitness,
            average_generation,
            current_generation: self.generation_counter,
            species_count,
            mutation_events: self.count_evolution_events(EvolutionEventType::Mutation),
            crossover_events: self.count_evolution_events(EvolutionEventType::Crossover),
        }
    }

    fn determine_species(&self, genome: &AgentGenome) -> String {
        // Simple species classification based on dominant gene types
        let mut specialization_genes = Vec::new();
        let mut leadership_score = 0.0;
        let mut cooperation_score = 0.0;

        for gene in &genome.genes {
            match &gene.gene_type {
                GeneType::Specialization(spec) => specialization_genes.push(spec.clone()),
                GeneType::Leadership => leadership_score = gene.value,
                GeneType::Cooperation => cooperation_score = gene.value,
                _ => {}
            }
        }

        if leadership_score > 0.7 {
            "Leader".to_string()
        } else if cooperation_score > 0.8 {
            "Collaborator".to_string()
        } else if !specialization_genes.is_empty() {
            format!("Specialist_{}", specialization_genes[0])
        } else {
            "Generalist".to_string()
        }
    }

    fn count_evolution_events(&self, event_type: EvolutionEventType) -> u32 {
        self.genomes.values()
            .flat_map(|genome| &genome.evolution_history)
            .filter(|event| std::mem::discriminant(&event.event_type) == std::mem::discriminant(&event_type))
            .count() as u32
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub task_success_rate: f64,
    pub energy_efficiency: f64,
    pub collaboration_score: f64,
    pub learning_progress: f64,
    pub adaptability_score: f64,
}

#[derive(Debug, Clone)]
pub struct EvolutionStats {
    pub total_agents: usize,
    pub average_fitness: f64,
    pub best_fitness: f64,
    pub average_generation: f64,
    pub current_generation: u32,
    pub species_count: usize,
    pub mutation_events: u32,
    pub crossover_events: u32,
}

impl Default for AgentEvolutionSystem {
    fn default() -> Self {
        Self::new()
    }
}