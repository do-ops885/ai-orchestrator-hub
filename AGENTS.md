# Development Guidelines for Agentic Coding Agents

## Available Agents

### Core Agents
- **security-auditor**: Security analysis and vulnerability assessment
- **plan**: Project planning and task management
- **git**: Git operations and version control
- **learner**: Learning and adaptation capabilities
- **github**: GitHub repository management
- **swarm-coordinator**: Swarm intelligence and multi-agent coordination

### Swarm Coordinator Agent
The `swarm-coordinator` agent is specialized for multi-agent systems and swarm intelligence operations. It has access to the following MCP tools:

#### Multiagent Hive Tools
- `create_swarm_agent`: Create new agents (Worker, Coordinator, Specialist, Learner)
- `batch_create_agents`: Create multiple agents simultaneously
- `assign_swarm_task`: Create tasks with priority levels
- `get_swarm_status`: Monitor hive metrics and performance
- `list_agents`: List and filter agents
- `list_tasks`: List and filter tasks
- `get_agent_details`: Get detailed agent information
- `get_task_details`: Get detailed task information
- `coordinate_agents`: Apply coordination strategies
- `analyze_with_nlp`: Text analysis capabilities
- `system_info`: System information retrieval
- `echo`: Simple echo tool for testing

#### Usage Examples
```bash
# Create a swarm coordinator agent
opencode swarm-coordinator "Create 5 worker agents and assign them a code review task"

# Monitor swarm status
opencode swarm-coordinator "Check the current status of the multiagent hive"

# Coordinate agents
opencode swarm-coordinator "Apply balanced coordination strategy to optimize task distribution"
```

## Build/Lint/Test Commands

### Backend (Rust)

* **Build**: `cd backend && cargo build`
* **Test all**: `cd backend && cargo test`
* **Test single**: `cd backend && cargo test test_function_name`
* **Lint**: `cd backend && cargo clippy --all-targets --all-features`
* **Format**: `cd backend && cargo fmt`
* **Check format**: `cd backend && cargo fmt --check`

### Frontend (TypeScript/React)

* **Build**: `cd frontend && npm run build`
* **Test all**: `cd frontend && npm test`
* **Test single**: `cd frontend && npm test -- testName`
* **Test coverage**: `cd frontend && npm run test:coverage`
* **Lint**: `cd frontend && npm run lint`
* **Lint fix**: `cd frontend && npm run lint:fix`
* **Lint check**: `cd frontend && npm run lint:check`

---

## Code Style Guidelines

### Rust (Backend)

* **Formatting**: rustfmt with 100 char width, 4 spaces, Unix line endings
* **Naming**: snake\_case for variables/functions, PascalCase for types
* **Error handling**: Use `Result<T, E>` and `Option<T>`, avoid `unwrap`/`panic`
* **Imports**: Group by std, external crates, then local modules
* **Documentation**: Use `//!` for module docs, `///` for public items
* **Async**: Use `async fn` and `await`, prefer tokio runtime
* **Types**: Explicit types preferred, use `anyhow` for error handling
* **Memory safety**: Prefer `Arc`/`Rc` over raw pointers, avoid unnecessary clones

### TypeScript/React (Frontend)

* **Formatting**: ESLint with single quotes, no semicolons, trailing commas
* **Naming**: camelCase for variables/functions, PascalCase for components/types
* **Error handling**: try/catch blocks, error boundaries for React
* **Imports**: Use path mapping (`@/`), group by React, external, internal
* **Components**: Functional components with hooks, prefer composition over inheritance
* **Styling**: Tailwind CSS classes, consistent spacing
* **State**: Zustand store for global state, local state for components
* **Memory safety**: Clean up subscriptions, event listeners, intervals, and observers in `useEffect`

---

## General Best Practices

### File & Code Organization

* **Max lines per file**:

  * Rust: \~400–500 lines
  * TypeScript: \~300–400 lines
  * Split logic into modules/components when exceeding thresholds
* **Max function length**: \~40–50 lines; extract helpers when longer
* **Single Responsibility Principle (SRP)**: Each file/module should serve one clear purpose

### Memory Leak Prevention

* **Rust**: Use `Drop` trait when holding external resources, avoid reference cycles (`Rc<RefCell<T>>`)
* **TypeScript/React**:

  * Always clean up `useEffect` side effects
  * Cancel async requests on unmount
  * Remove event listeners and clear timers

### Logging

* **Rust**: Use `tracing` or `log` crates with structured context, never log secrets
* **TypeScript/React**: Use a centralized logger (e.g., `pino`, `winston`) with log levels
* **General**:

  * Levels: `debug`, `info`, `warn`, `error`
  * Include correlation/request IDs for traceability
  * Redact sensitive fields before logging

### Performance

* **Rust**: Prefer `&str` over `String` when borrowing, avoid heap allocations, use iterators
* **TypeScript/React**: Memoize expensive operations (`useMemo`, `useCallback`), lazy load routes/components
* **General**:

  * Minimize network requests
  * Batch operations where possible
  * Profile performance regularly

### Security

* **Secrets**: Never hardcode; load from environment variables/config
* **Validation**: Validate all external inputs (frontend forms + backend APIs)
* **Dependencies**: Keep up-to-date, run security audits (`cargo audit`, `npm audit`)
* **Frontend**: Escape user-generated content, enable CSP, use HTTPS
* **Backend**: Use TLS, sanitize DB queries (SQLx prepared statements, Supabase policies)
