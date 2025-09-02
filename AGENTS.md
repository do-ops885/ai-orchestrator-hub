# Agent Development Guide

## Build/Lint/Test Commands

### Backend (Rust)
- **Build**: `cargo build` or `cargo build --release`
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Format**: `cargo fmt --all -- --check`
- **Test All**: `cargo test --all-features`
- **Single Test**: `cargo test test_name` or `cargo test -- --test-threads=1 test_name`
- **Run**: `cargo run` (basic) or `cargo run --features advanced-neural`

### Frontend (TypeScript)
- **Build**: `cd frontend && npm run build`
- **Lint**: `cd frontend && npm run lint:check`
- **Format**: `cd frontend && npm run lint:fix`
- **Type Check**: `cd frontend && npx tsc --noEmit`
- **Dev Server**: `cd frontend && npm run dev`

## Code Style Guidelines

### Rust
- **Line Width**: 100 characters max
- **Indentation**: 4 spaces, no tabs
- **Line Endings**: Unix (LF)
- **Imports**: Reorder with `cargo fmt`
- **Functions**: `fn_params_layout = "Tall"`
- **Error Handling**: Use `anyhow::Result<T>`
- **Naming**: snake_case for variables/functions, PascalCase for types
- **Avoid**: `unwrap()`, `panic!`, `clone_on_ref_ptr`

### TypeScript/React
- **Quotes**: Single quotes (`'`)
- **Semicolons**: Never use
- **Indentation**: 2 spaces
- **Line Endings**: Unix (LF)
- **Types**: Use `interface` over `type`
- **Components**: PascalCase, functional with hooks
- **Imports**: Group by external/internal, alphabetize
- **Error Handling**: Try-catch with proper error boundaries
- **Naming**: camelCase for variables/functions, PascalCase for components/types

### General
- **Configuration**: No hard-coded setting strings.
  - Rust backend → load from `settings/*.toml` with layered overrides (default, development, production).
  - React frontend → use Vite `import.meta.env` for build-time and a `settings.json` file for runtime values.
  - Secrets → never stored in config files, always injected via environment variables or a secret manager.
  - All settings must be validated (Rust: serde + validator; React: Zod).
- **Comments**: Only add when necessary for clarity, avoid redundancy.
- **Async**: Always use `async/await` for asynchronous operations.
- **Security**: Never expose secrets. Validate and sanitize all inputs.
- **Performance**: Follow `clippy` performance lints, prevent memory leaks, prefer efficient data structures.
- **Use the current runtime date and time from the operating environment.**
  - Always fetch the date/time dynamically from the system where the agent or process is running (e.g., terminal session on Windows, Linux, macOS).
  - Support differences in shell or environment: Bash, Zsh, PowerShell, Command Prompt, etc.
  - Never rely on the model’s internal knowledge cutoff date.
  - Use this runtime date/time whenever interpreting “today”, “this year”, or any time-sensitive queries. * Prefer standard formats like ISO 8601  (`YYYY-MM-DDTHH:MM:SSZ`) for consistency across systems.

## Best Practices for AI Orchestrator Hub

### File Size Limits & Code Organization
- **Code Size**: Keep functions under 50-100 lines, files under 300-700 lines
- **Single Responsibility Principle**: Each file should have one clear purpose
- **Bounded Contexts**: Divide large systems into smaller, focused contexts
- **Extract Class/Method**: Use refactoring techniques to break down large files
- **Module splitting**: Split large modules into focused, cohesive units
- **Bounded contexts for multiagent systems**: Separate agent coordination, neural processing, and communication layers

### Duplicate Code Avoidance
- **Extract Method**: Move duplicated code fragments into reusable methods
- **Extract Class**: Create new classes for shared functionality
- **Extract Superclass**: Use inheritance for common behavior
- **Template Method pattern**: Define algorithm skeletons in base classes
- **Trait-based extraction**: Use Rust traits for code reuse across agent types
- **Component composition**: Build reusable React components to avoid duplication

### Testing Best Practices with Edge Cases
- **Fuzz testing**: Automated input validation for robustness
- **Edge case coverage**: Test boundary conditions, error states, and unusual inputs
- **Property-based testing**: Test general properties rather than specific examples
- **Test pyramid approach**: Unit tests (base), integration tests (middle), e2e tests (top)
- **Neural feature testing**: Test edge cases in AI model interactions
- **WebSocket communication testing**: Test connection failures, message ordering, and timeouts

### Advanced Observability
- **OpenTelemetry integration**: Standardized telemetry across the stack
- **LLM observability**: Monitor AI model performance and hallucinations
- **GraphRAG implementation**: Enhanced knowledge retrieval for agents
- **Real-time monitoring**: WebSocket-based dashboards for swarm metrics
- **Performance profiling**: Track agent energy levels and task completion rates

### Security & Data Practices
- **Threat modeling**: AI-specific security considerations for multiagent systems
- **Just-in-time privileged access**: Dynamic authorization for agent operations
- **Data product thinking**: Consumer-centric data management for AI applications
- **MCP protocol security**: Secure model context protocol communications
- **Input validation**: Comprehensive validation for neural network inputs

### Performance Optimization
- **SIMD vectorization**: CPU-optimized operations for neural computations
- **Memory pooling**: Efficient memory management for agent swarms
- **Async optimization**: Tokio runtime tuning for concurrent agent operations
- **Caching strategies**: Intelligent caching for frequently accessed neural models
- **Resource monitoring**: Track and optimize agent energy consumption
