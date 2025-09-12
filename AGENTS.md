# Agent Development Guidelines

## Build/Lint/Test Commands

### Backend (Rust)
- **Build**: `cd backend && cargo build`
- **Test all**: `cd backend && cargo test`
- **Test single**: `cd backend && cargo test test_function_name`
- **Lint**: `cd backend && cargo clippy --all-targets --all-features`
- **Format**: `cd backend && cargo fmt`
- **MAS-specific**: Run agent integration tests with `cargo test --test integration` to verify multi-agent communication

### Frontend (TypeScript/React)
- **Build**: `cd frontend && npm run build`
- **Test all**: `cd frontend && npm test`
- **Test single**: `cd frontend && npm test -- testName`
- **Test coverage**: `cd frontend && npm run test:coverage`
- **Lint**: `cd frontend && npm run lint`
- **Format**: `cd frontend && npm run format`
- **MAS-specific**: Use Playwright for E2E testing of agent dashboard interactions

## Code Style Guidelines

### Rust (Backend)
- **Formatting**: rustfmt (100 width, 4 spaces, Unix line endings)
- **Naming**: snake_case (vars/fns), PascalCase (types), SCREAMING_SNAKE_CASE (consts)
- **Error handling**: Result<T,E> + Option<T>, avoid unwrap/panic, use anyhow; implement agent failure recovery patterns
- **Imports**: Group std → external crates → local modules
- **Types**: Explicit types preferred, use Arc/Rc for shared ownership; define agent traits for polymorphism
- **Async**: async fn + await, tokio runtime; use async channels for agent communication
- **Memory**: Prefer &str over String, avoid unnecessary clones; implement resource pooling for agent instances
- **MAS-specific**: Agents should be stateless where possible; use message passing for state synchronization

### TypeScript/React (Frontend)
- **Formatting**: ESLint + Prettier (single quotes, no semicolons, trailing commas)
- **Naming**: camelCase (vars/fns), PascalCase (components/types)
- **Error handling**: try/catch, React error boundaries; implement agent communication error handling
- **Imports**: Path mapping (@/), group React → external → internal
- **Components**: Functional with hooks, composition over inheritance
- **State**: Zustand for global, local for components; use stores for agent state management
- **Styling**: Tailwind CSS classes
- **Memory**: Clean up useEffect side effects, event listeners, timers; implement agent subscription cleanup
- **MAS-specific**: Use custom hooks for agent communication; implement real-time agent status updates

## Code Organization

### Multi-Agent System Structure
- **Agent Modules**: Place individual agents in `backend/src/agents/` with clear naming (e.g., `adaptive_verification.rs`)
- **Communication Layer**: Centralize messaging protocols in `backend/src/communication/`
- **Core Orchestration**: Hive and scaling logic in `backend/src/core/hive/`
- **Shared Utilities**: Common agent functions in `backend/src/utils/`
- **Configuration**: Agent settings in `backend/settings/` with environment-specific overrides

### Agent Design Patterns
- **Trait-Based Architecture**: Implement `Agent` trait for consistent interfaces across agents
- **Message Passing**: Use async channels for inter-agent communication to ensure loose coupling
- **Lifecycle Management**: Implement initialization, execution, and cleanup phases for each agent
- **Modularity**: Keep agents focused on single responsibilities; compose complex behaviors through orchestration

## Testing Strategies

### Agent-Specific Testing
- **Unit Tests**: Test individual agent logic in isolation using mocks for dependencies
- **Behavior Verification**: Validate agent decision-making with predefined scenarios
- **Mock Communication**: Use test doubles for message passing to isolate agent functionality

### Integration Testing
- **Agent Interactions**: Test communication between agents using integration test suites
- **End-to-End MAS Testing**: Validate complete workflows from agent coordination to final output
- **Load Testing**: Simulate high agent concurrency to ensure system stability
- **Chaos Engineering**: Introduce failures to test agent resilience and recovery mechanisms

### MAS-Specific Testing Best Practices
- **Communication Reliability**: Test message delivery guarantees and error handling
- **Scalability Validation**: Verify performance under varying agent loads
- **State Consistency**: Ensure agent states remain synchronized across the system

## Documentation Standards

### Agent Documentation
- **Agent Specifications**: Document each agent's purpose, capabilities, inputs/outputs, and dependencies
- **Interface Contracts**: Clearly define message formats and communication protocols
- **Usage Examples**: Provide code samples showing agent integration and common use cases

### Code Documentation
- **Inline Comments**: Explain complex agent logic, especially decision algorithms
- **Docstrings**: Use Rust doc comments for public agent APIs and traits
- **README Files**: Maintain per-agent documentation in `backend/src/agents/` subdirectories

### MAS-Specific Documentation
- **System Architecture**: Document overall agent orchestration and data flow
- **Deployment Guides**: Detail agent configuration and scaling parameters
- **Troubleshooting**: Include common agent failure modes and resolution steps

## File Size Limits
- **Source Files**: Limit to 500 lines maximum for maintainability; split large agents into multiple files
- **Agent Modules**: Keep core agent logic under 300 lines; use separate files for utilities and tests
- **Exceptions**: Generated code, large data structures, or comprehensive test files may exceed limits with justification
- **Rationale**: Smaller files improve code review efficiency and reduce merge conflicts in MAS development
