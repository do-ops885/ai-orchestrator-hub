#!/bin/bash

# AI Orchestrator Hub API Documentation Generator
# Automatically generates comprehensive API documentation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKEND_DIR="$REPO_ROOT/backend"
FRONTEND_DIR="$REPO_ROOT/frontend"
DOCS_DIR="$REPO_ROOT/docs"
API_DOCS_DIR="$DOCS_DIR/api"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging function
log() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    case $level in
        "INFO")
            echo -e "${BLUE}[INFO]${NC} $timestamp - $message"
            ;;
        "WARN")
            echo -e "${YELLOW}[WARN]${NC} $timestamp - $message"
            ;;
        "ERROR")
            echo -e "${RED}[ERROR]${NC} $timestamp - $message"
            ;;
        "SUCCESS")
            echo -e "${GREEN}[SUCCESS]${NC} $timestamp - $message"
            ;;
    esac
}

# Function to check prerequisites
check_prerequisites() {
    log "INFO" "Checking prerequisites..."

    # Check if we're in the right directory
    if [[ ! -f "$BACKEND_DIR/Cargo.toml" ]]; then
        log "ERROR" "Backend directory not found. Please run from the ai-orchestrator-hub directory."
        exit 1
    fi

    if [[ ! -f "$FRONTEND_DIR/package.json" ]]; then
        log "ERROR" "Frontend directory not found. Please run from the ai-orchestrator-hub directory."
        exit 1
    fi

    # Check for required tools
    local missing_tools=()

    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi

    if ! command -v node &> /dev/null; then
        missing_tools+=("node")
    fi

    if ! command -v jq &> /dev/null; then
        missing_tools+=("jq")
    fi

    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        log "ERROR" "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi

    log "SUCCESS" "Prerequisites check passed"
}

# Function to extract Rust API endpoints from source code
extract_rust_endpoints() {
    log "INFO" "Extracting Rust API endpoints..."

    local endpoints_file="$API_DOCS_DIR/rust_endpoints.json"

    # Find all route definitions in the Rust code
    local routes=$(grep -r "route\|get\|post\|put\|delete\|patch" "$BACKEND_DIR/src" --include="*.rs" | \
                   grep -v "//" | \
                   sed 's/.*route//' | \
                   sed 's/.*get//' | \
                   sed 's/.*post//' | \
                   sed 's/.*put//' | \
                   sed 's/.*delete//' | \
                   sed 's/.*patch//' | \
                   sed 's/[^a-zA-Z0-9_/].*//' | \
                   sort | uniq)

    # Convert to JSON format
    local json_routes="[]"
    while IFS= read -r route; do
        if [[ -n "$route" ]]; then
            json_routes=$(echo "$json_routes" | jq --arg route "$route" '. + [{path: $route, method: "auto", source: "rust"}]')
        fi
    done <<< "$routes"

    echo "$json_routes" > "$endpoints_file"
    log "SUCCESS" "Extracted Rust endpoints to $endpoints_file"
}

# Function to extract TypeScript API calls from frontend
extract_typescript_endpoints() {
    log "INFO" "Extracting TypeScript API endpoints..."

    local endpoints_file="$API_DOCS_DIR/typescript_endpoints.json"

    # Find all API calls in the TypeScript/React code
    local api_calls=$(grep -r "fetch\|axios\|api/" "$FRONTEND_DIR/src" --include="*.ts" --include="*.tsx" | \
                      grep -v "//" | \
                      grep -v "import" | \
                      sed 's/.*fetch(//' | \
                      sed 's/.*axios.//' | \
                      sed 's/.*api\///' | \
                      sed 's/[^a-zA-Z0-9_/].*//' | \
                      sort | uniq)

    # Convert to JSON format
    local json_calls="[]"
    while IFS= read -r call; do
        if [[ -n "$call" ]]; then
            json_calls=$(echo "$json_calls" | jq --arg call "$call" '. + [{endpoint: $call, type: "frontend_call", source: "typescript"}]')
        fi
    done <<< "$api_calls"

    echo "$json_calls" > "$endpoints_file"
    log "SUCCESS" "Extracted TypeScript endpoints to $endpoints_file"
}

# Function to generate OpenAPI specification
generate_openapi_spec() {
    log "INFO" "Generating OpenAPI specification..."

    local spec_file="$API_DOCS_DIR/openapi.json"

    # Create basic OpenAPI structure
    local openapi_spec='{
        "openapi": "3.0.3",
        "info": {
            "title": "AI Orchestrator Hub API",
            "description": "REST API for the Multiagent Hive System",
            "version": "0.1.0-alpha.1",
            "contact": {
                "name": "AI Orchestrator Hub Team",
                "url": "https://github.com/your-org/ai-orchestrator-hub"
            }
        },
        "servers": [
            {
                "url": "http://localhost:3001",
                "description": "Development server"
            }
        ],
        "paths": {},
        "components": {
            "schemas": {},
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        },
        "security": [
            {
                "bearerAuth": []
            }
        ]
    }'

    # Add common API paths
    openapi_spec=$(echo "$openapi_spec" | jq '.paths."/api/agents" = {
        "get": {
            "summary": "List all agents",
            "description": "Retrieve a list of all agents in the hive",
            "responses": {
                "200": {
                    "description": "Successful response",
                    "content": {
                        "application/json": {
                            "schema": {
                                "type": "object",
                                "properties": {
                                    "agents": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/Agent"}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "post": {
            "summary": "Create a new agent",
            "description": "Create a new agent with specified configuration",
            "requestBody": {
                "required": true,
                "content": {
                    "application/json": {
                        "schema": {"$ref": "#/components/schemas/AgentConfig"}
                    }
                }
            },
            "responses": {
                "201": {
                    "description": "Agent created successfully",
                    "content": {
                        "application/json": {
                            "schema": {
                                "type": "object",
                                "properties": {
                                    "id": {"type": "string", "format": "uuid"}
                                }
                            }
                        }
                    }
                }
            }
        }
    }')

    # Add more API paths
    openapi_spec=$(echo "$openapi_spec" | jq '.paths."/api/tasks" = {
        "get": {
            "summary": "List all tasks",
            "description": "Retrieve a list of all tasks in the system",
            "responses": {
                "200": {
                    "description": "Successful response"
                }
            }
        },
        "post": {
            "summary": "Create a new task",
            "description": "Create a new task with specified configuration",
            "responses": {
                "201": {
                    "description": "Task created successfully"
                }
            }
        }
    }')

    openapi_spec=$(echo "$openapi_spec" | jq '.paths."/api/hive/status" = {
        "get": {
            "summary": "Get hive status",
            "description": "Retrieve current status and metrics of the hive",
            "responses": {
                "200": {
                    "description": "Successful response"
                }
            }
        }
    }')

    # Add schema definitions
    openapi_spec=$(echo "$openapi_spec" | jq '.components.schemas.Agent = {
        "type": "object",
        "properties": {
            "id": {"type": "string", "format": "uuid"},
            "name": {"type": "string"},
            "type": {"type": "string", "enum": ["worker", "coordinator", "learner", "specialist"]},
            "state": {"type": "string", "enum": ["idle", "working", "failed", "inactive"]},
            "capabilities": {
                "type": "array",
                "items": {"$ref": "#/components/schemas/Capability"}
            },
            "energy": {"type": "number", "format": "float"},
            "created_at": {"type": "string", "format": "date-time"}
        }
    }')

    openapi_spec=$(echo "$openapi_spec" | jq '.components.schemas.Capability = {
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "proficiency": {"type": "number", "format": "float", "minimum": 0.0, "maximum": 1.0},
            "learning_rate": {"type": "number", "format": "float"}
        }
    }')

    openapi_spec=$(echo "$openapi_spec" | jq '.components.schemas.AgentConfig = {
        "type": "object",
        "required": ["name", "type"],
        "properties": {
            "name": {"type": "string"},
            "type": {"type": "string"},
            "capabilities": {
                "type": "array",
                "items": {"$ref": "#/components/schemas/Capability"}
            }
        }
    }')

    echo "$openapi_spec" > "$spec_file"
    log "SUCCESS" "Generated OpenAPI specification at $spec_file"
}

# Function to generate API documentation in Markdown
generate_markdown_docs() {
    log "INFO" "Generating Markdown API documentation..."

    local md_file="$API_DOCS_DIR/api-reference.md"

    cat > "$md_file" << 'EOF'
# AI Orchestrator Hub API Reference

## Overview

The AI Orchestrator Hub provides a comprehensive REST API for managing multiagent systems, tasks, and monitoring system health.

## Base URL
```
http://localhost:3001
```

## Authentication

The API uses Bearer token authentication. Include the token in the Authorization header:

```
Authorization: Bearer <your-jwt-token>
```

## Endpoints

### Agents

#### GET /api/agents

Retrieve a list of all agents in the hive.

**Response:**
```json
{
  "agents": [
    {
      "id": "uuid",
      "name": "string",
      "type": "worker|coordinator|learner|specialist",
      "state": "idle|working|failed|inactive",
      "capabilities": [
        {
          "name": "string",
          "proficiency": 0.0-1.0,
          "learning_rate": 0.0-1.0
        }
      ],
      "energy": 0.0-100.0,
      "created_at": "ISO 8601 datetime"
    }
  ]
}
```

#### POST /api/agents

Create a new agent with specified configuration.

**Request Body:**
```json
{
  "name": "string",
  "type": "worker|coordinator|learner|specialist",
  "capabilities": [
    {
      "name": "string",
      "proficiency": 0.0-1.0,
      "learning_rate": 0.0-1.0
    }
  ]
}
```

**Response:**
```json
{
  "id": "uuid"
}
```

### Tasks

#### GET /api/tasks

Retrieve a list of all tasks in the system.

**Response:**
```json
{
  "tasks": [
    {
      "id": "uuid",
      "title": "string",
      "description": "string",
      "type": "string",
      "priority": "low|medium|high|critical",
      "status": "pending|assigned|completed|failed",
      "required_capabilities": [
        {
          "name": "string",
          "min_proficiency": 0.0-1.0
        }
      ],
      "created_at": "ISO 8601 datetime"
    }
  ]
}
```

#### POST /api/tasks

Create a new task with specified configuration.

**Request Body:**
```json
{
  "description": "string",
  "type": "string",
  "priority": "low|medium|high|critical",
  "required_capabilities": [
    {
      "name": "string",
      "min_proficiency": 0.0-1.0
    }
  ]
}
```

**Response:**
```json
{
  "id": "uuid"
}
```

### Hive Status

#### GET /api/hive/status

Retrieve current status and metrics of the hive.

**Response:**
```json
{
  "hive_id": "uuid",
  "created_at": "ISO 8601 datetime",
  "last_update": "ISO 8601 datetime",
  "metrics": {
    "total_agents": 0,
    "active_agents": 0,
    "completed_tasks": 0,
    "failed_tasks": 0,
    "average_performance": 0.0,
    "swarm_cohesion": 0.0,
    "learning_progress": 0.0
  },
  "swarm_center": [0.0, 0.0],
  "total_energy": 0.0
}
```

### Resource Information

#### GET /api/resources

Retrieve current resource utilization and system health information.

**Response:**
```json
{
  "system_resources": {
    "cpu_usage": 0.0,
    "memory_usage": 0.0,
    "available_memory": 0,
    "cpu_cores": 0
  },
  "resource_profile": {
    "profile_name": "string",
    "max_agents": 0,
    "neural_complexity": 0.0
  }
}
```

## Error Responses

All endpoints may return the following error responses:

### 400 Bad Request
```json
{
  "error": "Invalid request parameters",
  "details": "Specific validation error message"
}
```

### 401 Unauthorized
```json
{
  "error": "Authentication required",
  "details": "Valid JWT token required"
}
```

### 403 Forbidden
```json
{
  "error": "Access denied",
  "details": "Insufficient permissions"
}
```

### 404 Not Found
```json
{
  "error": "Resource not found",
  "details": "The requested resource does not exist"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal server error",
  "details": "An unexpected error occurred"
}
```

## Rate Limiting

The API implements rate limiting to prevent abuse:

- **Authenticated requests**: 1000 requests per minute
- **Unauthenticated requests**: 100 requests per minute

Rate limit headers are included in all responses:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1638360000
```

## WebSocket API

The system also provides real-time communication via WebSocket:

### Connection
```
ws://localhost:3001/ws
```

### Message Format
```json
{
  "type": "message_type",
  "data": {},
  "timestamp": "ISO 8601 datetime"
}
```

### Supported Message Types

- `agent_status_update`: Agent status changes
- `task_completed`: Task completion notifications
- `system_metrics`: Real-time system metrics
- `error_notification`: Error alerts

## SDKs and Libraries

### JavaScript/TypeScript Client

```javascript
import { HiveClient } from '@ai-orchestrator-hub/client'

const client = new HiveClient({
  baseURL: 'http://localhost:3001',
  token: 'your-jwt-token'
})

// List agents
const agents = await client.agents.list()

// Create task
const task = await client.tasks.create({
  description: 'Process data',
  type: 'data_processing',
  priority: 'high'
})
```

### Python Client

```python
from ai_orchestrator_hub import HiveClient

client = HiveClient(
    base_url='http://localhost:3001',
    token='your-jwt-token'
)

# List agents
agents = client.agents.list()

# Create task
task = client.tasks.create({
    'description': 'Process data',
    'type': 'data_processing',
    'priority': 'high'
})
```

## Changelog

### Version 0.1.0
- Initial API release
- Basic agent and task management
- System monitoring endpoints
- WebSocket real-time communication

---

*This documentation is automatically generated. Last updated: $(date)*
EOF

    log "SUCCESS" "Generated Markdown API documentation at $md_file"
}

# Function to generate architecture diagrams
generate_architecture_diagrams() {
    log "INFO" "Generating architecture diagrams..."

    local diagram_dir="$API_DOCS_DIR/diagrams"
    mkdir -p "$diagram_dir"

    # Generate system architecture diagram in Mermaid format
    local system_diagram="$diagram_dir/system-architecture.md"

    cat > "$system_diagram" << 'EOF'
# System Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        Web[Web Frontend<br/>React/Next.js]
        Mobile[Mobile App<br/>React Native]
        CLI[CLI Tools<br/>Rust]
    end

    subgraph "API Gateway"
        Gateway[API Gateway<br/>Axum/Rust]
        Auth[Authentication<br/>JWT/OAuth]
        RateLimit[Rate Limiting<br/>Token Bucket]
    end

    subgraph "Core Services"
        Hive[Hive Coordinator<br/>Swarm Intelligence]
        TaskMgr[Task Manager<br/>Work Stealing Queue]
        AgentMgr[Agent Manager<br/>Dynamic Scaling]
        ResourceMgr[Resource Manager<br/>Auto-optimization]
    end

    subgraph "AI/ML Layer"
        Neural[Neural Processor<br/>Hybrid FANN/LSTM]
        NLP[NLP Engine<br/>Text Processing]
        Learning[Learning System<br/>Adaptive Algorithms]
    end

    subgraph "Data Layer"
        Cache[Multi-level Cache<br/>L1/L2/Performance]
        DB[(SQLite/PostgreSQL<br/>System State)]
        Persistence[Persistence Manager<br/>Backup/Recovery]
    end

    subgraph "Infrastructure"
        Monitoring[Monitoring System<br/>Metrics/Alerts]
        Logging[Logging System<br/>Structured Logs]
        Health[Health Checks<br/>System Diagnostics]
    end

    Web --> Gateway
    Mobile --> Gateway
    CLI --> Gateway

    Gateway --> Auth
    Gateway --> RateLimit

    Auth --> Hive
    RateLimit --> Hive

    Hive --> TaskMgr
    Hive --> AgentMgr
    Hive --> ResourceMgr

    TaskMgr --> Neural
    AgentMgr --> Neural
    Neural --> NLP
    Neural --> Learning

    Hive --> Cache
    TaskMgr --> Cache
    AgentMgr --> Cache

    Cache --> DB
    DB --> Persistence

    Hive --> Monitoring
    TaskMgr --> Monitoring
    AgentMgr --> Monitoring
    ResourceMgr --> Monitoring

    Monitoring --> Logging
    Monitoring --> Health

    classDef frontend fill:#e1f5fe
    classDef gateway fill:#f3e5f5
    classDef core fill:#e8f5e8
    classDef ai fill:#fff3e0
    classDef data fill:#fce4ec
    classDef infra fill:#f1f8e9

    class Web,Mobile,CLI frontend
    class Gateway,Auth,RateLimit gateway
    class Hive,TaskMgr,AgentMgr,ResourceMgr core
    class Neural,NLP,Learning ai
    class Cache,DB,Persistence data
    class Monitoring,Logging,Health infra
```

## Component Interactions

```mermaid
sequenceDiagram
    participant Client
    participant Gateway
    participant Hive
    participant TaskMgr
    participant AgentMgr
    participant Neural
    participant Cache
    participant DB

    Client->>Gateway: API Request
    Gateway->>Gateway: Authenticate & Rate Limit
    Gateway->>Hive: Process Request

    alt Agent Operation
        Hive->>AgentMgr: Get/Create Agent
        AgentMgr->>Cache: Check Cache
        Cache-->>AgentMgr: Cache Hit/Miss
        AgentMgr->>DB: Persist Agent
    end

    alt Task Operation
        Hive->>TaskMgr: Create/Assign Task
        TaskMgr->>Neural: Process Task Requirements
        Neural-->>TaskMgr: Task Analysis
        TaskMgr->>AgentMgr: Find Suitable Agent
        AgentMgr-->>TaskMgr: Agent Assignment
    end

    Hive-->>Gateway: Response
    Gateway-->>Client: API Response
```

## Data Flow Architecture

```mermaid
flowchart TD
    A[User Request] --> B{Authentication}
    B -->|Valid| C[Rate Limiting]
    B -->|Invalid| D[401 Unauthorized]

    C --> E{Request Type}
    E -->|Agent| F[Agent Manager]
    E -->|Task| G[Task Manager]
    E -->|Status| H[Hive Status]
    E -->|Resources| I[Resource Monitor]

    F --> J[Cache Layer]
    G --> J
    H --> J
    I --> J

    J --> K[(Database)]
    J --> L[Neural Processing]

    K --> M[Response Cache]
    L --> M

    M --> N[API Response]
    N --> O[WebSocket Push]
    N --> P[Logging]
```

---

*Generated automatically on: $(date)*
EOF

    log "SUCCESS" "Generated architecture diagrams in $diagram_dir"
}

# Function to validate documentation
validate_documentation() {
    log "INFO" "Validating generated documentation..."

    local errors=0

    # Check if all required files exist
    local required_files=(
        "$API_DOCS_DIR/openapi.json"
        "$API_DOCS_DIR/api-reference.md"
        "$API_DOCS_DIR/diagrams/system-architecture.md"
    )

    for file in "${required_files[@]}"; do
        if [[ ! -f "$file" ]]; then
            log "ERROR" "Required documentation file missing: $file"
            ((errors++))
        fi
    done

    # Validate OpenAPI spec
    if [[ -f "$API_DOCS_DIR/openapi.json" ]]; then
        if ! jq empty "$API_DOCS_DIR/openapi.json" 2>/dev/null; then
            log "ERROR" "OpenAPI specification is not valid JSON"
            ((errors++))
        else
            log "SUCCESS" "OpenAPI specification is valid JSON"
        fi
    fi

    # Check for broken links in Markdown
    if command -v markdown-link-check &> /dev/null; then
        if ! markdown-link-check "$API_DOCS_DIR/api-reference.md" --quiet; then
            log "WARN" "Some links in API documentation may be broken"
        else
            log "SUCCESS" "All links in API documentation are valid"
        fi
    else
        log "INFO" "markdown-link-check not available, skipping link validation"
    fi

    if [[ $errors -eq 0 ]]; then
        log "SUCCESS" "Documentation validation passed"
    else
        log "ERROR" "Documentation validation failed with $errors errors"
        return 1
    fi
}

# Function to generate documentation index
generate_documentation_index() {
    log "INFO" "Generating documentation index..."

    local index_file="$DOCS_DIR/README.md"

    cat > "$index_file" << EOF
# AI Orchestrator Hub Documentation

Welcome to the AI Orchestrator Hub documentation. This comprehensive guide covers all aspects of the multiagent system.

## 📚 Documentation Overview

### 🏗️ Architecture & Design
- **[System Architecture](api/diagrams/system-architecture.md)** - High-level system design and component interactions
- **[API Reference](api/api-reference.md)** - Complete REST API documentation
- **[OpenAPI Specification](api/openapi.json)** - Machine-readable API specification

### 🔧 Development
- **[Setup Guide](../README.md)** - Getting started with development
- **[Contributing Guidelines](../CONTRIBUTING.md)** - How to contribute to the project
- **[API Documentation](../docs/api/)** - Detailed API documentation

### 🧪 Testing & Quality
- **[Testing Guide](../TESTING.md)** - Testing strategies and guidelines
- **[Performance Benchmarks](../backend/benches/)** - Performance testing results
- **[Security Policy](../.github/SECURITY.MD)** - Security guidelines and procedures

### 📊 Monitoring & Operations
- **[Monitoring Setup](../monitoring/)** - System monitoring and alerting
- **[Deployment Guide](../scripts/)** - Deployment and operational scripts
- **[Troubleshooting](../docs/troubleshooting.md)** - Common issues and solutions

## 🚀 Quick Start

1. **Clone the repository**
   \`\`\`bash
   git clone https://github.com/your-org/ai-orchestrator-hub.git
   cd ai-orchestrator-hub
   \`\`\`

2. **Start the development environment**
   \`\`\`bash
   # Backend
   cd backend && cargo run

   # Frontend (in another terminal)
   cd frontend && npm run dev
   \`\`\`

3. **Access the application**
   - Frontend: http://localhost:3000
   - API: http://localhost:3001
   - API Docs: http://localhost:3001/docs

## 📖 API Documentation

The API documentation is automatically generated and includes:

- **REST Endpoints** - Complete endpoint reference with examples
- **WebSocket API** - Real-time communication protocols
- **Authentication** - JWT-based authentication system
- **Rate Limiting** - API rate limiting and quotas
- **Error Handling** - Comprehensive error response formats

### Key Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| \`/api/agents\` | GET | List all agents |
| \`/api/agents\` | POST | Create new agent |
| \`/api/tasks\` | GET | List all tasks |
| \`/api/tasks\` | POST | Create new task |
| \`/api/hive/status\` | GET | Get system status |
| \`/api/resources\` | GET | Get resource metrics |

## 🏛️ System Architecture

The AI Orchestrator Hub is built with a modular, scalable architecture:

### Core Components

- **Hive Coordinator** - Central orchestration engine
- **Agent Manager** - Dynamic agent lifecycle management
- **Task Manager** - Work-stealing task distribution
- **Neural Processor** - AI/ML processing engine
- **Resource Manager** - System resource optimization
- **Persistence Layer** - Data storage and recovery

### Technology Stack

- **Backend**: Rust with Axum web framework
- **Frontend**: React/Next.js with TypeScript
- **Database**: SQLite with connection pooling
- **Cache**: Multi-level caching system
- **Real-time**: WebSocket communication
- **AI/ML**: Custom neural networks (FANN/LSTM)

## 🔒 Security

The system implements comprehensive security measures:

- JWT-based authentication
- Rate limiting and DDoS protection
- Input validation and sanitization
- Secure communication (HTTPS/WSS)
- Audit logging and monitoring
- Regular security updates

## 📈 Performance

Optimized for high-performance multiagent operations:

- **Connection Pooling** - Efficient database connections
- **Multi-level Caching** - Fast data access
- **Async Processing** - Non-blocking operations
- **Resource Optimization** - Automatic scaling
- **Work-stealing Queues** - Optimal task distribution

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](../CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Code Quality

- **Linting**: ESLint for frontend, Clippy for backend
- **Testing**: Comprehensive test suite with CI/CD
- **Documentation**: Auto-generated API docs
- **Security**: Automated security scanning

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/your-org/ai-orchestrator-hub/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/ai-orchestrator-hub/discussions)
- **Documentation**: [Wiki](https://github.com/your-org/ai-orchestrator-hub/wiki)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

---

*Documentation automatically generated on: $(date)*
*Last updated: $(date +%Y-%m-%d\ %H:%M:%S\ %Z)*
EOF

    log "SUCCESS" "Generated documentation index at $index_file"
}

# Function to run documentation tests
run_documentation_tests() {
    log "INFO" "Running documentation tests..."

    local test_results="$API_DOCS_DIR/test-results.json"
    local errors=0

    # Test 1: Check if all endpoints are documented
    log "INFO" "Testing endpoint documentation coverage..."

    if [[ -f "$API_DOCS_DIR/rust_endpoints.json" ]] && [[ -f "$API_DOCS_DIR/typescript_endpoints.json" ]]; then
        local rust_count=$(jq '. | length' "$API_DOCS_DIR/rust_endpoints.json")
        local ts_count=$(jq '. | length' "$API_DOCS_DIR/typescript_endpoints.json")

        if [[ $rust_count -gt 0 ]] && [[ $ts_count -gt 0 ]]; then
            log "SUCCESS" "Found $rust_count Rust endpoints and $ts_count TypeScript calls"
        else
            log "WARN" "Low endpoint coverage detected"
            ((errors++))
        fi
    else
        log "ERROR" "Endpoint documentation files missing"
        ((errors++))
    fi

    # Test 2: Validate OpenAPI specification
    log "INFO" "Testing OpenAPI specification..."

    if [[ -f "$API_DOCS_DIR/openapi.json" ]]; then
        if jq -e '.openapi and .info and .paths' "$API_DOCS_DIR/openapi.json" > /dev/null 2>&1; then
            log "SUCCESS" "OpenAPI specification is valid"
        else
            log "ERROR" "OpenAPI specification is invalid"
            ((errors++))
        fi
    else
        log "ERROR" "OpenAPI specification file missing"
        ((errors++))
    fi

    # Test 3: Check documentation completeness
    log "INFO" "Testing documentation completeness..."

    if [[ -f "$API_DOCS_DIR/api-reference.md" ]]; then
        local line_count=$(wc -l < "$API_DOCS_DIR/api-reference.md")
        if [[ $line_count -gt 50 ]]; then
            log "SUCCESS" "API documentation appears comprehensive ($line_count lines)"
        else
            log "WARN" "API documentation may be incomplete ($line_count lines)"
            ((errors++))
        fi
    else
        log "ERROR" "API reference documentation missing"
        ((errors++))
    fi

    # Save test results
    local test_result_json=$(jq -n \
        --arg timestamp "$(date -Iseconds)" \
        --arg errors "$errors" \
        --arg rust_endpoints "$(jq '. | length' "$API_DOCS_DIR/rust_endpoints.json" 2>/dev/null || echo 0)" \
        --arg ts_endpoints "$(jq '. | length' "$API_DOCS_DIR/typescript_endpoints.json" 2>/dev/null || echo 0)" \
        '{
            timestamp: $timestamp,
            tests_passed: ($errors == "0"),
            errors: $errors | tonumber,
            coverage: {
                rust_endpoints: $rust_endpoints | tonumber,
                typescript_endpoints: $ts_endpoints | tonumber
            }
        }')

    echo "$test_result_json" > "$test_results"

    if [[ $errors -eq 0 ]]; then
        log "SUCCESS" "All documentation tests passed"
    else
        log "ERROR" "Documentation tests failed with $errors errors"
        return 1
    fi
}

# Main function
main() {
    log "INFO" "Starting API documentation generation..."

    # Create documentation directory
    mkdir -p "$API_DOCS_DIR"

    # Run all documentation generation steps
    check_prerequisites
    extract_rust_endpoints
    extract_typescript_endpoints
    generate_openapi_spec
    generate_markdown_docs
    generate_architecture_diagrams
    generate_documentation_index

    # Validate and test documentation
    if validate_documentation && run_documentation_tests; then
        log "SUCCESS" "API documentation generation completed successfully"
        log "INFO" "Documentation available at: $API_DOCS_DIR"
        log "INFO" "Main documentation index: $DOCS_DIR/README.md"
    else
        log "ERROR" "Documentation generation failed"
        exit 1
    fi
}

# Run main function
main "$@"</content>
</xai:function_call">scripts/generate-api-docs.sh
