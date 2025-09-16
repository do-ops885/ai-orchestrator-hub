//! # Communication Protocols
//!
//! This module defines standardized message formats and protocols for agent communication.
//! It includes:
//! - Message envelope structure
//! - Message types and payloads
//! - Serialization/deserialization
//! - Protocol versioning

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::patterns::{DeliveryGuarantee, MessagePriority};

/// Standardized message envelope for all agent communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Unique message identifier
    pub id: Uuid,
    /// Message type identifier
    pub message_type: MessageType,
    /// Sender agent identifier
    pub sender_id: Uuid,
    /// Target recipient(s)
    pub recipients: Vec<Uuid>,
    /// Message payload
    pub payload: MessagePayload,
    /// Message priority
    pub priority: MessagePriority,
    /// Delivery guarantee level
    pub delivery_guarantee: DeliveryGuarantee,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Correlation ID for request-response patterns
    pub correlation_id: Option<Uuid>,
    /// Message time-to-live in seconds
    pub ttl_seconds: Option<u64>,
    /// Protocol version
    pub protocol_version: String,
    /// Message metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl MessageEnvelope {
    /// Create a new message envelope
    pub fn new(
        message_type: MessageType,
        sender_id: Uuid,
        recipients: Vec<Uuid>,
        payload: MessagePayload,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type,
            sender_id,
            recipients,
            payload,
            priority: MessagePriority::Normal,
            delivery_guarantee: DeliveryGuarantee::AtLeastOnce,
            timestamp: Utc::now(),
            correlation_id: None,
            ttl_seconds: Some(300), // 5 minutes default
            protocol_version: "1.0".to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a request message with correlation ID
    pub fn new_request(
        message_type: MessageType,
        sender_id: Uuid,
        recipient: Uuid,
        payload: MessagePayload,
    ) -> Self {
        let correlation_id = Uuid::new_v4();
        Self {
            id: Uuid::new_v4(),
            message_type,
            sender_id,
            recipients: vec![recipient],
            payload,
            priority: MessagePriority::Normal,
            delivery_guarantee: DeliveryGuarantee::AtLeastOnce,
            timestamp: Utc::now(),
            correlation_id: Some(correlation_id),
            ttl_seconds: Some(300),
            protocol_version: "1.0".to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a response message
    pub fn new_response(
        original_message: &MessageEnvelope,
        sender_id: Uuid,
        payload: MessagePayload,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type: MessageType::Response,
            sender_id,
            recipients: vec![original_message.sender_id],
            payload,
            priority: original_message.priority,
            delivery_guarantee: original_message.delivery_guarantee,
            timestamp: Utc::now(),
            correlation_id: original_message.correlation_id,
            ttl_seconds: original_message.ttl_seconds,
            protocol_version: original_message.protocol_version.clone(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Check if message is expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let elapsed = self.timestamp.signed_duration_since(Utc::now());
            elapsed.num_seconds() > ttl as i64
        } else {
            false
        }
    }

    /// Add metadata to the message
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set delivery guarantee
    pub fn with_delivery_guarantee(mut self, guarantee: DeliveryGuarantee) -> Self {
        self.delivery_guarantee = guarantee;
        self
    }

    /// Set TTL
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self
    }
}

/// Standardized message types for agent communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MessageType {
    // Agent lifecycle messages
    AgentCreated,
    AgentDestroyed,
    AgentStatusUpdate,

    // Task-related messages
    TaskAssigned,
    TaskCompleted,
    TaskFailed,
    TaskProgress,

    // Communication messages
    Request,
    Response,
    Broadcast,
    Multicast,

    // Coordination messages
    CoordinationRequest,
    CoordinationResponse,
    SwarmUpdate,

    // Error messages
    Error,
    Timeout,

    // Custom messages
    Custom(String),
}

/// Standardized message payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Empty payload
    Empty,

    /// JSON payload
    Json(serde_json::Value),

    /// Binary payload
    Binary(Vec<u8>),

    /// Text payload
    Text(String),

    /// Structured agent status
    AgentStatus {
        agent_id: Uuid,
        state: String,
        capabilities: Vec<String>,
        position: Option<(f64, f64)>,
        energy: Option<f64>,
    },

    /// Task information
    TaskInfo {
        task_id: Uuid,
        task_type: String,
        description: String,
        priority: String,
        progress: Option<f64>,
        assigned_agent: Option<Uuid>,
    },

    /// Error information
    ErrorInfo {
        error_code: String,
        error_message: String,
        error_details: Option<serde_json::Value>,
    },

    /// Coordination data
    CoordinationData {
        swarm_center: (f64, f64),
        neighbor_count: usize,
        coordination_strategy: String,
        performance_metrics: std::collections::HashMap<String, f64>,
    },
}

impl MessagePayload {
    /// Get the size of the payload in bytes
    pub fn size_bytes(&self) -> usize {
        match self {
            MessagePayload::Empty => 0,
            MessagePayload::Json(value) => {
                serde_json::to_string(value).map(|s| s.len()).unwrap_or(0)
            }
            MessagePayload::Binary(data) => data.len(),
            MessagePayload::Text(text) => text.len(),
            MessagePayload::AgentStatus { .. } => 256, // Approximate size
            MessagePayload::TaskInfo { .. } => 512,    // Approximate size
            MessagePayload::ErrorInfo { .. } => 256,   // Approximate size
            MessagePayload::CoordinationData { .. } => 1024, // Approximate size
        }
    }

    /// Check if payload is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, MessagePayload::Empty)
    }
}

/// Protocol version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ProtocolVersion {
    /// Create a new protocol version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse version from string
    pub fn parse(version: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", version));
        }

        let major = parts[0].parse().map_err(|_| "Invalid major version")?;
        let minor = parts[1].parse().map_err(|_| "Invalid minor version")?;
        let patch = parts[2].parse().map_err(|_| "Invalid patch version")?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// Check if this version is compatible with another
    pub fn is_compatible(&self, other: &ProtocolVersion) -> bool {
        self.major == other.major
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

/// Message serialization/deserialization utilities
pub struct MessageSerializer;

impl MessageSerializer {
    /// Serialize a message envelope to bytes
    pub fn serialize(envelope: &MessageEnvelope) -> Result<Vec<u8>, String> {
        serde_json::to_vec(envelope).map_err(|e| format!("Serialization failed: {}", e))
    }

    /// Deserialize a message envelope from bytes
    pub fn deserialize(data: &[u8]) -> Result<MessageEnvelope, String> {
        serde_json::from_slice(data).map_err(|e| format!("Deserialization failed: {}", e))
    }

    /// Serialize with compression
    pub fn serialize_compressed(envelope: &MessageEnvelope) -> Result<Vec<u8>, String> {
        let json_data =
            serde_json::to_vec(envelope).map_err(|e| format!("Serialization failed: {}", e))?;

        // Simple compression using flate2
        use std::io::Write;
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder
            .write_all(&json_data)
            .map_err(|e| format!("Compression failed: {}", e))?;
        encoder
            .finish()
            .map_err(|e| format!("Compression finish failed: {}", e))
    }

    /// Deserialize with decompression
    pub fn deserialize_compressed(data: &[u8]) -> Result<MessageEnvelope, String> {
        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| format!("Decompression failed: {}", e))?;

        serde_json::from_slice(&decompressed).map_err(|e| format!("Deserialization failed: {}", e))
    }
}

/// Message validation utilities
pub struct MessageValidator;

impl MessageValidator {
    /// Validate a message envelope
    pub fn validate(envelope: &MessageEnvelope) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check required fields
        if envelope.sender_id.is_nil() {
            errors.push("Sender ID cannot be nil".to_string());
        }

        if envelope.recipients.is_empty() {
            errors.push("Recipients list cannot be empty".to_string());
        }

        if envelope.recipients.iter().any(|id| id.is_nil()) {
            errors.push("Recipient IDs cannot be nil".to_string());
        }

        // Check TTL
        if let Some(ttl) = envelope.ttl_seconds {
            if ttl == 0 {
                errors.push("TTL cannot be zero".to_string());
            }
        }

        // Check timestamp is not in the future (with some tolerance)
        let now = Utc::now();
        let tolerance = chrono::Duration::seconds(5);
        if envelope.timestamp > now + tolerance {
            errors.push("Message timestamp cannot be in the future".to_string());
        }

        // Check payload size (arbitrary limit)
        if envelope.payload.size_bytes() > 10 * 1024 * 1024 {
            // 10MB
            errors.push("Message payload too large".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate protocol version compatibility
    pub fn validate_protocol_version(
        message_version: &str,
        current_version: &ProtocolVersion,
    ) -> Result<(), String> {
        let message_ver = ProtocolVersion::parse(message_version)
            .map_err(|e| format!("Invalid protocol version: {}", e))?;

        if !current_version.is_compatible(&message_ver) {
            return Err(format!(
                "Protocol version {} is not compatible with current version {}",
                message_version,
                current_version.to_string()
            ));
        }

        Ok(())
    }
}

/// Message routing utilities
pub struct MessageRouter;

impl MessageRouter {
    /// Route a message to appropriate handlers based on message type
    pub fn route_message<'a>(
        envelope: &MessageEnvelope,
        handlers: &'a [Box<dyn super::patterns::MessageHandler>],
    ) -> Vec<&'a Box<dyn super::patterns::MessageHandler>> {
        handlers
            .iter()
            .filter(|handler| handler.can_handle(&envelope.message_type))
            .collect()
    }

    /// Determine if a message should be broadcast
    pub fn should_broadcast(envelope: &MessageEnvelope) -> bool {
        matches!(
            envelope.message_type,
            MessageType::Broadcast | MessageType::SwarmUpdate
        ) || envelope.recipients.len() > 10 // Arbitrary threshold
    }

    /// Get routing priority based on message characteristics
    pub fn get_routing_priority(envelope: &MessageEnvelope) -> MessagePriority {
        // Critical messages get highest priority
        if matches!(
            envelope.message_type,
            MessageType::Error | MessageType::Timeout
        ) {
            return MessagePriority::Critical;
        }

        // High priority for coordination and task messages
        if matches!(
            envelope.message_type,
            MessageType::CoordinationRequest | MessageType::TaskAssigned | MessageType::TaskFailed
        ) {
            return MessagePriority::High;
        }

        // Use the message's own priority
        envelope.priority.clone()
    }
}

/// Message deduplication utilities
pub struct MessageDeduplicator {
    seen_messages: std::collections::HashSet<Uuid>,
    max_size: usize,
}

impl MessageDeduplicator {
    /// Create a new deduplicator
    pub fn new(max_size: usize) -> Self {
        Self {
            seen_messages: std::collections::HashSet::new(),
            max_size,
        }
    }

    /// Check if message is duplicate
    pub fn is_duplicate(&self, message_id: &Uuid) -> bool {
        self.seen_messages.contains(message_id)
    }

    /// Mark message as seen
    pub fn mark_seen(&mut self, message_id: Uuid) {
        if self.seen_messages.len() >= self.max_size {
            // Remove oldest entries (simple FIFO eviction)
            let to_remove: Vec<Uuid> = self.seen_messages.iter().take(100).cloned().collect();
            for id in to_remove {
                self.seen_messages.remove(&id);
            }
        }
        self.seen_messages.insert(message_id);
    }

    /// Clear all seen messages
    pub fn clear(&mut self) {
        self.seen_messages.clear();
    }
}
