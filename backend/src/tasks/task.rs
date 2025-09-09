use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequiredCapability {
    pub name: String,
    pub minimum_proficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub task_type: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub required_capabilities: Vec<TaskRequiredCapability>,
    pub assigned_agent: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub estimated_duration: Option<u64>, // in seconds
    pub context: HashMap<String, String>,
    pub dependencies: Vec<Uuid>, // task IDs this task depends on
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub agent_id: Uuid,
    pub success: bool,
    pub output: String,
    pub error_message: Option<String>,
    pub execution_time: u64, // in milliseconds
    pub completed_at: DateTime<Utc>,
    pub quality_score: Option<f64>, // 0.0 to 1.0
    pub learned_insights: Vec<String>,
}

pub struct TaskQueue {
    pub pending_tasks: VecDeque<Task>,
    pub assigned_tasks: HashMap<Uuid, Task>, // agent_id -> task
    pub completed_tasks: Vec<TaskResult>,
    pub failed_tasks: Vec<TaskResult>,
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueue {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending_tasks: VecDeque::new(),
            assigned_tasks: HashMap::new(),
            completed_tasks: Vec::new(),
            failed_tasks: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        match task.priority {
            TaskPriority::Critical => self.pending_tasks.push_front(task),
            TaskPriority::High => {
                // Insert after any critical tasks
                let mut insert_pos = 0;
                for (i, t) in self.pending_tasks.iter().enumerate() {
                    if matches!(t.priority, TaskPriority::Critical) {
                        insert_pos = i + 1;
                    } else {
                        break;
                    }
                }
                self.pending_tasks.insert(insert_pos, task);
            }
            _ => self.pending_tasks.push_back(task),
        }
    }

    pub fn get_next_task(&mut self) -> Option<Task> {
        self.pending_tasks.pop_front()
    }

    pub fn assign_task(&mut self, task: Task, agent_id: Uuid) {
        let mut assigned_task = task;
        assigned_task.assigned_agent = Some(agent_id);
        assigned_task.status = TaskStatus::Assigned;
        assigned_task.updated_at = Utc::now();
        self.assigned_tasks.insert(agent_id, assigned_task);
    }

    pub fn complete_task(&mut self, result: TaskResult) {
        if let Some(_task) = self.assigned_tasks.remove(&result.agent_id) {
            if result.success {
                self.completed_tasks.push(result);
            } else {
                self.failed_tasks.push(result);
            }
        }
    }

    #[must_use]
    pub fn get_task_by_agent(&self, agent_id: &Uuid) -> Option<&Task> {
        self.assigned_tasks.get(agent_id)
    }

    #[must_use]
    pub fn get_pending_count(&self) -> usize {
        self.pending_tasks.len()
    }

    #[must_use]
    pub fn get_assigned_count(&self) -> usize {
        self.assigned_tasks.len()
    }

    #[must_use]
    pub fn get_completed_count(&self) -> usize {
        self.completed_tasks.len()
    }

    #[must_use]
    pub fn get_failed_count(&self) -> usize {
        self.failed_tasks.len()
    }

    #[must_use]
    pub fn find_suitable_tasks(
        &self,
        capabilities: &[crate::agents::AgentCapability],
    ) -> Vec<&Task> {
        self.pending_tasks
            .iter()
            .filter(|task| {
                task.required_capabilities.iter().all(|req_cap| {
                    capabilities.iter().any(|agent_cap| {
                        agent_cap.name == req_cap.name
                            && agent_cap.proficiency >= req_cap.minimum_proficiency
                    })
                })
            })
            .collect()
    }

    /// Clear all tasks from the queue
    pub fn clear(&mut self) {
        self.pending_tasks.clear();
        self.assigned_tasks.clear();
        self.completed_tasks.clear();
        self.failed_tasks.clear();
    }

    /// Cleanup old tasks based on age
    pub async fn cleanup_old_tasks(&mut self, cutoff_time: DateTime<Utc>) -> anyhow::Result<usize> {
        let mut removed_count = 0;

        // Remove old pending tasks
        self.pending_tasks.retain(|task| {
            if task.created_at < cutoff_time {
                removed_count += 1;
                false
            } else {
                true
            }
        });

        // Remove old completed tasks
        self.completed_tasks.retain(|result| {
            if result.completed_at < cutoff_time {
                removed_count += 1;
                false
            } else {
                true
            }
        });

        // Remove old failed tasks
        self.failed_tasks.retain(|result| {
            if result.completed_at < cutoff_time {
                removed_count += 1;
                false
            } else {
                true
            }
        });

        Ok(removed_count)
    }
}

impl Task {
    #[must_use]
    pub fn new(
        title: String,
        description: String,
        task_type: String,
        priority: TaskPriority,
        required_capabilities: Vec<TaskRequiredCapability>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            task_type,
            priority,
            status: TaskStatus::Pending,
            required_capabilities,
            assigned_agent: None,
            created_at: now,
            updated_at: now,
            deadline: None,
            estimated_duration: None,
            context: HashMap::new(),
            dependencies: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    #[must_use]
    pub fn with_duration(mut self, duration_seconds: u64) -> Self {
        self.estimated_duration = Some(duration_seconds);
        self
    }

    #[must_use]
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }

    #[must_use]
    pub fn with_dependencies(mut self, dependencies: Vec<Uuid>) -> Self {
        self.dependencies = dependencies;
        self
    }

    #[must_use]
    pub fn is_ready_to_execute(&self, completed_tasks: &[TaskResult]) -> bool {
        if self.dependencies.is_empty() {
            return true;
        }

        let completed_task_ids: std::collections::HashSet<Uuid> = completed_tasks
            .iter()
            .filter(|result| result.success)
            .map(|result| result.task_id)
            .collect();

        self.dependencies
            .iter()
            .all(|dep_id| completed_task_ids.contains(dep_id))
    }
}

impl TaskResult {
    #[must_use]
    pub fn success(task_id: Uuid, agent_id: Uuid, output: String, execution_time: u64) -> Self {
        Self {
            task_id,
            agent_id,
            success: true,
            output,
            error_message: None,
            execution_time,
            completed_at: Utc::now(),
            quality_score: None,
            learned_insights: Vec::new(),
        }
    }

    #[must_use]
    pub fn failure(
        task_id: Uuid,
        agent_id: Uuid,
        error_message: String,
        execution_time: u64,
    ) -> Self {
        Self {
            task_id,
            agent_id,
            success: false,
            output: String::new(),
            error_message: Some(error_message),
            execution_time,
            completed_at: Utc::now(),
            quality_score: None,
            learned_insights: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_quality_score(mut self, score: f64) -> Self {
        self.quality_score = Some(score.clamp(0.0, 1.0));
        self
    }

    #[must_use]
    pub fn with_insights(mut self, insights: Vec<String>) -> Self {
        self.learned_insights = insights;
        self
    }
}
