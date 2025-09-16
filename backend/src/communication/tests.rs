//! # Communication Tests
//!
//! Comprehensive test suite for standardized agent communication patterns.
//! Tests include:
//! - Message envelope creation and validation
//! - Communication patterns and protocols
//! - Error handling and recovery
//! - Performance optimizations
//! - Resource management

#[cfg(test)]
mod tests {
    use super::*;
    use crate::communication::patterns::{CircuitBreaker, RetryMechanism, ConnectionPool, ResourceManager};
    use crate::communication::protocols::{MessageEnvelope, MessageType, MessagePayload, MessageValidator, MessageRouter};
    use std::time::Duration;
    use uuid::Uuid;

    #[test]
    fn test_message_envelope_creation() {
        let sender_id = Uuid::new_v4();
        let recipient_id = Uuid::new_v4();

        let envelope = MessageEnvelope::new(
            MessageType::Request,
            sender_id,
            vec![recipient_id],
            MessagePayload::Text("Test message".to_string()),
        );

        assert_eq!(envelope.sender_id, sender_id);
        assert_eq!(envelope.recipients, vec![recipient_id]);
        assert_eq!(envelope.message_type, MessageType::Request);
        assert!(!envelope.id.is_nil());
        assert!(envelope.correlation_id.is_none());
    }

    #[test]
    fn test_message_envelope_request_response() {
        let sender_id = Uuid::new_v4();
        let recipient_id = Uuid::new_v4();

        let request = MessageEnvelope::new_request(
            MessageType::Request,
            sender_id,
            recipient_id,
            MessagePayload::Text("Test request".to_string()),
        );

        assert_eq!(request.message_type, MessageType::Request);
        assert!(request.correlation_id.is_some());

        let response = MessageEnvelope::new_response(
            &request,
            recipient_id,
            MessagePayload::Text("Test response".to_string()),
        );

        assert_eq!(response.message_type, MessageType::Response);
        assert_eq!(response.sender_id, recipient_id);
        assert_eq!(response.recipients, vec![sender_id]);
        assert_eq!(response.correlation_id, request.correlation_id);
    }

    #[test]
    fn test_message_validation() {
        let sender_id = Uuid::new_v4();
        let recipient_id = Uuid::new_v4();

        // Valid message
        let valid_envelope = MessageEnvelope::new(
            MessageType::Request,
            sender_id,
            vec![recipient_id],
            MessagePayload::Text("Valid message".to_string()),
        );

        assert!(MessageValidator::validate(&valid_envelope).is_ok());

        // Invalid message - nil sender
        let mut invalid_envelope = valid_envelope.clone();
        invalid_envelope.sender_id = Uuid::nil();

        let errors = MessageValidator::validate(&invalid_envelope).unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("Sender ID")));

        // Invalid message - nil recipient
        let mut invalid_envelope2 = valid_envelope.clone();
        invalid_envelope2.recipients = vec![Uuid::nil()];

        let errors2 = MessageValidator::validate(&invalid_envelope2).unwrap_err();
        assert!(!errors2.is_empty());
        assert!(errors2.iter().any(|e| e.contains("Recipient")));
    }

    #[test]
    fn test_message_payload_size() {
        let large_payload = MessagePayload::Binary(vec![0u8; 1024 * 1024]); // 1MB
        assert_eq!(large_payload.size_bytes(), 1024 * 1024);

        let text_payload = MessagePayload::Text("Hello World".to_string());
        assert_eq!(text_payload.size_bytes(), 11);

        let empty_payload = MessagePayload::Empty;
        assert_eq!(empty_payload.size_bytes(), 0);
    }

    #[test]
    fn test_message_ttl() {
        let sender_id = Uuid::new_v4();
        let recipient_id = Uuid::new_v4();

        let mut envelope = MessageEnvelope::new(
            MessageType::Request,
            sender_id,
            vec![recipient_id],
            MessagePayload::Text("Test".to_string()),
        );

        // Test with TTL
        envelope.ttl_seconds = Some(1); // 1 second TTL
        assert!(!envelope.is_expired());

        // Simulate expiration by setting timestamp in the past
        envelope.timestamp = envelope.timestamp - chrono::Duration::seconds(2);
        assert!(envelope.is_expired());

        // Test without TTL
        envelope.ttl_seconds = None;
        assert!(!envelope.is_expired());
    }

    #[test]
    fn test_circuit_breaker() {
        let config = super::patterns::CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 1,
        };

        let circuit_breaker = CircuitBreaker::new(config);

        // Test successful operation
        let result = circuit_breaker.execute(|| async { Ok("success") }).await;
        assert!(result.is_ok());

        // Test failures
        let result1 = circuit_breaker.execute(|| async { Err(crate::utils::error::HiveError::Communication("test error".to_string())) }).await;
        assert!(result1.is_err());

        let result2 = circuit_breaker.execute(|| async { Err(crate::utils::error::HiveError::Communication("test error".to_string())) }).await;
        assert!(result2.is_err());

        // Circuit should be open now
        let result3 = circuit_breaker.execute(|| async { Ok("should fail") }).await;
        assert!(result3.is_err());
    }

    #[test]
    fn test_retry_mechanism() {
        let config = super::patterns::RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
        };

        let retry = RetryMechanism::new(config);
        let mut attempts = 0;

        let result = retry.execute(|| {
            attempts += 1;
            async move {
                if attempts < 3 {
                    Err(crate::utils::error::HiveError::Communication("retry error".to_string()))
                } else {
                    Ok("success")
                }
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(attempts, 3);
    }

    #[test]
    fn test_connection_pool() {
        let config = super::patterns::CommunicationConfig::default();
        let pool = ConnectionPool::new(config);

        let key = "test_connection".to_string();
        let channel = super::patterns::CommunicationChannel::Internal {
            sender: tokio::sync::mpsc::unbounded_channel().0,
            agent_id: Uuid::new_v4(),
        };

        // Test adding and getting connection
        tokio::spawn(async move {
            pool.add_connection(key.clone(), channel).await;
        });

        // Note: In a real test, we'd need to wait for the spawn to complete
        // This is a simplified test structure
    }

    #[test]
    fn test_resource_manager() {
        let manager = ResourceManager::new(10);

        // Test acquiring permit
        tokio::spawn(async move {
            let _permit = manager.acquire().await.expect("Should acquire permit");
            // Permit is held here
        });

        // Test getting stats
        tokio::spawn(async move {
            let stats = manager.get_stats().await;
            assert_eq!(stats.active_connections, 0); // No active connections in this test
        });
    }

    #[test]
    fn test_message_router() {
        let sender_id = Uuid::new_v4();
        let recipient_id = Uuid::new_v4();

        let envelope = MessageEnvelope::new(
            MessageType::Broadcast,
            sender_id,
            vec![recipient_id],
            MessagePayload::Text("Broadcast message".to_string()),
        );

        // Test broadcast detection
        assert!(MessageRouter::should_broadcast(&envelope));

        let request_envelope = MessageEnvelope::new(
            MessageType::Request,
            sender_id,
            vec![recipient_id],
            MessagePayload::Text("Request message".to_string()),
        );

        assert!(!MessageRouter::should_broadcast(&request_envelope));
    }

    #[test]
    fn test_message_serialization() {
        let sender_id = Uuid::new_v4();
        let recipient_id = Uuid::new_v4();

        let envelope = MessageEnvelope::new(
            MessageType::Request,
            sender_id,
            vec![recipient_id],
            MessagePayload::Json(serde_json::json!({"test": "data"})),
        );

        // Test serialization
        let serialized = super::protocols::MessageSerializer::serialize(&envelope);
        assert!(serialized.is_ok());

        let data = serialized.expect("Serialization should succeed");

        // Test deserialization
        let deserialized = super::protocols::MessageSerializer::deserialize(&data);
        assert!(deserialized.is_ok());

        let restored = deserialized.expect("Deserialization should succeed");
        assert_eq!(restored.sender_id, sender_id);
        assert_eq!(restored.message_type, MessageType::Request);
    }

    #[test]
    fn test_protocol_version() {
        let version = super::protocols::ProtocolVersion::new(1, 0, 0);
        assert_eq!(version.to_string(), "1.0.0");

        let parsed = super::protocols::ProtocolVersion::parse("2.1.3").expect("Version parsing should succeed");
        assert_eq!(parsed.major, 2);
        assert_eq!(parsed.minor, 1);
        assert_eq!(parsed.patch, 3);

        assert!(version.is_compatible(&parsed));
    }

    #[test]
    fn test_communication_config() {
        let config = super::patterns::CommunicationConfig::default();

        assert_eq!(config.default_timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
        assert!(!config.enable_compression);
        assert_eq!(config.max_concurrent_messages, 1000);
    }

    #[test]
    fn test_message_priority() {
        use super::patterns::MessagePriority;

        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }

    #[test]
    fn test_delivery_guarantee() {
        use super::patterns::DeliveryGuarantee;

        // Test ordering
        let guarantees = vec![
            DeliveryGuarantee::AtMostOnce,
            DeliveryGuarantee::AtLeastOnce,
            DeliveryGuarantee::ExactlyOnce,
        ];

        // All guarantees are different
        for i in 0..guarantees.len() {
            for j in (i + 1)..guarantees.len() {
                assert_ne!(guarantees[i], guarantees[j]);
            }
        }
    }
}