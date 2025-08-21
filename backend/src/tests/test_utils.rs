//! Test utilities and common fixtures for the multiagent hive system tests


use crate::agents::{Agent, AgentCapability, AgentType};
use crate::tasks::{Task, TaskPriority, TaskRequiredCapability};

/// Creates a test agent with default configuration
pub fn create_test_agent(name: &str, agent_type: AgentType) -> Agent {
    let mut agent = Agent::new(name.to_string(), agent_type);
    
    // Add some basic capabilities
    agent.add_capability(AgentCapability {
        name: "general".to_string(),
        proficiency: 0.7,
        learning_rate: 0.1,
    });
    
    agent.add_capability(AgentCapability {
        name: "communication".to_string(),
        proficiency: 0.6,
        learning_rate: 0.15,
    });
    
    agent
}

/// Creates a test agent with specific capabilities
pub fn create_test_agent_with_capabilities(
    name: &str,
    agent_type: AgentType,
    capabilities: Vec<AgentCapability>,
) -> Agent {
    let mut agent = Agent::new(name.to_string(), agent_type);
    
    for capability in capabilities {
        agent.add_capability(capability);
    }
    
    agent
}

/// Creates a test task with default configuration
pub fn create_test_task(description: &str, task_type: &str, priority: TaskPriority) -> Task {
    Task::new(
        description.to_string(),
        description.to_string(),
        task_type.to_string(),
        priority,
        vec![],
    )
}

/// Creates a test task with required capabilities
pub fn create_test_task_with_requirements(
    description: &str,
    task_type: &str,
    priority: TaskPriority,
    required_capabilities: Vec<TaskRequiredCapability>,
) -> Task {
    Task::new(
        description.to_string(),
        description.to_string(),
        task_type.to_string(),
        priority,
        required_capabilities,
    )
}

/// Creates a test capability
pub fn create_test_capability(name: &str, proficiency: f64, learning_rate: f64) -> AgentCapability {
    AgentCapability {
        name: name.to_string(),
        proficiency: proficiency.clamp(0.0, 1.0),
        learning_rate: learning_rate.clamp(0.0, 1.0),
    }
}

/// Creates a test required capability
pub fn create_test_required_capability(name: &str, min_proficiency: f64) -> TaskRequiredCapability {
    TaskRequiredCapability {
        name: name.to_string(),
        minimum_proficiency: min_proficiency.clamp(0.0, 1.0),
    }
}

/// Asserts that two floating point numbers are approximately equal
pub fn assert_approx_eq(a: f64, b: f64, tolerance: f64) {
    assert!(
        (a - b).abs() < tolerance,
        "Values not approximately equal: {} vs {} (tolerance: {})",
        a,
        b,
        tolerance
    );
}

/// Creates a mock JSON configuration for agent creation
pub fn create_agent_config(
    name: &str,
    agent_type: &str,
    capabilities: Option<Vec<(&str, f64, f64)>>,
) -> serde_json::Value {
    let mut config = serde_json::json!({
        "name": name,
        "type": agent_type
    });
    
    if let Some(caps) = capabilities {
        let capabilities_json: Vec<serde_json::Value> = caps
            .into_iter()
            .map(|(name, proficiency, learning_rate)| {
                serde_json::json!({
                    "name": name,
                    "proficiency": proficiency,
                    "learning_rate": learning_rate
                })
            })
            .collect();
        
        config["capabilities"] = serde_json::Value::Array(capabilities_json);
    }
    
    config
}

/// Creates a mock JSON configuration for task creation
pub fn create_task_config(
    description: &str,
    task_type: &str,
    priority: u64,
    required_capabilities: Option<Vec<(&str, f64)>>,
) -> serde_json::Value {
    let mut config = serde_json::json!({
        "description": description,
        "type": task_type,
        "priority": priority
    });
    
    if let Some(req_caps) = required_capabilities {
        let capabilities_json: Vec<serde_json::Value> = req_caps
            .into_iter()
            .map(|(name, min_proficiency)| {
                serde_json::json!({
                    "name": name,
                    "min_proficiency": min_proficiency
                })
            })
            .collect();
        
        config["required_capabilities"] = serde_json::Value::Array(capabilities_json);
    }
    
    config
}