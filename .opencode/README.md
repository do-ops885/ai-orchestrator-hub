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

### 🚀 **New High-Impact Commands**

#### `/deploy`

Deploy the application to various environments with automated checks.

- **Use case**: Production deployment with safety and monitoring
- **Features**: Blue-green deployment, canary releases, automated rollback, health monitoring
- **Agent**: `github-workflow-manager`

#### `/security-audit`

Comprehensive security audit for the entire codebase.

- **Use case**: Security vulnerability assessment and compliance
- **Features**: Dependency scanning, code analysis, configuration review, compliance validation
- **Agent**: `security-auditor`

#### `/quality-check`

Comprehensive quality assurance checks across the entire codebase.

- **Use case**: Automated quality validation and improvement
- **Features**: Code quality metrics, testing coverage, performance analysis, documentation review
- **Agent**: `quality-assurance`

#### `/ci-status`

Check and monitor CI/CD pipeline status with actionable insights.

- **Use case**: CI/CD pipeline monitoring and optimization
- **Features**: Pipeline health analysis, performance metrics, failure diagnostics, cost analysis
- **Agent**: `github-workflow-manager`

#### `/workflow-optimize`

Analyze and optimize GitHub Actions workflows for performance and cost efficiency.

- **Use case**: Workflow performance optimization
- **Features**: Parallelization analysis, caching optimization, cost reduction, reliability enhancement
- **Agent**: `github-workflow-optimizer`

#### `/code-review`

Comprehensive code review using multiple specialized agents.

- **Use case**: Automated multi-agent code review
- **Features**: AI analysis, security review, performance analysis, quality assessment, documentation review
- **Agent**: `ai-code-analysis-swarm`

#### `/debug-session`

Start comprehensive debugging session with multiple specialized agents.

- **Use case**: Multi-agent collaborative debugging
- **Features**: Rust debugging, React debugging, system analysis, root cause identification, solution development
- **Agent**: `coordinator`

### 📊 Performance & Analysis

#### `/benchmark`

Run performance benchmarks for the entire system.

- **Use case**: System performance testing
- **Features**: Neural network benchmarks, swarm performance, memory analysis, throughput metrics

#### `/analyze-deps`

Analyze and optimize project dependencies for Rust and Node.js.

- **Use case**: Dependency management
- **Features**: Unused dependency detection, security scanning, license checking

### 🧹 **Quality Assurance Commands**

#### `/lint-all`

Comprehensive linting across Rust, TypeScript, and configuration files.

- **Use case**: Code quality and consistency validation
- **Features**: Multi-language linting, auto-fixing, security checks, documentation validation
- **Agent**: `formatting-agent`

#### `/test-coverage`

Comprehensive test coverage analysis across Rust backend and React frontend.

- **Use case**: Test effectiveness and coverage validation
- **Features**: Multi-level coverage analysis, gap identification, automated test generation
- **Agent**: `test-runner`

#### `/validate-config`

Validate all configuration files across the project for correctness and security.

- **Use case**: Configuration validation and security
- **Features**: Syntax validation, schema checking, security analysis, environment consistency
- **Agent**: `false-positive-validator`

#### `/performance-profile`

Comprehensive performance profiling for Rust backend and React frontend.

- **Use case**: Performance analysis and optimization
- **Features**: CPU/memory profiling, bottleneck identification, optimization recommendations
- **Agent**: `performance-optimizer`

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

## Enhanced Agent Utilization

The OpenCode CLI now features improved agent utilization with:

### Multi-Agent Commands
- **`/code-review`**: Uses `ai-code-analysis-swarm` + multiple specialized agents
- **`/debug-session`**: Uses `coordinator` + Rust, React, and system agents
- **`/deploy`**: Uses `github-workflow-manager` with monitoring integration
- **`/security-audit`**: Uses `security-auditor` + `false-positive-validator`

### Specialized Agent Integration
- **Performance Agents**: `performance-optimizer`, `github-workflow-optimizer`
- **Quality Agents**: `quality-assurance`, `technical-reviewer`
- **Security Agents**: `security-auditor`, `false-positive-validator`
- **Development Agents**: `rust-developer`, `react-developer`, `formatting-agent`

## Command Structure

Each command is a markdown file with enhanced frontmatter:

```markdown
---description: Brief description of the commandagent: primary-agent---
Command content with instructions, shell commands, and multi-agent coordination.
```

### Advanced Features
- **Multi-Agent Coordination**: Commands can leverage multiple agents simultaneously
- **Interactive Workflows**: Step-by-step guided processes with agent collaboration
- **Automated Reporting**: Comprehensive reports with actionable insights
- **Integration Hooks**: Seamless integration with existing CI/CD and development workflows

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

## Enhanced Workflow Integration

### CI/CD Integration
- **Automated Quality Gates**: `/quality-check` integrates with CI pipelines
- **Security Scanning**: `/security-audit` runs automatically on code changes
- **Performance Monitoring**: `/ci-status` provides real-time pipeline insights
- **Deployment Automation**: `/deploy` supports blue-green and canary deployments

### Development Workflow
- **Pre-commit Hooks**: Quality and security checks before commits
- **Code Review Automation**: `/code-review` provides comprehensive analysis
- **Debugging Support**: `/debug-session` offers multi-agent debugging
- **Workflow Optimization**: `/workflow-optimize` improves CI/CD efficiency

### Monitoring & Analytics
- **Real-time Dashboards**: Live monitoring for deployments and CI/CD
- **Performance Analytics**: Cost and performance optimization insights
- **Quality Metrics**: Comprehensive quality and security reporting
- **Trend Analysis**: Historical analysis and predictive insights

## Best Practices

### Multi-Agent Collaboration
1. **Agent Selection**: Choose the most appropriate primary agent for each task
2. **Coordination**: Leverage coordinator agents for complex multi-agent workflows
3. **Specialization**: Utilize specialized agents for domain-specific expertise
4. **Integration**: Combine multiple agents for comprehensive analysis

### Quality & Security
1. **Automated Checks**: Run quality and security checks early and often
2. **Comprehensive Coverage**: Use multi-agent analysis for thorough assessment
3. **Continuous Monitoring**: Monitor systems continuously with automated alerts
4. **Compliance Focus**: Ensure compliance with security and quality standards

### Performance & Efficiency
1. **Optimization First**: Use optimization commands to improve performance
2. **Cost Awareness**: Monitor and optimize CI/CD costs
3. **Scalability**: Design workflows that scale with project growth
4. **Automation**: Automate repetitive tasks and manual processes

### Development Workflow
1. **Early Integration**: Integrate quality checks into development workflow
2. **Collaborative Debugging**: Use multi-agent debugging for complex issues
3. **Documentation**: Maintain comprehensive documentation of processes
4. **Continuous Learning**: Learn from debugging sessions and code reviews

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
