//! Test module to verify standardized agent communication patterns

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::*;

    use crate::communication::protocols::{MessageEnvelope, MessagePayload, MessageType};
    use crate::neural::NLPProcessor;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_adaptive_verification_communication() {
        let nlp = Arc::new(
            NLPProcessor::new()
                .await
                .expect("Failed to create NLP processor"),
        );
        let learning_system = Arc::new(tokio::sync::RwLock::new(
            crate::neural::adaptive_learning::AdaptiveLearningSystem::new(),
        ));
        let config = crate::agents::adaptive_verification::AdaptationConfig::default();
        let mut system = crate::agents::adaptive_verification::AdaptiveVerificationSystem::new(
            crate::agents::simple_verification::SimpleVerificationSystem::new(nlp.clone()),
            learning_system,
            config,
        );

        let envelope = MessageEnvelope::new(
            MessageType::Request,
            uuid::Uuid::new_v4(),
            vec![uuid::Uuid::new_v4()],
            MessagePayload::Text("Test request".to_string()),
        );

        let result = system.communicate(envelope).await;
        assert!(result.is_ok());
        let response = result.expect("Communication should succeed");
        assert!(response.is_some());
    }

    #[tokio::test]
    async fn test_simple_verification_communication() {
        let nlp = Arc::new(
            NLPProcessor::new()
                .await
                .expect("Failed to create NLP processor"),
        );
        let mut system = crate::agents::simple_verification::SimpleVerificationSystem::new(nlp);

        let envelope = MessageEnvelope::new(
            MessageType::Request,
            uuid::Uuid::new_v4(),
            vec![uuid::Uuid::new_v4()],
            MessagePayload::Text("Test request".to_string()),
        );

        let result = system.communicate(envelope).await;
        assert!(result.is_ok());
        let response = result.expect("Communication should succeed");
        assert!(response.is_some());
    }

    #[tokio::test]
    async fn test_skill_evolution_communication() {
        let nlp = Arc::new(
            NLPProcessor::new()
                .await
                .expect("Failed to create NLP processor"),
        );
        let config = crate::agents::skill_evolution::SkillEvolutionConfig::default();
        let mut system = crate::agents::skill_evolution::SkillEvolutionSystem::new(nlp, config);

        let envelope = MessageEnvelope::new(
            MessageType::Request,
            uuid::Uuid::new_v4(),
            vec![uuid::Uuid::new_v4()],
            MessagePayload::Text("Test request".to_string()),
        );

        let result = system.communicate(envelope).await;
        assert!(result.is_ok());
        let response = result.expect("Communication should succeed");
        assert!(response.is_some());
    }

    #[tokio::test]
    async fn test_recovery_manager_communication() {
        let mut manager = crate::agents::recovery::AgentRecoveryManager::new();

        let envelope = MessageEnvelope::new(
            MessageType::Request,
            uuid::Uuid::new_v4(),
            vec![uuid::Uuid::new_v4()],
            MessagePayload::Text("Test request".to_string()),
        );

        let result = manager.communicate(envelope).await;
        assert!(result.is_ok());
        let response = result.expect("Communication should succeed");
        assert!(response.is_some());
    }

    #[tokio::test]
    async fn test_pair_coordinator_communication() {
        let nlp = Arc::new(
            NLPProcessor::new()
                .await
                .expect("Failed to create NLP processor"),
        );
        let mut coordinator = crate::agents::verification::PairCoordinator::new(nlp);

        let envelope = MessageEnvelope::new(
            MessageType::Request,
            uuid::Uuid::new_v4(),
            vec![uuid::Uuid::new_v4()],
            MessagePayload::Text("Test request".to_string()),
        );

        let result = coordinator.communicate(envelope).await;
        assert!(result.is_ok());
        let response = result.expect("Communication should succeed");
        assert!(response.is_some());
    }
}
