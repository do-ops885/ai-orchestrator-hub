# Agent Communication Patterns

This document describes the standardized communication patterns implemented for the AI Orchestrator Hub's multiagent system.

## Overview

The communication system provides:
- **Standardized Message Formats**: Consistent message envelopes for all agent interactions
- **Async Communication Patterns**: Proper async/await patterns with timeout and cancellation
- **Error Handling**: Comprehensive error handling and recovery mechanisms
- **Performance Optimizations**: Resource management and performance monitoring
- **Protocol Versioning**: Backward-compatible protocol evolution

## Core Components

### 1. Message Envelope

All inter-agent communication uses the `MessageEnvelope` structure:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub id: Uuid,                    // Unique message identifier
    pub message_type: MessageType,   // Type of message
    pub sender_id: Uuid,            // Sending agent ID
    pub recipients: Vec<Uuid>,      // Target recipient(s)
    pub payload: MessagePayload,    // Message content
    pub priority: MessagePriority,  // Message priority level
    pub delivery_guarantee: DeliveryGuarantee, // Delivery requirements
    pub timestamp: DateTime<Utc>,   // Message creation time
    pub correlation_id: Option<Uuid>, // For request-response patterns
    pub ttl_seconds: Option<u64>,   // Time-to-live
    pub protocol_version: String,   // Protocol version
    pub metadata: HashMap<String, Value>, // Additional metadata
}
```

### 2. Message Types

Standardized message types for different communication scenarios:

- **Agent Lifecycle**: `AgentCreated`, `AgentDestroyed`, `AgentStatusUpdate`
- **Task Management**: `TaskAssigned`, `TaskCompleted`, `TaskFailed`, `TaskProgress`
- **Communication**: `Request`, `Response`, `Broadcast`, `Multicast`
- **Coordination**: `CoordinationRequest`, `CoordinationResponse`, `SwarmUpdate`
- **Error Handling**: `Error`, `Timeout`
- **Custom**: `Custom(String)` for application-specific messages

### 3. Message Priorities

Four priority levels for message processing:

- **Low**: Background updates, non-critical information
- **Normal**: Standard communication, default priority
- **High**: Important updates, expedited processing
- **Critical**: Immediate attention required, highest priority

### 4. Delivery Guarantees

Three delivery guarantee levels:

- **AtMostOnce**: Message delivered once or not at all
- **AtLeastOnce**: Message delivered at least once (may be duplicated)
- **ExactlyOnce**: Message delivered exactly once (highest guarantee)

## Communication Patterns

### Request-Response Pattern

```rust
// Create a request
let request = MessageEnvelope::new_request(
    MessageType::Request,
    sender_id,
    recipient_id,
    MessagePayload::Text("Process this data".to_string())
);

// Send and wait for response
let response = agent.request_response(request, Duration::from_secs(10)).await?;
```

### Broadcast Pattern

```rust
// Create a broadcast message
let broadcast = MessageEnvelope::new(
    MessageType::Broadcast,
    sender_id,
    vec![recipient1, recipient2, recipient3], // Multiple recipients
    MessagePayload::Text("System update available".to_string())
);

// Send broadcast
communicator.send_message(broadcast, Duration::from_secs(5)).await?;
```

### Fire-and-Forget Pattern

```rust
// Send message without waiting for response
let message = MessageEnvelope::new(
    MessageType::TaskAssigned,
    sender_id,
    vec![worker_id],
    MessagePayload::TaskInfo { ... }
);

communicator.send_message(message, Duration::from_secs(1)).await?;
```

## Error Handling

### Circuit Breaker Pattern

Prevents cascading failures by temporarily stopping communication when errors exceed threshold:

```rust
let circuit_breaker = CircuitBreaker::new(config);

let result = circuit_breaker.execute(|| async {
    // Communication operation
    agent.communicate(envelope).await
}).await;
```

### Retry Mechanism

Automatic retry with exponential backoff for transient failures:

```rust
let retry = RetryMechanism::new(config);

let result = retry.execute(|| async {
    // Operation that might fail
    communicator.send_message(message, timeout).await
}).await;
```

## Resource Management

### Connection Pooling

Manages communication connections efficiently:

```rust
let pool = ConnectionPool::new(config);

// Get connection
let connection = pool.get_connection("agent_123").await?;

// Use connection
// ...

// Return to pool (automatic)
```

### Resource Limits

Configurable limits prevent resource exhaustion:

```rust
let config = CommunicationConfig {
    max_concurrent_messages: 1000,
    buffer_size: 8192,
    max_retries: 3,
    default_timeout: Duration::from_secs(30),
    // ...
};
```

## Performance Optimizations

### Message Compression

Automatic compression for large messages:

```rust
let config = CommunicationConfig {
    enable_compression: true,
    // ...
};
```

### Batch Processing

Group multiple messages for efficient transmission:

```rust
// Send multiple messages as a batch
communicator.broadcast_batch(messages, recipients).await?;
```

### Connection Reuse

Maintain persistent connections to reduce overhead:

```rust
// Connections are automatically reused from the pool
let connection = pool.get_connection("persistent_agent").await?;
```

## Protocol Versioning

### Version Compatibility

Ensure backward compatibility between protocol versions:

```rust
// Check version compatibility
if current_version.is_compatible(&message_version) {
    // Process message
} else {
    // Handle version mismatch
    return Err(HiveError::ProtocolVersionMismatch);
}
```

### Version Negotiation

Agents negotiate protocol versions during initial handshake:

```rust
// During agent registration
let supported_versions = vec!["1.0", "1.1", "2.0"];
let negotiated_version = negotiate_protocol_version(supported_versions)?;
```

## Monitoring and Metrics

### Communication Statistics

Track communication performance and health:

```rust
let stats = communicator.get_stats().await?;

println!("Messages sent: {}", stats.messages_sent);
println!("Average response time: {}ms", stats.average_response_time_ms);
println!("Error rate: {}%", stats.error_rate * 100.0);
```

### Health Checks

Monitor communication system health:

```rust
// Check if communication is healthy
let health = communication_manager.health_check().await?;
if !health.is_healthy {
    // Handle unhealthy communication
    log::warn!("Communication system unhealthy: {}", health.details);
}
```

## Testing

### Unit Tests

Test individual communication components:

```rust
#[test]
fn test_message_envelope_creation() {
    let envelope = MessageEnvelope::new(/* ... */);
    assert!(!envelope.id.is_nil());
    assert_eq!(envelope.message_type, MessageType::Request);
}
```

### Integration Tests

Test end-to-end communication scenarios:

```rust
#[tokio::test]
async fn test_agent_communication() {
    // Set up test agents
    let agent1 = create_test_agent().await;
    let agent2 = create_test_agent().await;

    // Test communication
    let result = agent1.communicate_with(agent2, test_message).await;

    assert!(result.is_ok());
}
```

### Performance Tests

Benchmark communication performance:

```rust
#[bench]
fn bench_message_serialization(b: &mut Bencher) {
    let envelope = create_large_message();
    b.iter(|| {
        let serialized = MessageSerializer::serialize(&envelope).unwrap();
        let _deserialized = MessageSerializer::deserialize(&serialized).unwrap();
    });
}
```

## Best Practices

### 1. Message Design

- Use appropriate message types for different scenarios
- Include correlation IDs for request-response patterns
- Set reasonable TTL values to prevent message accumulation
- Use metadata for additional context

### 2. Error Handling

- Implement proper error handling for all communication operations
- Use circuit breakers to prevent cascading failures
- Implement retry logic with exponential backoff
- Log errors with sufficient context for debugging

### 3. Performance

- Use connection pooling for frequently contacted agents
- Enable compression for large messages
- Set appropriate timeouts to prevent hanging operations
- Monitor and tune resource limits based on usage patterns

### 4. Security

- Validate message contents and sender identities
- Use appropriate delivery guarantees for sensitive messages
- Implement rate limiting to prevent abuse
- Encrypt sensitive message payloads

### 5. Monitoring

- Monitor message throughput and latency
- Track error rates and failure patterns
- Set up alerts for communication failures
- Log important communication events

## Migration Guide

### From Legacy Communication

1. **Update AgentBehavior trait implementations**:
   ```rust
   // Old
   async fn communicate(&mut self, message: &str, target: Option<Uuid>) -> Result<String>

   // New
   async fn communicate(&mut self, envelope: MessageEnvelope) -> Result<Option<MessageEnvelope>>
   ```

2. **Replace direct WebSocket usage**:
   ```rust
   // Old
   websocket.send(Message::Text(json_string)).await

   // New
   communication_manager.send_message(envelope, timeout).await
   ```

3. **Update message creation**:
   ```rust
   // Old
   let message = format!("Agent {}: {}", agent_name, content);

   // New
   let envelope = MessageEnvelope::new(
       MessageType::Request,
       agent_id,
       vec![target_id],
       MessagePayload::Text(content)
   );
   ```

## Troubleshooting

### Common Issues

1. **Message Timeouts**
   - Check network connectivity
   - Verify timeout values are appropriate
   - Monitor system load

2. **High Error Rates**
   - Check circuit breaker status
   - Review error logs for patterns
   - Verify agent availability

3. **Performance Degradation**
   - Monitor resource usage
   - Check connection pool utilization
   - Review message sizes and compression settings

### Debugging Tools

- Enable detailed logging for communication operations
- Use communication metrics to identify bottlenecks
- Monitor message queues and buffer usage
- Test with isolated agent pairs to identify issues

## Future Enhancements

- **Advanced Routing**: Content-based routing and message filtering
- **Quality of Service**: Guaranteed delivery with QoS parameters
- **Message Encryption**: End-to-end encryption for sensitive communications
- **Distributed Tracing**: Full request tracing across agent networks
- **Adaptive Timeouts**: Dynamic timeout adjustment based on network conditions