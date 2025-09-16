# API Reference

This section provides comprehensive documentation for the AI Orchestrator Hub REST API, WebSocket events, and MCP protocol integration.

## Overview

The AI Orchestrator Hub provides a RESTful API for managing agents, tasks, and monitoring system health. All endpoints return standardized JSON responses with consistent error handling.

### Base URL
```
http://localhost:3001
```

### Authentication
Currently, the API does not require authentication. Rate limiting is applied to prevent abuse.

### Response Format
All API responses follow this standardized format:

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "timestamp": "2024-01-01T00:00:00Z",
  "request_id": "uuid-v4-optional"
}
```

### Error Response Format
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": { ... },
    "field_errors": [
      {
        "field": "field_name",
        "message": "Field-specific error",
        "value": "invalid_value"
      }
    ]
  },
  "timestamp": "2024-01-01T00:00:00Z",
  "request_id": "uuid-v4-optional"
}
```

## API Sections

### Core Endpoints
- [System Health](health.md) - Health checks and system status
- [Metrics](metrics.md) - Performance metrics and monitoring

### Agent Management
- [Agent Operations](agents.md) - Create, read, update, delete agents
- [Agent Lifecycle](agent-lifecycle.md) - Agent states and transitions

### Task Management
- [Task Operations](tasks.md) - Create and manage tasks
- [Task Execution](task-execution.md) - Task lifecycle and execution

### Hive Operations
- [Hive Status](hive-status.md) - Overall system status
- [Resource Management](resources.md) - System resource monitoring

### Real-time Communication
- [WebSocket Events](websocket.md) - Real-time event streaming
- [MCP Protocol](mcp.md) - Model Context Protocol integration

## Rate Limiting

The API implements rate limiting to prevent abuse:

- **Agent creation**: 10 requests per minute
- **Task creation**: 20 requests per minute
- **Status endpoints**: 60 requests per minute

When rate limited, you'll receive:

```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded",
    "details": {
      "retry_after_seconds": 60
    }
  }
}
```

## Error Codes

### Common Error Codes
- `VALIDATION_ERROR`: Input validation failed
- `RATE_LIMIT_EXCEEDED`: Too many requests
- `INTERNAL_ERROR`: Unexpected server error
- `SERVICE_UNAVAILABLE`: Service temporarily unavailable

### Agent-Specific Errors
- `AGENT_NOT_FOUND`: Specified agent doesn't exist
- `AGENT_CREATION_FAILED`: Failed to create agent
- `AGENT_LIMIT_EXCEEDED`: Maximum number of agents reached

### Task-Specific Errors
- `TASK_NOT_FOUND`: Specified task doesn't exist
- `TASK_CREATION_FAILED`: Failed to create task
- `TASK_ASSIGNMENT_FAILED`: Could not assign task to agent

## SDKs and Libraries

### Official SDKs
- **Rust SDK**: Native Rust client library
- **TypeScript SDK**: TypeScript/JavaScript client
- **Python SDK**: Python client library

### Community Libraries
- **Go Client**: Community-maintained Go client
- **Java Client**: Community-maintained Java client

## Examples

### Basic Agent Creation
```bash
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "DataProcessor-1",
    "type": "worker",
    "capabilities": [
      {
        "name": "data_processing",
        "proficiency": 0.8,
        "learning_rate": 0.1
      }
    ]
  }'
```

### Task Creation
```bash
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Process customer data",
    "type": "data_analysis",
    "priority": 2,
    "required_capabilities": [
      {
        "name": "data_processing",
        "min_proficiency": 0.7
      }
    ]
  }'
```

### WebSocket Connection
```javascript
const ws = new WebSocket('ws://localhost:3001/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Received:', data);
};

ws.onopen = () => {
  console.log('Connected to AI Orchestrator Hub');
};
```

## Versioning

The API follows semantic versioning:

- **v1.0**: Initial release with basic agent and task management
- **v2.0**: Enhanced with neural processing and advanced monitoring
- **v2.1**: Modular architecture with improved performance

## Support

For API support or questions:

- **Health Checks**: Use `/health` endpoint for system diagnostics
- **Interactive Documentation**: Visit `/docs` endpoint when available
- **GitHub Issues**: Report bugs and request features
- **Community**: Join discussions and get help from the community

See the individual API section documents for detailed endpoint specifications and examples.