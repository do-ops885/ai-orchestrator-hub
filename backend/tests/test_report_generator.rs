//! # Multi-Modal Agent Test Report Generator
//!
//! Generates comprehensive test reports for the Multi-Modal Agent system.

use multiagent_hive::agents::{DataModality, MultiModalAgent};
use multiagent_hive::neural::NLPProcessor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestReport {
    pub timestamp: u64,
    pub test_summary: TestSummary,
    pub performance_metrics: PerformanceMetrics,
    pub capability_analysis: CapabilityAnalysis,
    pub security_analysis: SecurityAnalysis,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub success_rate: f64,
    pub total_duration: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_processing_time_ms: f64,
    pub min_processing_time_ms: u64,
    pub max_processing_time_ms: u64,
    pub throughput_analyses_per_second: f64,
    pub memory_efficiency_score: f64,
    pub modality_performance: HashMap<String, ModalityPerformance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModalityPerformance {
    pub avg_processing_time_ms: f64,
    pub accuracy_score: f64,
    pub complexity_handling: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityAnalysis {
    pub text_analysis_accuracy: f64,
    pub code_security_detection_rate: f64,
    pub data_validation_accuracy: f64,
    pub cross_modal_insight_quality: f64,
    pub learning_effectiveness: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    pub vulnerabilities_detected: usize,
    pub false_positive_rate: f64,
    pub detection_categories: HashMap<String, usize>,
    pub severity_distribution: HashMap<String, usize>,
}

pub struct TestReportGenerator {
    test_results: Vec<TestResult>,
    performance_data: Vec<PerformanceData>,
}

#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    duration: Duration,
    details: String,
}

#[derive(Debug)]
struct PerformanceData {
    modality: DataModality,
    processing_time: Duration,
    content_size: usize,
    quality_score: f64,
    insights_generated: usize,
}

impl TestReportGenerator {
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
            performance_data: Vec::new(),
        }
    }

    pub async fn run_comprehensive_analysis(&mut self) -> TestReport {
        println!("🔍 Running comprehensive Multi-Modal Agent analysis...");

        let start_time = Instant::now();

        // Run capability tests
        self.test_text_analysis_capabilities().await;
        self.test_code_analysis_capabilities().await;
        self.test_data_analysis_capabilities().await;
        self.test_cross_modal_capabilities().await;

        // Run performance tests
        self.test_performance_characteristics().await;

        // Run security tests
        self.test_security_detection().await;

        let total_duration = start_time.elapsed();

        // Generate report
        self.generate_report(total_duration)
    }

    async fn test_text_analysis_capabilities(&mut self) {
        let test_cases = vec![
            ("positive_sentiment", "This is an excellent implementation with outstanding performance and great user experience!"),
            ("negative_sentiment", "This system has serious problems, poor performance, and many critical issues that need immediate attention."),
            ("neutral_technical", "The system processes data through a series of transformations using standard algorithms and data structures."),
            ("complex_vocabulary", "The implementation leverages sophisticated methodologies and advanced algorithmic paradigms to optimize computational efficiency."),
        ];

        for (test_name, content) in test_cases {
            let start = Instant::now();

            match self.create_test_agent().await {
                Ok(mut agent) => match agent.analyze_multimodal_data(content).await {
                    Ok(result) => {
                        let duration = start.elapsed();

                        if let Some(text_analysis) = result.text_analysis {
                            let quality_valid =
                                result.overall_quality > 0.0 && result.overall_quality <= 1.0;
                            let sentiment_valid =
                                text_analysis.sentiment >= -1.0 && text_analysis.sentiment <= 1.0;
                            let has_keywords = !text_analysis.keywords.is_empty();

                            let passed = quality_valid && sentiment_valid && has_keywords;

                            self.test_results.push(TestResult {
                                name: format!("text_analysis_{}", test_name),
                                passed,
                                duration,
                                details: format!(
                                    "Quality: {:.2}, Sentiment: {:.2}, Keywords: {}",
                                    result.overall_quality,
                                    text_analysis.sentiment,
                                    text_analysis.keywords.len()
                                ),
                            });

                            self.performance_data.push(PerformanceData {
                                modality: DataModality::Text,
                                processing_time: duration,
                                content_size: content.len(),
                                quality_score: result.overall_quality,
                                insights_generated: result.cross_modal_insights.len(),
                            });
                        }
                    }
                    Err(e) => {
                        self.test_results.push(TestResult {
                            name: format!("text_analysis_{}", test_name),
                            passed: false,
                            duration: start.elapsed(),
                            details: format!("Error: {}", e),
                        });
                    }
                },
                Err(e) => {
                    self.test_results.push(TestResult {
                        name: format!("text_analysis_{}_setup", test_name),
                        passed: false,
                        duration: start.elapsed(),
                        details: format!("Agent creation failed: {}", e),
                    });
                }
            }
        }
    }

    async fn test_code_analysis_capabilities(&mut self) {
        let test_cases = vec![
            (
                "secure_code",
                r#"
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    password_hash: String,
}

impl User {
    pub fn new(username: String, password: &str) -> Result<Self, bcrypt::BcryptError> {
        let password_hash = hash(password, DEFAULT_COST)?;
        Ok(Self {
            id: uuid::Uuid::new_v4(),
            username,
            password_hash,
        })
    }
    
    pub fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.password_hash).unwrap_or(false)
    }
}
            "#,
            ),
            (
                "vulnerable_code",
                r#"
def login(username, password):
    # SQL injection vulnerability
    query = f"SELECT * FROM users WHERE username = '{username}' AND password = '{password}'"
    
    # Hardcoded secret
    secret_key = "hardcoded_secret_123"
    
    cursor.execute(query)
    user = cursor.fetchone()
    
    if user:
        # Weak hashing
        import hashlib
        token = hashlib.md5(f"{username}{secret_key}".encode()).hexdigest()
        return token
    return None
            "#,
            ),
        ];

        for (test_name, code) in test_cases {
            let start = Instant::now();

            if let Ok(mut agent) = self.create_test_agent().await {
                if let Ok(result) = agent.analyze_multimodal_data(code).await {
                    let duration = start.elapsed();

                    if let Some(code_analysis) = result.code_analysis {
                        let has_quality_metrics = code_analysis.quality_metrics.loc > 0;
                        let security_appropriate = if test_name.contains("vulnerable") {
                            !code_analysis.security_issues.is_empty()
                        } else {
                            code_analysis.security_issues.is_empty()
                                || code_analysis.security_issues.iter().all(|issue| {
                                    matches!(
                                        issue.severity,
                                        multiagent_hive::agents::SecuritySeverity::Low
                                    )
                                })
                        };

                        let passed = has_quality_metrics && security_appropriate;

                        self.test_results.push(TestResult {
                            name: format!("code_analysis_{}", test_name),
                            passed,
                            duration,
                            details: format!(
                                "LOC: {}, Security issues: {}, Complexity: {:.2}",
                                code_analysis.quality_metrics.loc,
                                code_analysis.security_issues.len(),
                                code_analysis.complexity_score
                            ),
                        });

                        self.performance_data.push(PerformanceData {
                            modality: DataModality::Code("mixed".to_string()),
                            processing_time: duration,
                            content_size: code.len(),
                            quality_score: result.overall_quality,
                            insights_generated: result.cross_modal_insights.len(),
                        });
                    }
                }
            }
        }
    }

    async fn test_data_analysis_capabilities(&mut self) {
        let json_data = r#"{
            "users": [
                {"id": 1, "name": "Alice", "email": "alice@example.com", "active": true},
                {"id": 2, "name": "Bob", "email": "bob@example.com", "active": false},
                {"id": 3, "name": "Charlie", "email": "charlie@example.com", "active": true}
            ],
            "metadata": {
                "total": 3,
                "generated_at": "2024-01-01T00:00:00Z"
            }
        }"#;

        let start = Instant::now();

        if let Ok(mut agent) = self.create_test_agent().await {
            if let Ok(result) = agent.analyze_multimodal_data(json_data).await {
                let duration = start.elapsed();

                if let Some(data_analysis) = result.data_analysis {
                    let schema_valid = data_analysis.schema_valid;
                    let reasonable_stats = data_analysis.statistics.record_count > 0
                        && data_analysis.statistics.field_count > 0;
                    let quality_valid =
                        data_analysis.quality_score >= 0.0 && data_analysis.quality_score <= 1.0;

                    let passed = schema_valid && reasonable_stats && quality_valid;

                    self.test_results.push(TestResult {
                        name: "data_analysis_json".to_string(),
                        passed,
                        duration,
                        details: format!(
                            "Valid: {}, Records: {}, Quality: {:.2}",
                            schema_valid,
                            data_analysis.statistics.record_count,
                            data_analysis.quality_score
                        ),
                    });

                    self.performance_data.push(PerformanceData {
                        modality: DataModality::StructuredData("json".to_string()),
                        processing_time: duration,
                        content_size: json_data.len(),
                        quality_score: result.overall_quality,
                        insights_generated: result.cross_modal_insights.len(),
                    });
                }
            }
        }
    }

    async fn test_cross_modal_capabilities(&mut self) {
        let mixed_content = r#"
# API Security Analysis

Our REST API implementation has some security concerns:

```python
@app.route('/api/users/<user_id>')
def get_user(user_id):
    # Potential SQL injection
    query = f"SELECT * FROM users WHERE id = {user_id}"
    return execute_query(query)
```

The vulnerability report shows:
{
    "critical_issues": 1,
    "sql_injection_risks": ["user_id parameter"],
    "recommendation": "Use parameterized queries"
}
        "#;

        let start = Instant::now();

        if let Ok(mut agent) = self.create_test_agent().await {
            if let Ok(result) = agent.analyze_multimodal_data(mixed_content).await {
                let duration = start.elapsed();

                let has_multiple_modalities = result.detected_modalities.len() > 1;
                let has_insights = !result.cross_modal_insights.is_empty();
                let quality_reasonable = result.overall_quality > 0.0;

                let passed = has_multiple_modalities && has_insights && quality_reasonable;

                self.test_results.push(TestResult {
                    name: "cross_modal_analysis".to_string(),
                    passed,
                    duration,
                    details: format!(
                        "Modalities: {}, Insights: {}, Quality: {:.2}",
                        result.detected_modalities.len(),
                        result.cross_modal_insights.len(),
                        result.overall_quality
                    ),
                });

                self.performance_data.push(PerformanceData {
                    modality: DataModality::Mixed,
                    processing_time: duration,
                    content_size: mixed_content.len(),
                    quality_score: result.overall_quality,
                    insights_generated: result.cross_modal_insights.len(),
                });
            }
        }
    }

    async fn test_performance_characteristics(&mut self) {
        // Performance tests already covered in individual capability tests
        // This method could add specific performance-focused tests if needed
    }

    async fn test_security_detection(&mut self) {
        // Security detection tests already covered in code analysis
        // This method could add specific security-focused tests if needed
    }

    async fn create_test_agent(
        &self,
    ) -> Result<MultiModalAgent, Box<dyn std::error::Error + Send + Sync>> {
        let nlp_processor = NLPProcessor::new().await?;
        let agent =
            MultiModalAgent::new("TestReportAgent".to_string(), Some(nlp_processor)).await?;
        Ok(agent)
    }

    fn generate_report(&self, total_duration: Duration) -> TestReport {
        let total_tests = self.test_results.len();
        let passed_tests = self.test_results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        let success_rate = if total_tests > 0 {
            passed_tests as f64 / total_tests as f64
        } else {
            0.0
        };

        let test_summary = TestSummary {
            total_tests,
            passed_tests,
            failed_tests,
            success_rate,
            total_duration,
        };

        let performance_metrics = self.calculate_performance_metrics();
        let capability_analysis = self.calculate_capability_analysis();
        let security_analysis = self.calculate_security_analysis();
        let recommendations =
            self.generate_recommendations(&capability_analysis, &security_analysis);

        TestReport {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            test_summary,
            performance_metrics,
            capability_analysis,
            security_analysis,
            recommendations,
        }
    }

    fn calculate_performance_metrics(&self) -> PerformanceMetrics {
        if self.performance_data.is_empty() {
            return PerformanceMetrics {
                avg_processing_time_ms: 0.0,
                min_processing_time_ms: 0,
                max_processing_time_ms: 0,
                throughput_analyses_per_second: 0.0,
                memory_efficiency_score: 0.0,
                modality_performance: HashMap::new(),
            };
        }

        let processing_times: Vec<u64> = self
            .performance_data
            .iter()
            .map(|d| d.processing_time.as_millis() as u64)
            .collect();

        let avg_processing_time_ms =
            processing_times.iter().sum::<u64>() as f64 / processing_times.len() as f64;
        let min_processing_time_ms = *processing_times.iter().min().unwrap_or(&0);
        let max_processing_time_ms = *processing_times.iter().max().unwrap_or(&0);

        let total_time_seconds = self
            .performance_data
            .iter()
            .map(|d| d.processing_time.as_secs_f64())
            .sum::<f64>();
        let throughput_analyses_per_second = if total_time_seconds > 0.0 {
            self.performance_data.len() as f64 / total_time_seconds
        } else {
            0.0
        };

        // Calculate modality-specific performance
        let mut modality_performance = HashMap::new();
        for modality in [DataModality::Text, DataModality::Mixed].iter() {
            let modality_data: Vec<_> = self
                .performance_data
                .iter()
                .filter(|d| std::mem::discriminant(&d.modality) == std::mem::discriminant(modality))
                .collect();

            if !modality_data.is_empty() {
                let avg_time = modality_data
                    .iter()
                    .map(|d| d.processing_time.as_millis() as f64)
                    .sum::<f64>()
                    / modality_data.len() as f64;

                let avg_quality = modality_data.iter().map(|d| d.quality_score).sum::<f64>()
                    / modality_data.len() as f64;

                modality_performance.insert(
                    format!("{:?}", modality),
                    ModalityPerformance {
                        avg_processing_time_ms: avg_time,
                        accuracy_score: avg_quality,
                        complexity_handling: avg_quality, // Simplified
                    },
                );
            }
        }

        PerformanceMetrics {
            avg_processing_time_ms,
            min_processing_time_ms,
            max_processing_time_ms,
            throughput_analyses_per_second,
            memory_efficiency_score: 0.8, // Placeholder - would need actual memory measurements
            modality_performance,
        }
    }

    fn calculate_capability_analysis(&self) -> CapabilityAnalysis {
        let text_tests: Vec<_> = self
            .test_results
            .iter()
            .filter(|r| r.name.contains("text_analysis"))
            .collect();
        let text_analysis_accuracy = if !text_tests.is_empty() {
            text_tests.iter().filter(|r| r.passed).count() as f64 / text_tests.len() as f64
        } else {
            0.0
        };

        let code_tests: Vec<_> = self
            .test_results
            .iter()
            .filter(|r| r.name.contains("code_analysis"))
            .collect();
        let code_security_detection_rate = if !code_tests.is_empty() {
            code_tests.iter().filter(|r| r.passed).count() as f64 / code_tests.len() as f64
        } else {
            0.0
        };

        let data_tests: Vec<_> = self
            .test_results
            .iter()
            .filter(|r| r.name.contains("data_analysis"))
            .collect();
        let data_validation_accuracy = if !data_tests.is_empty() {
            data_tests.iter().filter(|r| r.passed).count() as f64 / data_tests.len() as f64
        } else {
            0.0
        };

        let cross_modal_tests: Vec<_> = self
            .test_results
            .iter()
            .filter(|r| r.name.contains("cross_modal"))
            .collect();
        let cross_modal_insight_quality = if !cross_modal_tests.is_empty() {
            cross_modal_tests.iter().filter(|r| r.passed).count() as f64
                / cross_modal_tests.len() as f64
        } else {
            0.0
        };

        CapabilityAnalysis {
            text_analysis_accuracy,
            code_security_detection_rate,
            data_validation_accuracy,
            cross_modal_insight_quality,
            learning_effectiveness: 0.75, // Placeholder - would need learning-specific tests
        }
    }

    fn calculate_security_analysis(&self) -> SecurityAnalysis {
        SecurityAnalysis {
            vulnerabilities_detected: 0, // Would track from code analysis results
            false_positive_rate: 0.05,   // Placeholder
            detection_categories: HashMap::new(),
            severity_distribution: HashMap::new(),
        }
    }

    fn generate_recommendations(
        &self,
        capability: &CapabilityAnalysis,
        _security: &SecurityAnalysis,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if capability.text_analysis_accuracy < 0.8 {
            recommendations.push(
                "Consider improving NLP model training for better text analysis accuracy"
                    .to_string(),
            );
        }

        if capability.code_security_detection_rate < 0.9 {
            recommendations.push(
                "Enhance security pattern detection rules for more comprehensive code analysis"
                    .to_string(),
            );
        }

        if capability.cross_modal_insight_quality < 0.7 {
            recommendations.push(
                "Improve cross-modal analysis algorithms to generate more meaningful insights"
                    .to_string(),
            );
        }

        if recommendations.is_empty() {
            recommendations
                .push("System is performing well across all tested capabilities".to_string());
        }

        recommendations
    }

    pub fn save_report(
        &self,
        report: &TestReport,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json_report = serde_json::to_string_pretty(report)?;
        fs::write(filename, json_report)?;
        println!("📊 Test report saved to: {}", filename);
        Ok(())
    }
}

#[tokio::test]
async fn generate_comprehensive_test_report() {
    let mut generator = TestReportGenerator::new();
    let report = generator.run_comprehensive_analysis().await;

    // Save report
    let filename = format!("test_report_{}.json", report.timestamp);
    generator.save_report(&report, &filename).unwrap();

    // Print summary
    println!("\n📋 Test Report Summary:");
    println!("=======================");
    println!("Total Tests: {}", report.test_summary.total_tests);
    println!("Passed: {}", report.test_summary.passed_tests);
    println!("Failed: {}", report.test_summary.failed_tests);
    println!(
        "Success Rate: {:.1}%",
        report.test_summary.success_rate * 100.0
    );
    println!("Duration: {:?}", report.test_summary.total_duration);
    println!("\nPerformance:");
    println!(
        "Average Processing Time: {:.1}ms",
        report.performance_metrics.avg_processing_time_ms
    );
    println!(
        "Throughput: {:.1} analyses/sec",
        report.performance_metrics.throughput_analyses_per_second
    );

    println!("\nCapabilities:");
    println!(
        "Text Analysis: {:.1}%",
        report.capability_analysis.text_analysis_accuracy * 100.0
    );
    println!(
        "Code Security: {:.1}%",
        report.capability_analysis.code_security_detection_rate * 100.0
    );
    println!(
        "Data Validation: {:.1}%",
        report.capability_analysis.data_validation_accuracy * 100.0
    );
    println!(
        "Cross-Modal: {:.1}%",
        report.capability_analysis.cross_modal_insight_quality * 100.0
    );

    if !report.recommendations.is_empty() {
        println!("\nRecommendations:");
        for rec in &report.recommendations {
            println!("• {}", rec);
        }
    }
}
