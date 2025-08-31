---
description: Generate comprehensive documentation for the AI Orchestrator Hub
agent: docs-specialist
---

# Generate Documentation Command

Generate comprehensive documentation for the AI Orchestrator Hub project, including API documentation, architecture guides, and usage examples.

## Documentation Strategy

### 1. Environment Setup
Prepare documentation generation environment:

```bash
# Install documentation tools
cargo install cargo-doc
npm install -g typedoc
npm install -g redoc-cli
npm install -g vuepress

# Create documentation directory
mkdir -p docs/generated/$(date +%Y%m%d)
```

### 2. API Documentation
Generate API documentation:

```bash
# Rust API documentation
cd backend
cargo doc --no-deps --document-private-items
cp -r target/doc ../docs/generated/api/rust/

# TypeScript API documentation
cd ../frontend
npx typedoc --out ../docs/generated/api/typescript src/
```

### 3. Architecture Documentation
Generate architecture and system documentation:

```bash
# Generate system architecture diagrams
npx madge --image docs/generated/architecture/dependency-graph.svg src/

# Create component documentation
npx jsdoc -c jsdoc.conf.json -d docs/generated/components src/components/
```

### 4. Code Documentation
Generate code-level documentation:

```bash
# Rust code documentation
cd backend
cargo doc --open

# Generate documentation coverage report
cargo doc --document-private-items --no-deps
```

### 5. User Documentation
Generate user-facing documentation:

```bash
# Build user guide
cd docs
npm run build

# Generate usage examples
npx documentation build src/ --format md --output docs/generated/examples/
```

## Documentation Types

### API Documentation
- **REST API**: OpenAPI/Swagger specifications
- **GraphQL**: Schema documentation and playground
- **WebSocket**: Real-time API documentation
- **SDK Documentation**: Client library documentation

### Code Documentation
- **Function Documentation**: Parameter and return value documentation
- **Class Documentation**: Class purpose and usage
- **Module Documentation**: Module overview and structure
- **Example Code**: Usage examples and code snippets

### Architecture Documentation
- **System Architecture**: High-level system design
- **Component Diagrams**: Component relationships and interactions
- **Data Flow**: Data flow through the system
- **Deployment Diagrams**: System deployment architecture

### User Documentation
- **Getting Started**: Quick start guide
- **User Manual**: Detailed usage instructions
- **API Reference**: Complete API reference
- **Troubleshooting**: Common issues and solutions

## Documentation Generation

### Automated Documentation
Set up automated documentation generation:

```bash
# Generate all documentation
npm run docs:generate

# Build documentation site
npm run docs:build

# Deploy documentation
npm run docs:deploy
```

### Documentation Structure
Organize documentation structure:

```
docs/
├── api/                    # API documentation
│   ├── rest/              # REST API docs
│   ├── graphql/           # GraphQL schema docs
│   └── websocket/         # WebSocket API docs
├── architecture/          # System architecture
│   ├── overview/          # System overview
│   ├── components/        # Component documentation
│   └── deployment/        # Deployment guides
├── guides/                # User guides
│   ├── getting-started/   # Quick start guide
│   ├── tutorials/         # Step-by-step tutorials
│   └── best-practices/    # Best practices
├── reference/             # Reference documentation
│   ├── configuration/     # Configuration reference
│   ├── cli/              # CLI reference
│   └── api/              # API reference
└── generated/             # Auto-generated docs
```

## Content Generation

### API Documentation
Generate comprehensive API documentation:

```bash
# OpenAPI specification
npx swagger-jsdoc -d swaggerDef.js src/routes/*.js > docs/api/openapi.json

# Generate HTML documentation
npx redoc-cli build docs/api/openapi.json -o docs/generated/api/index.html

# Generate Postman collection
npx openapi-to-postmanv2 -s docs/api/openapi.json -o docs/api/postman.json
```

### Code Examples
Generate and validate code examples:

```bash
# Extract code examples from tests
npx documentation src/ --format md --output docs/generated/examples/

# Validate examples
npm run examples:validate

# Generate interactive examples
npx codesandboxer src/examples/ --output docs/generated/playground/
```

### Architecture Diagrams
Generate architecture diagrams:

```bash
# Generate dependency graphs
npx madge --image docs/generated/architecture/dependency-graph.svg src/

# Generate class diagrams
npx jsdoc-to-diagram src/ --output docs/generated/architecture/class-diagram.svg

# Generate sequence diagrams
npx mermaid-cli -i docs/architecture/sequences.mmd -o docs/generated/architecture/
```

## Quality Assurance

### Documentation Testing
Test documentation quality:

```bash
# Check links
npx link-check docs/

# Check spelling
npx cspell docs/

# Validate structure
npx documentation-lint docs/
```

### Content Validation
Validate documentation content:

```bash
# Check API documentation accuracy
npm run docs:test:api

# Validate code examples
npm run docs:test:examples

# Check documentation coverage
npm run docs:coverage
```

## Publishing and Deployment

### Documentation Site
Build and deploy documentation site:

```bash
# Build static site
npm run docs:build

# Preview locally
npm run docs:serve

# Deploy to GitHub Pages
npm run docs:deploy
```

### Version Management
Manage documentation versions:

```bash
# Tag documentation version
npm run docs:version -- v1.0.0

# Generate version comparison
npm run docs:diff -- v0.9.0 v1.0.0

# Archive old versions
npm run docs:archive
```

## Integration

### CI/CD Integration
Integrate documentation generation into CI:

```yaml
# GitHub Actions documentation workflow
name: Documentation
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Generate docs
        run: npm run docs:generate
      - name: Build docs site
        run: npm run docs:build
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/build
```

### Automated Updates
Set up automated documentation updates:

```bash
# Update API docs on code changes
npm run docs:api:update

# Update architecture docs
npm run docs:architecture:update

# Update examples
npm run docs:examples:update
```

## Best Practices

1. **Keep Documentation Current**: Update docs with code changes
2. **Use Clear Language**: Write for the target audience
3. **Include Examples**: Provide practical code examples
4. **Maintain Structure**: Keep documentation well-organized
5. **Version Documentation**: Match docs to software versions
6. **Test Documentation**: Validate examples and links
7. **Gather Feedback**: Collect user feedback on documentation

## Common Issues

- **Outdated Documentation**: Documentation not matching code
- **Missing Examples**: Lack of practical usage examples
- **Poor Organization**: Difficult to navigate documentation
- **Technical Jargon**: Using terminology without explanation
- **Broken Links**: Invalid or outdated links
- **Inconsistent Formatting**: Inconsistent documentation style
- **Missing API Coverage**: Incomplete API documentation