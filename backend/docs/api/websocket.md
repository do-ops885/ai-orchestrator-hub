# WebSocket API

The WebSocket API provides real-time communication for monitoring system events, task progress, and agent activities.

## Connection

### Establishing Connection

Connect to the WebSocket endpoint for real-time updates:

```javascript
const ws = new WebSocket('ws://localhost:3001/ws');
```

```python
import websocket

ws = websocket.WebSocket()
ws.connect('ws://localhost:3001/ws')
```

### Connection Parameters

- **URL**: `ws://localhost:3001/ws`
- **Protocol**: Standard WebSocket
- **Authentication**: None required (can be added via query parameters)
- **Heartbeat**: Automatic ping/pong every 30 seconds

### Connection Lifecycle

```javascript
ws.onopen = () => {
  console.log('Connected to AI Orchestrator Hub');
};

ws.onclose = (event) => {
  console.log('Connection closed:', event.code, event.reason);
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};
```

## Message Format

All WebSocket messages follow a standardized format:

```json
{
  "type": "message_type",
  "data": { ... },
  "timestamp": "2024-01-01T00:00:00Z",
  "message_id": "uuid-optional"
}
```

### Message Fields

- `type`: Message type identifier
- `data`: Message-specific payload
- `timestamp`: Server timestamp
- `message_id`: Optional unique message identifier

## Agent Events

### agent_created

Sent when a new agent is created in the system.

```json
{
  "type": "agent_created",
  "data": {
    "agent": {
      "id": "uuid-agent-123",
      "name": "DataProcessor-1",
      "type": "specialist",
      "capabilities": [
        {
          "name": "data_processing",
          "proficiency": 0.85,
          "learning_rate": 0.1
        }
      ],
      "created_at": "2024-01-01T00:00:00Z"
    }
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### agent_status_changed

Sent when an agent's status changes.

```json
{
  "type": "agent_status_changed",
  "data": {
    "agent_id": "uuid-agent-123",
    "old_status": "idle",
    "new_status": "active",
    "reason": "task_assigned",
    "timestamp": "2024-01-01T00:05:00Z"
  },
  "timestamp": "2024-01-01T00:05:00Z"
}
```

### agent_performance_updated

Sent when an agent's performance metrics are updated.

```json
{
  "type": "agent_performance_updated",
  "data": {
    "agent_id": "uuid-agent-123",
    "metrics": {
      "tasks_completed": 25,
      "success_rate": 0.92,
      "average_response_time_ms": 1250,
      "capability_improvements": 3
    },
    "timestamp": "2024-01-01T00:10:00Z"
  },
  "timestamp": "2024-01-01T00:10:00Z"
}
```

### agent_removed

Sent when an agent is removed from the system.

```json
{
  "type": "agent_removed",
  "data": {
    "agent_id": "uuid-agent-123",
    "reason": "manual_removal",
    "cleanup_summary": {
      "tasks_reassigned": 2,
      "data_removed": true
    },
    "timestamp": "2024-01-01T01:00:00Z"
  },
  "timestamp": "2024-01-01T01:00:00Z"
}
```

## Task Events

### task_created

Sent when a new task is created.

```json
{
  "type": "task_created",
  "data": {
    "task": {
      "id": "uuid-task-456",
      "description": "Process customer data",
      "type": "data_processing",
      "priority": 2,
      "status": "pending",
      "required_capabilities": [
        {
          "name": "data_processing",
          "min_proficiency": 0.7
        }
      ],
      "created_at": "2024-01-01T00:00:00Z"
    },
    "matching_agents": 3
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### task_assigned

Sent when a task is assigned to an agent.

```json
{
  "type": "task_assigned",
  "data": {
    "task_id": "uuid-task-456",
    "agent_id": "uuid-agent-123",
    "assignment_reason": "capability_match",
    "estimated_completion": "2024-01-01T00:30:00Z",
    "timestamp": "2024-01-01T00:01:00Z"
  },
  "timestamp": "2024-01-01T00:01:00Z"
}
```

### task_started

Sent when a task begins execution.

```json
{
  "type": "task_started",
  "data": {
    "task_id": "uuid-task-456",
    "agent_id": "uuid-agent-123",
    "attempt_number": 1,
    "timestamp": "2024-01-01T00:01:30Z"
  },
  "timestamp": "2024-01-01T00:01:30Z"
}
```

### task_progress

Sent periodically during task execution to report progress.

```json
{
  "type": "task_progress",
  "data": {
    "task_id": "uuid-task-456",
    "agent_id": "uuid-agent-123",
    "progress": 0.75,
    "current_step": "Processing data batch 3/4",
    "estimated_completion": "2024-01-01T00:25:00Z",
    "timestamp": "2024-01-01T00:20:00Z"
  },
  "timestamp": "2024-01-01T00:20:00Z"
}
```

### task_completed

Sent when a task completes successfully.

```json
{
  "type": "task_completed",
  "data": {
    "task_id": "uuid-task-456",
    "agent_id": "uuid-agent-123",
    "execution_time_ms": 240000,
    "result": {
      "records_processed": 10000,
      "success_rate": 0.98,
      "output_file": "processed_data.json"
    },
    "timestamp": "2024-01-01T00:25:00Z"
  },
  "timestamp": "2024-01-01T00:25:00Z"
}
```

### task_failed

Sent when a task fails.

```json
{
  "type": "task_failed",
  "data": {
    "task_id": "uuid-task-456",
    "agent_id": "uuid-agent-123",
    "attempt_number": 1,
    "error": {
      "code": "DATA_PROCESSING_ERROR",
      "message": "Failed to process data: invalid format",
      "details": {
        "error_line": 150,
        "data_sample": "invalid_data_format"
      }
    },
    "retry_scheduled": true,
    "next_retry_at": "2024-01-01T00:35:00Z",
    "timestamp": "2024-01-01T00:25:00Z"
  },
  "timestamp": "2024-01-01T00:25:00Z"
}
```

### task_cancelled

Sent when a task is cancelled.

```json
{
  "type": "task_cancelled",
  "data": {
    "task_id": "uuid-task-456",
    "reason": "manual_cancellation",
    "cancelled_by": "user-123",
    "timestamp": "2024-01-01T00:15:00Z"
  },
  "timestamp": "2024-01-01T00:15:00Z"
}
```

## System Events

### hive_status_update

Sent periodically with overall system status.

```json
{
  "type": "hive_status_update",
  "data": {
    "status": "healthy",
    "metrics": {
      "total_agents": 5,
      "active_agents": 3,
      "pending_tasks": 8,
      "completed_tasks_today": 142,
      "system_load": 0.65
    },
    "alerts": [
      {
        "level": "warning",
        "message": "High memory usage detected",
        "timestamp": "2024-01-01T00:20:00Z"
      }
    ],
    "timestamp": "2024-01-01T00:30:00Z"
  },
  "timestamp": "2024-01-01T00:30:00Z"
}
```

### system_alert

Sent when system alerts are triggered.

```json
{
  "type": "system_alert",
  "data": {
    "level": "critical",
    "title": "Database Connection Lost",
    "message": "Unable to connect to primary database",
    "details": {
      "error": "connection_timeout",
      "affected_components": ["task_management", "agent_management"]
    },
    "timestamp": "2024-01-01T00:45:00Z"
  },
  "timestamp": "2024-01-01T00:45:00Z"
}
```

### resource_alert

Sent when resource usage exceeds thresholds.

```json
{
  "type": "resource_alert",
  "data": {
    "resource": "cpu",
    "usage_percent": 85.2,
    "threshold_percent": 80.0,
    "trend": "increasing",
    "recommendations": [
      "Consider scaling up agent pool",
      "Review task priorities"
    ],
    "timestamp": "2024-01-01T00:50:00Z"
  },
  "timestamp": "2024-01-01T00:50:00Z"
}
```

## Modular System Events

### module_status_changed

Sent when a module's status changes.

```json
{
  "type": "module_status_changed",
  "data": {
    "module": "background_processes",
    "old_status": "healthy",
    "new_status": "degraded",
    "reason": "High CPU usage in learning process",
    "affected_functions": ["learning_cycle", "swarm_coordination"],
    "timestamp": "2024-01-01T01:00:00Z"
  },
  "timestamp": "2024-01-01T01:00:00Z"
}
```

### inter_module_message

Sent when modules communicate with each other.

```json
{
  "type": "inter_module_message",
  "data": {
    "from_module": "task_management",
    "to_module": "metrics_collection",
    "message_type": "TaskCompleted",
    "payload": {
      "task_id": "uuid-task-456",
      "execution_time_ms": 240000
    },
    "timestamp": "2024-01-01T00:25:00Z"
  },
  "timestamp": "2024-01-01T00:25:00Z"
}
```

### coordination_event

Sent for swarm coordination events.

```json
{
  "type": "coordination_event",
  "data": {
    "event_type": "SwarmOptimization",
    "module": "coordinator",
    "details": {
      "optimization_type": "load_balancing",
      "agents_rebalanced": 3,
      "efficiency_improvement": 0.15
    },
    "timestamp": "2024-01-01T00:30:00Z"
  },
  "timestamp": "2024-01-01T00:30:00Z"
}
```

## Learning Events

### learning_cycle_started

Sent when a learning cycle begins.

```json
{
  "type": "learning_cycle_started",
  "data": {
    "cycle_id": "learn-uuid-789",
    "agent_id": "uuid-agent-123",
    "learning_type": "capability_improvement",
    "target_capability": "data_processing",
    "estimated_duration_minutes": 15,
    "timestamp": "2024-01-01T02:00:00Z"
  },
  "timestamp": "2024-01-01T02:00:00Z"
}
```

### learning_cycle_completed

Sent when a learning cycle completes.

```json
{
  "type": "learning_cycle_completed",
  "data": {
    "cycle_id": "learn-uuid-789",
    "agent_id": "uuid-agent-123",
    "improvements": {
      "data_processing_proficiency": {
        "old_value": 0.85,
        "new_value": 0.88,
        "improvement": 0.03
      }
    },
    "duration_ms": 900000,
    "timestamp": "2024-01-01T02:15:00Z"
  },
  "timestamp": "2024-01-01T02:15:00Z"
}
```

## Error Events

### error_occurred

Sent when errors occur in the system.

```json
{
  "type": "error_occurred",
  "data": {
    "error_id": "err-uuid-101",
    "component": "task_executor",
    "error": {
      "code": "EXECUTION_FAILED",
      "message": "Task execution failed due to timeout",
      "details": {
        "task_id": "uuid-task-456",
        "timeout_seconds": 300,
        "execution_time_seconds": 320
      }
    },
    "severity": "error",
    "timestamp": "2024-01-01T00:40:00Z"
  },
  "timestamp": "2024-01-01T00:40:00Z"
}
```

## Client Implementation Examples

### JavaScript Client

```javascript
class HiveWebSocketClient {
  constructor(url = 'ws://localhost:3001/ws') {
    this.url = url;
    this.ws = null;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
    this.messageHandlers = new Map();
    this.setupDefaultHandlers();
  }

  setupDefaultHandlers() {
    // Agent events
    this.messageHandlers.set('agent_created', (data) => {
      console.log('🆕 New agent:', data.agent.name);
    });

    this.messageHandlers.set('agent_status_changed', (data) => {
      console.log(`🔄 Agent ${data.agent_id}: ${data.old_status} → ${data.new_status}`);
    });

    // Task events
    this.messageHandlers.set('task_created', (data) => {
      console.log('📋 New task:', data.task.description);
    });

    this.messageHandlers.set('task_completed', (data) => {
      console.log('✅ Task completed:', data.task_id);
    });

    this.messageHandlers.set('task_failed', (data) => {
      console.error('❌ Task failed:', data.task_id, data.error.message);
    });

    // System events
    this.messageHandlers.set('system_alert', (data) => {
      console.warn('🚨 Alert:', data.title, data.message);
    });
  }

  connect() {
    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        console.log('✅ Connected to Hive WebSocket');
        this.reconnectAttempts = 0;
      };

      this.ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          this.handleMessage(message);
        } catch (error) {
          console.error('❌ Failed to parse message:', error);
        }
      };

      this.ws.onclose = (event) => {
        console.log('🔌 Connection closed:', event.code);
        this.attemptReconnect();
      };

      this.ws.onerror = (error) => {
        console.error('❌ WebSocket error:', error);
      };

    } catch (error) {
      console.error('❌ Failed to connect:', error);
    }
  }

  handleMessage(message) {
    const handler = this.messageHandlers.get(message.type);
    if (handler) {
      handler(message.data);
    } else {
      console.log('❓ Unknown message type:', message.type);
    }
  }

  attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);

      console.log(`🔄 Reconnecting in ${delay}ms...`);
      setTimeout(() => this.connect(), delay);
    } else {
      console.error('❌ Max reconnection attempts reached');
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
    }
  }

  // Add custom message handler
  on(eventType, handler) {
    this.messageHandlers.set(eventType, handler);
  }
}

// Usage
const client = new HiveWebSocketClient();
client.connect();

// Add custom handler
client.on('custom_event', (data) => {
  console.log('🎨 Custom event:', data);
});
```

### Python Client

```python
import websocket
import json
import threading
import time

class HiveWebSocketClient:
    def __init__(self, url='ws://localhost:3001/ws'):
        self.url = url
        self.ws = None
        self.running = False
        self.message_handlers = {}

    def connect(self):
        try:
            self.ws = websocket.WebSocket()
            self.ws.connect(self.url)
            self.running = True
            print("✅ Connected to Hive WebSocket")

            # Start message handling thread
            threading.Thread(target=self._message_loop, daemon=True).start()

        except Exception as e:
            print(f"❌ Failed to connect: {e}")

    def _message_loop(self):
        while self.running:
            try:
                message = self.ws.recv()
                if message:
                    self._handle_message(json.loads(message))
            except Exception as e:
                print(f"❌ Message handling error: {e}")
                break

    def _handle_message(self, message):
        message_type = message.get('type')
        handler = self.message_handlers.get(message_type)

        if handler:
            handler(message.get('data', {}))
        else:
            print(f"❓ Unknown message type: {message_type}")

    def on(self, event_type, handler):
        self.message_handlers[event_type] = handler

    def disconnect(self):
        self.running = False
        if self.ws:
            self.ws.close()

# Usage
client = HiveWebSocketClient()

# Set up event handlers
client.on('agent_created', lambda data: print(f"🆕 New agent: {data['agent']['name']}"))
client.on('task_completed', lambda data: print(f"✅ Task completed: {data['task_id']}"))
client.on('system_alert', lambda data: print(f"🚨 Alert: {data['title']}"))

client.connect()

# Keep running
try:
    while True:
        time.sleep(1)
except KeyboardInterrupt:
    client.disconnect()
```

## Best Practices

### Connection Management
1. **Implement reconnection logic** for network interruptions
2. **Handle connection timeouts** gracefully
3. **Monitor connection health** with periodic pings
4. **Clean up resources** on disconnection

### Message Handling
1. **Validate message format** before processing
2. **Handle unknown message types** gracefully
3. **Process messages asynchronously** to avoid blocking
4. **Log important events** for debugging

### Error Handling
1. **Handle WebSocket errors** appropriately
2. **Implement exponential backoff** for reconnections
3. **Log connection issues** for troubleshooting
4. **Gracefully degrade** when WebSocket is unavailable

### Performance Considerations
1. **Filter messages** based on client needs
2. **Batch updates** when possible
3. **Monitor message volume** to prevent overload
4. **Use compression** for large payloads

## Troubleshooting

### Connection Issues

**Problem**: Cannot connect to WebSocket
```bash
# Check if backend is running
curl http://localhost:3001/health

# Check firewall settings
sudo ufw status | grep 3001

# Test WebSocket connection
websocat ws://localhost:3001/ws
```

**Problem**: Connection drops frequently
```javascript
// Enable heartbeat monitoring
ws.on('ping', () => {
  console.log('Received ping');
});

ws.on('pong', () => {
  console.log('Received pong');
});
```

### Message Issues

**Problem**: Not receiving expected messages
```javascript
// Check message filters
const expectedTypes = ['agent_created', 'task_completed'];
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (expectedTypes.includes(message.type)) {
    console.log('Received:', message);
  }
};
```

**Problem**: Messages arriving out of order
```javascript
// Add sequence numbers
let sequenceNumber = 0;
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.sequence > sequenceNumber) {
    sequenceNumber = message.sequence;
    processMessage(message);
  }
};
```

### Performance Issues

**Problem**: Too many messages overwhelming client
```javascript
// Implement message throttling
let lastMessageTime = 0;
const throttleMs = 100;

ws.onmessage = (event) => {
  const now = Date.now();
  if (now - lastMessageTime > throttleMs) {
    lastMessageTime = now;
    processMessage(JSON.parse(event.data));
  }
};
```

**Problem**: Large payloads causing delays
```javascript
// Request compressed messages
const ws = new WebSocket('ws://localhost:3001/ws', [], {
  headers: {
    'Accept-Encoding': 'gzip, deflate'
  }
});
```

This WebSocket API provides comprehensive real-time monitoring and control capabilities for the AI Orchestrator Hub system.