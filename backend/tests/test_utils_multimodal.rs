//! # Multi-Modal Agent Test Utilities
//!
//! Shared utilities and helpers for testing the Multi-Modal Agent.

use multiagent_hive::agents::{
    CodeAnalysis, DataModality, MultiModalAgent, StructuredDataAnalysis,
};
use multiagent_hive::neural::{NLPProcessor, ProcessedText};
use std::collections::HashMap;

/// Test data generators for consistent testing
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate realistic code samples for testing
    pub fn generate_realistic_code_samples() -> HashMap<&'static str, &'static str> {
        let mut samples = HashMap::new();

        samples.insert(
            "secure_rust",
            r#"
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SecureUser {
    id: uuid::Uuid,
    username: String,
    password_hash: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl SecureUser {
    pub fn new(username: String, password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
        
        Ok(Self {
            id: uuid::Uuid::new_v4(),
            username,
            password_hash,
            created_at: chrono::Utc::now(),
        })
    }
    
    pub fn verify_password(&self, password: &str) -> bool {
        if let Ok(parsed_hash) = PasswordHash::new(&self.password_hash) {
            Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
        } else {
            false
        }
    }
}
        "#,
        );

        samples.insert("vulnerable_python", r#"
import sqlite3
import hashlib

class InsecureUserManager:
    def __init__(self):
        self.db_path = "users.db"
        self.secret_key = "hardcoded_secret_123"  # SECURITY ISSUE
        
    def create_user(self, username, password):
        # SQL Injection vulnerability
        query = f"INSERT INTO users (username, password) VALUES ('{username}', '{password}')"
        
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute(query)  # VULNERABLE
        conn.commit()
        
    def authenticate(self, username, password):
        # Weak hashing
        password_hash = hashlib.md5(password.encode()).hexdigest()  # WEAK
        
        # More SQL injection
        query = f"SELECT * FROM users WHERE username = '{username}' AND password = '{password_hash}'"
        
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute(query)  # VULNERABLE
        return cursor.fetchone() is not None
        "#);

        samples.insert(
            "complex_javascript",
            r#"
class AdvancedDataProcessor {
    constructor(options = {}) {
        this.config = {
            batchSize: options.batchSize || 100,
            retryAttempts: options.retryAttempts || 3,
            timeout: options.timeout || 5000,
            ...options
        };
        
        this.cache = new Map();
        this.processingQueue = new Set();
        this.metrics = {
            processed: 0,
            failed: 0,
            cacheHits: 0
        };
    }
    
    async processBatch(items) {
        const batches = this.chunkArray(items, this.config.batchSize);
        const results = [];
        
        for (const batch of batches) {
            try {
                const batchResults = await Promise.allSettled(
                    batch.map(item => this.processItem(item))
                );
                
                results.push(...batchResults.map(result => 
                    result.status === 'fulfilled' ? result.value : null
                ).filter(Boolean));
                
            } catch (error) {
                console.error('Batch processing failed:', error);
                this.metrics.failed += batch.length;
            }
        }
        
        return results;
    }
    
    async processItem(item) {
        const cacheKey = this.generateCacheKey(item);
        
        if (this.cache.has(cacheKey)) {
            this.metrics.cacheHits++;
            return this.cache.get(cacheKey);
        }
        
        if (this.processingQueue.has(cacheKey)) {
            // Wait for ongoing processing
            return this.waitForProcessing(cacheKey);
        }
        
        this.processingQueue.add(cacheKey);
        
        try {
            const result = await this.performProcessing(item);
            this.cache.set(cacheKey, result);
            this.metrics.processed++;
            return result;
        } finally {
            this.processingQueue.delete(cacheKey);
        }
    }
}
        "#,
        );

        samples
    }

    /// Generate realistic JSON data samples
    pub fn generate_json_samples() -> HashMap<&'static str, &'static str> {
        let mut samples = HashMap::new();

        samples.insert(
            "user_analytics",
            r#"{
  "analytics": {
    "users": {
      "total": 15420,
      "active_last_30_days": 8934,
      "new_registrations": 324,
      "churn_rate": 0.034
    },
    "engagement": {
      "average_session_duration": 1847,
      "page_views_per_session": 4.7,
      "bounce_rate": 0.23,
      "conversion_rate": 0.087
    },
    "revenue": {
      "total": 289430.50,
      "recurring": 245670.30,
      "one_time": 43760.20,
      "currency": "USD"
    },
    "top_features": [
      {"name": "dashboard", "usage": 0.89},
      {"name": "reports", "usage": 0.76},
      {"name": "integrations", "usage": 0.54},
      {"name": "api", "usage": 0.32}
    ]
  },
  "metadata": {
    "generated_at": "2024-01-15T10:30:00Z",
    "period": "2024-01-01T00:00:00Z/2024-01-31T23:59:59Z",
    "version": "2.1.0"
  }
}"#,
        );

        samples.insert(
            "complex_config",
            r#"{
  "application": {
    "name": "multi-modal-system",
    "version": "1.0.0",
    "environment": "production"
  },
  "database": {
    "primary": {
      "host": "primary-db.internal",
      "port": 5432,
      "database": "app_production",
      "pool_size": 20,
      "timeout": 30000
    },
    "replica": {
      "host": "replica-db.internal",
      "port": 5432,
      "database": "app_production",
      "pool_size": 10,
      "read_only": true
    }
  },
  "caching": {
    "redis": {
      "cluster": [
        {"host": "redis-1.internal", "port": 6379},
        {"host": "redis-2.internal", "port": 6379},
        {"host": "redis-3.internal", "port": 6379}
      ],
      "ttl": {
        "user_sessions": 3600,
        "api_responses": 300,
        "analytics": 900
      }
    }
  },
  "monitoring": {
    "metrics": {
      "prometheus": {
        "enabled": true,
        "port": 9090,
        "path": "/metrics"
      }
    },
    "logging": {
      "level": "info",
      "format": "json",
      "destinations": ["stdout", "file", "elasticsearch"]
    },
    "health_checks": {
      "interval": 30,
      "timeout": 5,
      "endpoints": ["/health", "/ready", "/metrics"]
    }
  }
}"#,
        );

        samples
    }
}

/// Test assertion helpers
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that code analysis detected expected security issues
    pub fn assert_security_issues_detected(code_analysis: &CodeAnalysis, expected_issues: &[&str]) {
        for expected in expected_issues {
            assert!(
                code_analysis.security_issues.iter().any(|issue| issue
                    .issue_type
                    .to_lowercase()
                    .contains(&expected.to_lowercase())
                    || issue
                        .description
                        .to_lowercase()
                        .contains(&expected.to_lowercase())),
                "Expected security issue '{}' not found. Found issues: {:?}",
                expected,
                code_analysis
                    .security_issues
                    .iter()
                    .map(|i| &i.issue_type)
                    .collect::<Vec<_>>()
            );
        }
    }

    /// Assert code quality metrics are within expected ranges
    pub fn assert_code_quality_metrics(code_analysis: &CodeAnalysis) {
        let metrics = &code_analysis.quality_metrics;

        assert!(metrics.loc > 0, "Lines of code should be positive");
        assert!(
            metrics.cyclomatic_complexity >= 1.0,
            "Cyclomatic complexity should be at least 1"
        );
        assert!(
            metrics.duplication_ratio >= 0.0 && metrics.duplication_ratio <= 1.0,
            "Duplication ratio should be between 0 and 1"
        );
        assert!(
            metrics.documentation_coverage >= 0.0 && metrics.documentation_coverage <= 1.0,
            "Documentation coverage should be between 0 and 1"
        );
    }

    /// Assert data analysis results are valid
    pub fn assert_data_analysis_valid(data_analysis: &StructuredDataAnalysis) {
        assert!(
            data_analysis.quality_score >= 0.0 && data_analysis.quality_score <= 1.0,
            "Quality score should be between 0 and 1"
        );
        assert!(
            data_analysis.statistics.record_count >= 0,
            "Record count should be non-negative"
        );
        assert!(
            data_analysis.statistics.field_count >= 0,
            "Field count should be non-negative"
        );
        assert!(
            data_analysis.statistics.missing_values_ratio >= 0.0
                && data_analysis.statistics.missing_values_ratio <= 1.0,
            "Missing values ratio should be between 0 and 1"
        );
    }

    /// Assert text analysis results are meaningful
    pub fn assert_text_analysis_valid(text_analysis: &ProcessedText) {
        assert!(
            !text_analysis.original_text.is_empty(),
            "Original text should not be empty"
        );
        assert!(
            text_analysis.sentiment >= -1.0 && text_analysis.sentiment <= 1.0,
            "Sentiment should be between -1 and 1"
        );
        assert!(
            !text_analysis.tokens.is_empty(),
            "Should have extracted tokens"
        );
    }

    /// Assert cross-modal insights are meaningful
    pub fn assert_cross_modal_insights_quality(insights: &[String]) {
        for insight in insights {
            assert!(!insight.is_empty(), "Insights should not be empty");
            assert!(
                insight.len() > 10,
                "Insights should be meaningful (>10 chars)"
            );

            // Should contain connecting words that indicate cross-modal analysis
            let connecting_words = [
                "but",
                "however",
                "while",
                "although",
                "whereas",
                "indicates",
                "suggests",
            ];
            assert!(
                connecting_words
                    .iter()
                    .any(|word| insight.to_lowercase().contains(word)),
                "Insight should contain connecting language: '{}'",
                insight
            );
        }
    }
}

/// Performance test utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    /// Assert processing time is within reasonable bounds for content size
    pub fn assert_processing_time_reasonable(content_size: usize, processing_time_ms: u64) {
        // Base time + scaling factor based on content size
        let expected_max_time = 50 + (content_size / 1000) as u64 * 10; // 50ms base + 10ms per KB

        assert!(
            processing_time_ms <= expected_max_time,
            "Processing time {}ms exceeds expected max {}ms for content size {}",
            processing_time_ms,
            expected_max_time,
            content_size
        );
    }

    /// Assert memory usage stays reasonable
    pub fn assert_memory_usage_reasonable(initial_memory: usize, final_memory: usize) {
        let memory_growth = final_memory.saturating_sub(initial_memory);
        let max_allowed_growth = initial_memory / 2; // Allow 50% growth

        assert!(
            memory_growth <= max_allowed_growth,
            "Memory usage grew by {} bytes, exceeding max allowed growth of {} bytes",
            memory_growth,
            max_allowed_growth
        );
    }
}

/// Create a test agent with standard configuration
pub async fn create_test_agent(name: &str) -> MultiModalAgent {
    let nlp_processor = NLPProcessor::new()
        .await
        .expect("Failed to create NLP processor for test");

    MultiModalAgent::new(name.to_string(), Some(nlp_processor))
        .await
        .expect("Failed to create test MultiModalAgent")
}
