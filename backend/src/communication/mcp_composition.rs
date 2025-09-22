//! # Tool Composition Engine for Phase 4
//!
//! This module implements the Tool Composition Engine that enables complex workflow automation
//! through tool chaining, data flow management, and parallel execution. It supports:
//!
//! - **Tool Chaining**: Sequential and parallel execution of MCP tools
//! - **Data Flow**: Flexible input/output mapping between composition steps
//! - **Error Recovery**: Retry policies and graceful error handling
//! - **Parallel Execution**: Concurrent processing of independent steps
//! - **Dependency Management**: DAG-based execution with proper ordering

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing;

use crate::communication::mcp_tool_registry::MCPToolRegistry;
use crate::communication::mcp_unified_error::MCPUnifiedError;

/// Represents different types of data sources for composition inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    /// Output from a previous tool execution
    ToolOutput { tool_name: String, step_id: String },
    /// Static data provided at composition start
    StaticData { value: Value },
    /// Data from an external API call
    ExternalAPI {
        url: String,
        method: String,
        headers: HashMap<String, String>,
    },
    /// Output from a previous step in the composition
    PreviousStep { step_id: String },
}

/// Individual step in a tool composition workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionStep {
    /// Unique identifier for this step
    pub id: String,
    /// Name of the MCP tool to execute
    pub tool_name: String,
    /// Mapping of tool input parameters to data sources
    pub input_mapping: HashMap<String, DataSource>,
    /// Key to store the output of this step
    pub output_key: String,
    /// IDs of steps that must complete before this step can run
    pub dependencies: Vec<String>,
    /// Optional retry policy for error recovery
    pub retry_policy: Option<RetryPolicy>,
}

/// Retry policy for handling transient failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of execution attempts
    pub max_attempts: u32,
    /// Backoff delay in milliseconds between retries
    pub backoff_ms: u64,
}

/// Complete tool composition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolComposition {
    /// Unique identifier for the composition
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this composition does
    pub description: String,
    /// Ordered list of steps to execute
    pub steps: Vec<CompositionStep>,
    /// Initial input parameters required to start the composition
    pub initial_inputs: HashMap<String, Value>,
    /// Mapping of final outputs from step outputs
    pub output_mapping: HashMap<String, String>,
}

/// Execution context for a running composition
struct ExecutionContext {
    /// Completed step outputs
    step_outputs: HashMap<String, Value>,
    /// Initial inputs provided to the composition
    initial_inputs: HashMap<String, Value>,
    /// Tool registry for executing tools
    tool_registry: Arc<MCPToolRegistry>,
}

/// Engine for executing tool compositions with data flow and error handling
pub struct CompositionEngine {
    /// Registry of available MCP tools
    tool_registry: Arc<MCPToolRegistry>,
    /// Registered compositions
    compositions: Arc<RwLock<HashMap<String, ToolComposition>>>,
}

impl CompositionEngine {
    /// Create a new composition engine
    #[must_use]
    pub fn new(tool_registry: Arc<MCPToolRegistry>) -> Self {
        Self {
            tool_registry,
            compositions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new tool composition
    ///
    /// # Errors
    ///
    /// Returns `MCPUnifiedError::Validation` if the composition is invalid
    pub async fn register_composition(
        &self,
        composition: ToolComposition,
    ) -> Result<(), MCPUnifiedError> {
        // Validate composition structure
        self.validate_composition(&composition).await?;

        let composition_id = composition.id.clone();
        let mut compositions = self.compositions.write().await;
        compositions.insert(composition_id.clone(), composition);

        tracing::info!("Registered tool composition: {}", composition_id);
        Ok(())
    }

    /// Execute a registered composition with the given inputs
    ///
    /// # Errors
    ///
    /// Returns `MCPUnifiedError` if execution fails
    pub async fn execute_composition(
        &self,
        composition_id: &str,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, MCPUnifiedError> {
        let composition = {
            let compositions = self.compositions.read().await;
            compositions.get(composition_id).cloned().ok_or_else(|| {
                MCPUnifiedError::validation(
                    "composition_id".to_string(),
                    "Composition not found".to_string(),
                    Some(Value::String(composition_id.to_string())),
                    None,
                )
            })?
        };

        // Validate inputs
        self.validate_inputs(&composition, &inputs)?;

        let context = ExecutionContext {
            step_outputs: HashMap::new(),
            initial_inputs: inputs,
            tool_registry: Arc::clone(&self.tool_registry),
        };

        // Execute steps in dependency order
        let executed_steps = self.execute_steps(composition.steps, context).await?;

        // Map final outputs
        let mut final_outputs = HashMap::new();
        for (output_key, step_key) in &composition.output_mapping {
            if let Some(value) = executed_steps.get(step_key) {
                final_outputs.insert(output_key.clone(), value.clone());
            } else {
                return Err(MCPUnifiedError::validation(
                    "output_mapping".to_string(),
                    format!("Step output not found: {}", step_key),
                    None,
                    None,
                ));
            }
        }

        tracing::info!("Successfully executed composition: {}", composition_id);
        Ok(final_outputs)
    }

    /// Get a registered composition by ID
    ///
    /// # Errors
    ///
    /// Returns `MCPUnifiedError::Validation` if composition not found
    pub async fn get_composition(
        &self,
        composition_id: &str,
    ) -> Result<ToolComposition, MCPUnifiedError> {
        let compositions = self.compositions.read().await;
        compositions.get(composition_id).cloned().ok_or_else(|| {
            MCPUnifiedError::validation(
                "composition_id".to_string(),
                "Composition not found".to_string(),
                Some(Value::String(composition_id.to_string())),
                None,
            )
        })
    }

    /// List all registered composition IDs
    pub async fn list_compositions(&self) -> Vec<String> {
        let compositions = self.compositions.read().await;
        compositions.keys().cloned().collect()
    }

    /// Validate composition structure and dependencies
    async fn validate_composition(
        &self,
        composition: &ToolComposition,
    ) -> Result<(), MCPUnifiedError> {
        if composition.steps.is_empty() {
            return Err(MCPUnifiedError::validation(
                "steps".to_string(),
                "Composition must have at least one step".to_string(),
                None,
                None,
            ));
        }

        let mut step_ids = HashSet::new();
        for step in &composition.steps {
            if !step_ids.insert(step.id.clone()) {
                return Err(MCPUnifiedError::validation(
                    "step.id".to_string(),
                    format!("Duplicate step ID: {}", step.id),
                    None,
                    None,
                ));
            }

            // Validate tool exists
            if self.tool_registry.get_tool(&step.tool_name).await.is_none() {
                return Err(MCPUnifiedError::validation(
                    "tool_name".to_string(),
                    format!("Tool not found in registry: {}", step.tool_name),
                    None,
                    None,
                ));
            }

            // Validate dependencies exist
            for dep in &step.dependencies {
                if !step_ids.contains(dep) {
                    return Err(MCPUnifiedError::validation(
                        "dependencies".to_string(),
                        format!("Dependency step not found: {}", dep),
                        None,
                        None,
                    ));
                }
            }
        }

        // Check for circular dependencies using topological sort
        if self.has_circular_dependencies(&composition.steps) {
            return Err(MCPUnifiedError::validation(
                "dependencies".to_string(),
                "Circular dependency detected in composition steps".to_string(),
                None,
                None,
            ));
        }

        Ok(())
    }

    /// Validate that all required inputs are provided
    fn validate_inputs(
        &self,
        composition: &ToolComposition,
        inputs: &HashMap<String, Value>,
    ) -> Result<(), MCPUnifiedError> {
        for (key, expected) in &composition.initial_inputs {
            if !inputs.contains_key(key) {
                return Err(MCPUnifiedError::validation(
                    "inputs".to_string(),
                    format!("Missing required input: {}", key),
                    Some(expected.clone()),
                    None,
                ));
            }
        }
        Ok(())
    }

    /// Execute steps in dependency order with parallelization where possible
    async fn execute_steps(
        &self,
        steps: Vec<CompositionStep>,
        mut context: ExecutionContext,
    ) -> Result<HashMap<String, Value>, MCPUnifiedError> {
        let mut remaining_steps: HashMap<String, CompositionStep> = steps
            .into_iter()
            .map(|step| (step.id.clone(), step))
            .collect();

        let mut completed = HashSet::new();

        while !remaining_steps.is_empty() {
            // Find steps that can be executed (all dependencies satisfied)
            let ready_steps: Vec<CompositionStep> = remaining_steps
                .values()
                .filter(|step| step.dependencies.iter().all(|dep| completed.contains(dep)))
                .cloned()
                .collect();

            if ready_steps.is_empty() {
                return Err(MCPUnifiedError::validation(
                    "dependencies".to_string(),
                    "No steps ready to execute - possible circular dependency".to_string(),
                    None,
                    None,
                ));
            }

            // Execute ready steps in parallel
            let execution_futures: Vec<_> = ready_steps
                .into_iter()
                .map(|step| {
                    let ctx = &context;
                    async move {
                        let result = self.execute_step_with_retry(step.clone(), ctx).await;
                        (step.id, result)
                    }
                })
                .collect();

            let results = join_all(execution_futures).await;

            // Process results
            for (step_id, result) in results {
                match result {
                    Ok(output) => {
                        context.step_outputs.insert(step_id.clone(), output);
                        completed.insert(step_id.clone());
                        remaining_steps.remove(&step_id);
                    }
                    Err(e) => {
                        tracing::error!("Step {} failed: {}", step_id, e);
                        return Err(e);
                    }
                }
            }
        }

        Ok(context.step_outputs)
    }

    /// Execute a single step with retry logic
    async fn execute_step_with_retry(
        &self,
        step: CompositionStep,
        context: &ExecutionContext,
    ) -> Result<Value, MCPUnifiedError> {
        let max_attempts = step.retry_policy.as_ref().map_or(1, |p| p.max_attempts);
        let backoff_ms = step.retry_policy.as_ref().map_or(0, |p| p.backoff_ms);

        let mut last_error = None;

        for attempt in 1..=max_attempts {
            match self.execute_step(&step, context).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!("Step {} attempt {} failed: {}", step.id, attempt, e);
                    last_error = Some(e);

                    if attempt < max_attempts && backoff_ms > 0 {
                        sleep(Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            MCPUnifiedError::tool_execution(
                step.tool_name,
                "All retry attempts failed".to_string(),
                None,
                None,
            )
        }))
    }

    /// Execute a single composition step
    async fn execute_step(
        &self,
        step: &CompositionStep,
        context: &ExecutionContext,
    ) -> Result<Value, MCPUnifiedError> {
        // Resolve inputs from data sources
        let mut resolved_inputs = HashMap::new();

        for (param_name, data_source) in &step.input_mapping {
            let value = match data_source {
                DataSource::StaticData { value } => value.clone(),
                DataSource::PreviousStep { step_id } => {
                    context.step_outputs.get(step_id).cloned().ok_or_else(|| {
                        MCPUnifiedError::validation(
                            "input_mapping".to_string(),
                            format!("Previous step output not found: {}", step_id),
                            None,
                            None,
                        )
                    })?
                }
                DataSource::ToolOutput { tool_name, step_id } => {
                    // For now, treat as previous step - could be extended for external tool calls
                    context.step_outputs.get(step_id).cloned().ok_or_else(|| {
                        MCPUnifiedError::validation(
                            "input_mapping".to_string(),
                            format!("Tool output not found: {} from step {}", tool_name, step_id),
                            None,
                            None,
                        )
                    })?
                }
                DataSource::ExternalAPI {
                    url,
                    method,
                    headers: _,
                } => {
                    // Placeholder for external API calls - would need HTTP client
                    return Err(MCPUnifiedError::validation(
                        "input_mapping".to_string(),
                        "External API calls not yet implemented".to_string(),
                        Some(Value::String(format!("{} {}", method, url))),
                        None,
                    ));
                }
            };
            resolved_inputs.insert(param_name.clone(), value);
        }

        // Get tool from registry
        let tool = context
            .tool_registry
            .get_tool(&step.tool_name)
            .await
            .ok_or_else(|| {
                MCPUnifiedError::validation(
                    "tool_name".to_string(),
                    format!("Tool not found: {}", step.tool_name),
                    None,
                    None,
                )
            })?;

        // Execute tool
        let params = Value::Object(resolved_inputs.into_iter().map(|(k, v)| (k, v)).collect());

        tracing::debug!("Executing step {} with tool {}", step.id, step.tool_name);
        tool.execute(&params).await.map_err(|e| {
            MCPUnifiedError::tool_execution(
                step.tool_name.clone(),
                format!("Tool execution failed: {}", e),
                Some(params),
                None,
            )
        })
    }

    /// Check for circular dependencies in composition steps
    fn has_circular_dependencies(&self, steps: &[CompositionStep]) -> bool {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut indegree: HashMap<String, usize> = HashMap::new();

        // Build graph
        for step in steps {
            graph.entry(step.id.clone()).or_default();
            indegree.entry(step.id.clone()).or_insert(0);

            for dep in &step.dependencies {
                graph.entry(dep.clone()).or_default().push(step.id.clone());
                *indegree.entry(step.id.clone()).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm for topological sort
        let mut queue: Vec<String> = indegree
            .iter()
            .filter_map(|(id, &deg)| if deg == 0 { Some(id.clone()) } else { None })
            .collect();

        let mut processed = 0;

        while let Some(node) = queue.pop() {
            processed += 1;
            if let Some(neighbors) = graph.get(&node) {
                for neighbor in neighbors {
                    if let Some(deg) = indegree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        processed != steps.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::communication::mcp_tool_registry::{CachingStrategy, MCPToolRegistry};
    use async_trait::async_trait;

    // Mock tool for testing
    struct MockTool;

    #[async_trait]
    impl crate::communication::mcp::MCPToolHandler for MockTool {
        async fn execute(&self, params: &Value) -> Result<Value> {
            Ok(params.clone())
        }

        fn get_schema(&self) -> Value {
            serde_json::json!({})
        }

        fn get_description(&self) -> String {
            "Mock tool for testing".to_string()
        }
    }

    #[tokio::test]
    async fn test_register_and_execute_simple_composition() {
        let registry = Arc::new(MCPToolRegistry::new(300)); // 5 minutes TTL
        registry
            .register_simple_tool(
                "mock".to_string(),
                MockTool,
                "test",
                "Mock tool".to_string(),
                CachingStrategy::Never,
            )
            .await
            .expect("replaced unwrap");

        let engine = CompositionEngine::new(Arc::clone(&registry));

        let composition = ToolComposition {
            id: "test_comp".to_string(),
            name: "Test Composition".to_string(),
            description: "A simple test composition".to_string(),
            steps: vec![CompositionStep {
                id: "step1".to_string(),
                tool_name: "mock".to_string(),
                input_mapping: HashMap::from([(
                    "input".to_string(),
                    DataSource::StaticData {
                        value: Value::String("test".to_string()),
                    },
                )]),
                output_key: "result".to_string(),
                dependencies: vec![],
                retry_policy: None,
            }],
            initial_inputs: HashMap::new(),
            output_mapping: HashMap::from([("output".to_string(), "step1".to_string())]),
        };

        engine
            .register_composition(composition)
            .await
            .expect("replaced unwrap");

        let inputs = HashMap::new();
        let result = engine
            .execute_composition("test_comp", inputs)
            .await
            .expect("replaced unwrap");

        assert_eq!(
            result.get("output").expect("replaced unwrap"),
            &Value::String("test".to_string())
        );
    }
}
