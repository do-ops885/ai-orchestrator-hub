# OpenCode Commands for AI Orchestrator Hub

This directory contains custom OpenCode commands specifically designed for the AI Orchestrator Hub project. These commands automate common development tasks and workflows for this complex AI/ML orchestration system with Rust backend and React frontend.

## Available Commands

### 🚀 Core Development Commands

#### `/build-backend`

Build the Rust backend with all agents and neural components.

- **Use case**: Backend compilation and optimization
- **Features**: Cargo build, WASM compilation, performance profiling

#### `/build-frontend`

Build the Next.js frontend with dashboard and visualization components.

- **Use case**: Frontend compilation and optimization
- **Features**: Next.js build, TypeScript checking, bundle analysis

#### `/test-all`

Run comprehensive tests across Rust backend and frontend.

- **Use case**: Complete test suite execution
- **Features**: Rust tests, integration tests, frontend tests, performance benchmarks

#### `/setup-dev`

Set up complete development environment for AI Orchestrator Hub.

- **Use case**: New developer onboarding
- **Features**: Rust toolchain, Node.js setup, WASM tools, testing frameworks

### 🧠 AI/ML Development Commands

#### `/swarm-simulate`

Simulate and analyze swarm intelligence behaviors.

- **Use case**: Swarm coordination testing and optimization
- **Features**: Agent interaction simulation, performance metrics, behavior analysis

#### `/neural-train`

Train and optimize neural network components.

- **Use case**: Neural network development and training
- **Features**: Model training, hyperparameter optimization, performance analysis

#### `/agent-monitor`

Monitor and analyze agent performance and health.

- **Use case**: Agent system monitoring
- **Features**: Real-time metrics, health checks, performance optimization

### 📊 Performance & Analysis

#### `/benchmark`

Run performance benchmarks for the entire system.

- **Use case**: System performance testing
- **Features**: Neural network benchmarks, swarm performance, memory analysis, throughput metrics

#### `/analyze-deps`

Analyze and optimize project dependencies for Rust and Node.js.

- **Use case**: Dependency management
- **Features**: Unused dependency detection, security scanning, license checking

### 📚 Documentation & Publishing

#### `/generate-docs`

Generate comprehensive documentation for the AI Orchestrator Hub.

- **Use case**: Documentation creation
- **Features**: API docs, architecture docs, agent documentation, usage guides

#### `/publish-all`

Publish packages to respective registries.

- **Use case**: Release management
- **Features**: Crates.io publishing, NPM publishing, Docker images

## Command Usage

### Running Commands

1. **In OpenCode TUI**: Type `/` followed by the command name

   ```
   /build-backend
   /test-all
   /swarm-simulate
   ```

2. **With Arguments**: Some commands accept arguments

   ```
   /agent-monitor --agent-type adaptive
   /benchmark --duration 60
   ```

3. **With File References**: Reference specific files
   ```
   /analyze-file @backend/src/agents/agent.rs
   ```

### Command Features

- **Shell Integration**: Commands can execute shell commands and include output
- **File References**: Include file contents in prompts using `@filename`
- **Agent Assignment**: Each command uses the most appropriate AI agent
- **Model Selection**: Optimized models for different task types

## Command Structure

Each command is a markdown file with frontmatter:

```markdown
---description: Brief description of the commandagent: rust-developer---
Command content with instructions and shell commands.
```

## Customization

### Adding New Commands

1. Create a new `.md` file in `.opencode/command/`
2. Add frontmatter with description, agent, and model
3. Write the command instructions
4. Use shell commands with `!command` syntax
5. Reference files with `@filename` syntax

### Modifying Existing Commands

Edit the corresponding `.md` file in `.opencode/command/` to:

- Change the agent or model
- Modify the command instructions
- Add new shell commands or file references
- Update the command description

## Best Practices

1. **Use Specific Agents**: Each command uses the most appropriate agent for the task
2. **Include Context**: Commands gather relevant information before making changes
3. **Handle Errors**: Commands analyze errors and provide solutions
4. **Optimize Performance**: Commands focus on performance and efficiency
5. **Maintain Documentation**: Commands help maintain up-to-date documentation

## Integration with Development Workflow

These commands integrate seamlessly with:

- **Git workflows**: Pre-commit checks, release automation
- **CI/CD pipelines**: Automated testing and deployment
- **Code review**: Automated code analysis and suggestions
- **Documentation**: Auto-generated API and usage documentation

## Troubleshooting

If commands don't work as expected:

1. Check that OpenCode is properly installed
2. Verify the command file syntax and frontmatter
3. Ensure required tools are installed (Rust, Node.js, etc.)
4. Check for any syntax errors in shell commands
5. Review the command output for specific error messages

## Contributing

To contribute new commands:

1. Follow the existing command structure
2. Test the command thoroughly
3. Add appropriate documentation
4. Ensure the command works across all relevant projects
5. Update this README with the new command information