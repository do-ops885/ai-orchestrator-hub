//! # Multi-Modal Intelligence Agent
//!
//! A sophisticated agent capable of processing multiple data modalities simultaneously:
//! - **Text Processing**: Natural language understanding and generation
//! - **Code Analysis**: Syntax parsing, pattern recognition, and quality assessment
//! - **Structured Data**: JSON, XML, and tabular data processing
//!
//! This agent represents a breakthrough in multi-domain intelligence, combining
//! linguistic understanding with technical analysis capabilities.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use uuid::Uuid; // Imported via other modules

use crate::agents::{Agent, AgentBehavior, AgentCapability, AgentState, AgentType, Experience};
use crate::communication::protocols::{MessageEnvelope, MessagePayload};
use crate::neural::{NLPProcessor, ProcessedText};
use crate::tasks::{Task, TaskResult};
use crate::utils::error::{HiveError, HiveResult};

/// Supported data modalities for multi-modal processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataModality {
    /// Natural language text
    Text,
    /// Source code in various programming languages
    Code(String), // language identifier
    /// Structured data (JSON, XML, etc.)
    StructuredData(String), // format identifier
    /// Mixed content containing multiple modalities
    Mixed,
}

/// Analysis result for code processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysis {
    /// Programming language detected
    pub language: String,
    /// Code complexity score (0.0-1.0)
    pub complexity_score: f64,
    /// Quality metrics
    pub quality_metrics: CodeQualityMetrics,
    /// Detected patterns and anti-patterns
    pub patterns: Vec<CodePattern>,
    /// Security vulnerabilities found
    pub security_issues: Vec<SecurityIssue>,
    /// Suggestions for improvement
    pub improvements: Vec<String>,
}

/// Code quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetrics {
    /// Lines of code
    pub loc: usize,
    /// Cyclomatic complexity
    pub cyclomatic_complexity: f64,
    /// Code duplication percentage
    pub duplication_ratio: f64,
    /// Test coverage estimate
    pub test_coverage: Option<f64>,
    /// Documentation coverage
    pub documentation_coverage: f64,
}

/// Detected code pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    /// Pattern type (design pattern, anti-pattern, etc.)
    pub pattern_type: String,
    /// Pattern name
    pub name: String,
    /// Confidence in detection (0.0-1.0)
    pub confidence: f64,
    /// Line numbers where pattern is found
    pub locations: Vec<usize>,
    /// Description of the pattern
    pub description: String,
}

/// Security issue detected in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// Severity level
    pub severity: SecuritySeverity,
    /// Issue type (SQL injection, XSS, etc.)
    pub issue_type: String,
    /// Line number where issue occurs
    pub line_number: usize,
    /// Description of the issue
    pub description: String,
    /// Suggested fix
    pub suggested_fix: String,
}

/// Security issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Analysis result for structured data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredDataAnalysis {
    /// Data format (JSON, XML, CSV, etc.)
    pub format: String,
    /// Schema validation result
    pub schema_valid: bool,
    /// Data quality score (0.0-1.0)
    pub quality_score: f64,
    /// Statistical summary
    pub statistics: DataStatistics,
    /// Detected anomalies
    pub anomalies: Vec<DataAnomaly>,
    /// Suggested transformations
    pub transformations: Vec<String>,
}

/// Statistical summary of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStatistics {
    /// Number of records/entries
    pub record_count: usize,
    /// Number of fields/columns
    pub field_count: usize,
    /// Missing value percentage
    pub missing_values_ratio: f64,
    /// Data type distribution
    pub type_distribution: HashMap<String, usize>,
    /// Uniqueness ratios for each field
    pub uniqueness_ratios: HashMap<String, f64>,
}

/// Detected data anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAnomaly {
    /// Anomaly type
    pub anomaly_type: String,
    /// Field/location where anomaly occurs
    pub location: String,
    /// Confidence in detection (0.0-1.0)
    pub confidence: f64,
    /// Description
    pub description: String,
    /// Suggested action
    pub suggested_action: String,
}

/// Comprehensive multi-modal analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalAnalysis {
    /// Primary modality detected
    pub primary_modality: DataModality,
    /// All modalities present in the data
    pub detected_modalities: Vec<DataModality>,
    /// Text analysis (if applicable)
    pub text_analysis: Option<ProcessedText>,
    /// Code analysis (if applicable)
    pub code_analysis: Option<CodeAnalysis>,
    /// Structured data analysis (if applicable)
    pub data_analysis: Option<StructuredDataAnalysis>,
    /// Cross-modal insights
    pub cross_modal_insights: Vec<String>,
    /// Overall quality score
    pub overall_quality: f64,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Multi-Modal Intelligence Agent implementation
#[derive(Debug, Clone)]
pub struct MultiModalAgent {
    /// Base agent properties
    pub base: Agent,
    /// NLP processor for text analysis
    pub nlp_processor: Option<NLPProcessor>,
    /// Specialized capabilities for multi-modal processing
    pub modality_expertise: HashMap<DataModality, f64>,
    /// Analysis history for learning
    pub analysis_history: Vec<MultiModalAnalysis>,
    /// Performance metrics
    pub performance_metrics: MultiModalMetrics,
}

/// Performance metrics for multi-modal agent
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultiModalMetrics {
    /// Total analyses performed
    pub total_analyses: u64,
    /// Analyses by modality
    pub analyses_by_modality: HashMap<String, u64>,
    /// Average processing time by modality
    pub avg_processing_time: HashMap<String, f64>,
    /// Accuracy scores by modality
    pub accuracy_scores: HashMap<String, f64>,
    /// Learning progression over time
    pub learning_curve: Vec<(DateTime<Utc>, f64)>,
}

impl MultiModalAgent {
    /// Create a new Multi-Modal Intelligence Agent
    pub async fn new(name: String, nlp_processor: Option<NLPProcessor>) -> HiveResult<Self> {
        let mut base = Agent::new(name, AgentType::Specialist("MultiModal".to_string()));
        
        // Add specialized capabilities
        base.add_capability(AgentCapability {
            name: "text_processing".to_string(),
            proficiency: 0.8,
            learning_rate: 0.15,
        });
        
        base.add_capability(AgentCapability {
            name: "code_analysis".to_string(),
            proficiency: 0.7,
            learning_rate: 0.12,
        });
        
        base.add_capability(AgentCapability {
            name: "data_processing".to_string(),
            proficiency: 0.75,
            learning_rate: 0.1,
        });
        
        base.add_capability(AgentCapability {
            name: "pattern_recognition".to_string(),
            proficiency: 0.85,
            learning_rate: 0.08,
        });

        // Initialize modality expertise
        let mut modality_expertise = HashMap::new();
        modality_expertise.insert(DataModality::Text, 0.8);
        modality_expertise.insert(DataModality::Code("general".to_string()), 0.7);
        modality_expertise.insert(DataModality::StructuredData("json".to_string()), 0.75);
        modality_expertise.insert(DataModality::Mixed, 0.6);

        Ok(Self {
            base,
            nlp_processor,
            modality_expertise,
            analysis_history: Vec::new(),
            performance_metrics: MultiModalMetrics::default(),
        })
    }

    /// Perform comprehensive multi-modal analysis
    pub async fn analyze_multimodal_data(&mut self, input: &str) -> HiveResult<MultiModalAnalysis> {
        let start_time = std::time::Instant::now();
        
        // Detect modalities present in the input
        let detected_modalities = self.detect_modalities(input).await?;
        let primary_modality = detected_modalities.first()
            .cloned()
            .unwrap_or(DataModality::Text);

        // Perform modality-specific analyses
        let text_analysis = if detected_modalities.contains(&DataModality::Text) {
            self.analyze_text(input).await?
        } else {
            None
        };

        let code_analysis = if detected_modalities.iter().any(|m| matches!(m, DataModality::Code(_))) {
            self.analyze_code(input).await?
        } else {
            None
        };

        let data_analysis = if detected_modalities.iter().any(|m| matches!(m, DataModality::StructuredData(_))) {
            self.analyze_structured_data(input).await?
        } else {
            None
        };

        // Generate cross-modal insights
        let cross_modal_insights = self.generate_cross_modal_insights(
            &text_analysis,
            &code_analysis,
            &data_analysis,
        ).await?;

        // Calculate overall quality score
        let overall_quality = self.calculate_overall_quality(
            &text_analysis,
            &code_analysis,
            &data_analysis,
        );

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        let analysis = MultiModalAnalysis {
            primary_modality,
            detected_modalities,
            text_analysis,
            code_analysis,
            data_analysis,
            cross_modal_insights,
            overall_quality,
            processing_time_ms,
        };

        // Store analysis for learning
        self.analysis_history.push(analysis.clone());
        self.update_performance_metrics(&analysis);

        // Learn from this analysis
        self.learn_from_analysis(&analysis).await?;

        Ok(analysis)
    }

    /// Detect which modalities are present in the input
    async fn detect_modalities(&self, input: &str) -> HiveResult<Vec<DataModality>> {
        let mut modalities = Vec::new();

        // Check for code patterns
        if self.looks_like_code(input) {
            let language = self.detect_programming_language(input);
            modalities.push(DataModality::Code(language));
        }

        // Check for structured data
        if self.looks_like_structured_data(input) {
            let format = self.detect_data_format(input);
            modalities.push(DataModality::StructuredData(format));
        }

        // Always consider as text if it contains readable content
        if self.contains_natural_language(input) {
            modalities.push(DataModality::Text);
        }

        // If multiple modalities detected, also mark as mixed
        if modalities.len() > 1 {
            modalities.push(DataModality::Mixed);
        }

        // Default to text if no specific modality detected
        if modalities.is_empty() {
            modalities.push(DataModality::Text);
        }

        Ok(modalities)
    }

    /// Analyze text content using NLP processor
    async fn analyze_text(&self, input: &str) -> HiveResult<Option<ProcessedText>> {
        if let Some(nlp) = &self.nlp_processor {
            let processed = nlp.process_text(input).await
                .map_err(|e| HiveError::ProcessingError(format!("Text analysis failed: {}", e)))?;
            Ok(Some(processed))
        } else {
            Ok(None)
        }
    }

    /// Analyze code content
    async fn analyze_code(&self, input: &str) -> HiveResult<Option<CodeAnalysis>> {
        let language = self.detect_programming_language(input);
        let complexity_score = self.calculate_code_complexity(input);
        let quality_metrics = self.analyze_code_quality(input);
        let patterns = self.detect_code_patterns(input);
        let security_issues = self.detect_security_issues(input);
        let improvements = self.suggest_code_improvements(input, &patterns, &security_issues);

        Ok(Some(CodeAnalysis {
            language,
            complexity_score,
            quality_metrics,
            patterns,
            security_issues,
            improvements,
        }))
    }

    /// Analyze structured data
    async fn analyze_structured_data(&self, input: &str) -> HiveResult<Option<StructuredDataAnalysis>> {
        let format = self.detect_data_format(input);
        let schema_valid = self.validate_schema(input, &format);
        let quality_score = self.calculate_data_quality(input);
        let statistics = self.calculate_data_statistics(input, &format)?;
        let anomalies = self.detect_data_anomalies(input, &format);
        let transformations = self.suggest_data_transformations(input, &statistics);

        Ok(Some(StructuredDataAnalysis {
            format,
            schema_valid,
            quality_score,
            statistics,
            anomalies,
            transformations,
        }))
    }

    /// Generate insights that span multiple modalities
    async fn generate_cross_modal_insights(
        &self,
        text_analysis: &Option<ProcessedText>,
        code_analysis: &Option<CodeAnalysis>,
        data_analysis: &Option<StructuredDataAnalysis>,
    ) -> HiveResult<Vec<String>> {
        let mut insights = Vec::new();

        // Text + Code insights
        if let (Some(text), Some(code)) = (text_analysis, code_analysis) {
            if text.sentiment > 0.5 && code.quality_metrics.documentation_coverage < 0.3 {
                insights.push(
                    "Positive sentiment in text but low code documentation - consider adding more comments".to_string()
                );
            }
            
            if text.keywords.iter().any(|k| k.contains("test")) && code.quality_metrics.test_coverage.unwrap_or(0.0) < 0.5 {
                insights.push(
                    "Testing mentioned in text but low test coverage detected in code".to_string()
                );
            }
        }

        // Code + Data insights
        if let (Some(code), Some(data)) = (code_analysis, data_analysis) {
            if code.language == "python" && data.format == "json" && data.statistics.missing_values_ratio > 0.2 {
                insights.push(
                    "Python code with JSON data containing high missing values - consider data validation".to_string()
                );
            }
        }

        // Text + Data insights
        if let (Some(text), Some(data)) = (text_analysis, data_analysis) {
            if text.keywords.iter().any(|k| k.contains("quality")) && data.quality_score < 0.6 {
                insights.push(
                    "Quality concerns mentioned in text align with low data quality score".to_string()
                );
            }
        }

        // Multi-modal complexity insight
        let total_complexity = self.calculate_total_complexity(text_analysis, code_analysis, data_analysis);
        if total_complexity > 0.8 {
            insights.push(
                "High complexity detected across multiple modalities - consider simplification".to_string()
            );
        }

        Ok(insights)
    }

    // Helper methods for modality detection
    fn looks_like_code(&self, input: &str) -> bool {
        let code_indicators = [
            "function", "class", "def ", "import ", "include ", "#include",
            "public ", "private ", "var ", "let ", "const ", "if (", "for (",
            "while (", "return ", "throw ", "catch", "try {", "}", "{", "}", ";",
        ];
        
        let indicator_count = code_indicators.iter()
            .filter(|&indicator| input.contains(indicator))
            .count();
            
        indicator_count >= 3 || input.lines().any(|line| {
            line.trim_start().starts_with("//") || 
            line.trim_start().starts_with('#') ||
            line.trim_start().starts_with("/*")
        })
    }

    fn looks_like_structured_data(&self, input: &str) -> bool {
        input.trim_start().starts_with('{') && input.trim_end().ends_with('}') ||
        input.trim_start().starts_with('[') && input.trim_end().ends_with(']') ||
        input.trim_start().starts_with('<') && input.trim_end().ends_with('>') ||
        input.contains(',') && input.lines().count() > 1 // CSV-like
    }

    fn contains_natural_language(&self, input: &str) -> bool {
        let word_count = input.split_whitespace().count();
        let sentence_indicators = input.matches('.').count() + input.matches('!').count() + input.matches('?').count();
        
        word_count > 5 && (sentence_indicators > 0 || word_count > 20)
    }

    fn detect_programming_language(&self, input: &str) -> String {
        if input.contains("fn ") && input.contains("->") && input.contains("pub ") {
            "rust".to_string()
        } else if input.contains("def ") && input.contains(":") {
            "python".to_string()
        } else if input.contains("function ") || input.contains("const ") || input.contains("let ") {
            "javascript".to_string()
        } else if input.contains("public class") || input.contains("private ") {
            "java".to_string()
        } else if input.contains("#include") || input.contains("int main") {
            "c".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn detect_data_format(&self, input: &str) -> String {
        if input.trim_start().starts_with('{') || input.trim_start().starts_with('[') {
            "json".to_string()
        } else if input.trim_start().starts_with('<') {
            "xml".to_string()
        } else if input.contains(',') && input.lines().count() > 1 {
            "csv".to_string()
        } else {
            "unknown".to_string()
        }
    }

    // Code analysis helper methods
    fn calculate_code_complexity(&self, input: &str) -> f64 {
        let lines = input.lines().count();
        let control_structures = input.matches("if ").count() + 
                                input.matches("for ").count() + 
                                input.matches("while ").count() +
                                input.matches("switch ").count();
        let nested_blocks = input.matches('{').count();
        
        // Simple complexity calculation
        let base_complexity = (control_structures as f64 * 0.3 + nested_blocks as f64 * 0.2) / lines.max(1) as f64;
        base_complexity.clamp(0.0, 1.0)
    }

    fn analyze_code_quality(&self, input: &str) -> CodeQualityMetrics {
        let lines = input.lines().collect::<Vec<_>>();
        let loc = lines.len();
        
        // Calculate cyclomatic complexity (simplified)
        let decision_points = input.matches("if ").count() + 
                             input.matches("for ").count() + 
                             input.matches("while ").count() +
                             input.matches("case ").count();
        let cyclomatic_complexity = (decision_points + 1) as f64;
        
        // Estimate duplication (very simplified)
        let unique_lines: std::collections::HashSet<&str> = lines.iter().cloned().collect();
        let duplication_ratio = 1.0 - (unique_lines.len() as f64 / loc.max(1) as f64);
        
        // Documentation coverage
        let comment_lines = lines.iter().filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//") || trimmed.starts_with('#') || 
            trimmed.starts_with("/*") || trimmed.starts_with('*')
        }).count();
        let documentation_coverage = comment_lines as f64 / loc.max(1) as f64;

        CodeQualityMetrics {
            loc,
            cyclomatic_complexity,
            duplication_ratio,
            test_coverage: None, // Would require deeper analysis
            documentation_coverage,
        }
    }

    fn detect_code_patterns(&self, input: &str) -> Vec<CodePattern> {
        let mut patterns = Vec::new();
        
        // Singleton pattern detection
        if input.contains("private static") && input.contains("getInstance") {
            patterns.push(CodePattern {
                pattern_type: "Design Pattern".to_string(),
                name: "Singleton".to_string(),
                confidence: 0.8,
                locations: vec![1], // Simplified
                description: "Singleton pattern detected".to_string(),
            });
        }
        
        // God class anti-pattern
        let method_count = input.matches("def ").count() + input.matches("function ").count();
        if method_count > 20 {
            patterns.push(CodePattern {
                pattern_type: "Anti-Pattern".to_string(),
                name: "God Class".to_string(),
                confidence: 0.7,
                locations: vec![1],
                description: "Class with too many methods detected".to_string(),
            });
        }
        
        patterns
    }

    fn detect_security_issues(&self, input: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        
        // SQL injection risk
        if input.contains("SELECT") && input.contains("'+") {
            issues.push(SecurityIssue {
                severity: SecuritySeverity::High,
                issue_type: "SQL Injection".to_string(),
                line_number: 1, // Simplified
                description: "Potential SQL injection vulnerability".to_string(),
                suggested_fix: "Use parameterized queries".to_string(),
            });
        }
        
        // Hardcoded credentials
        if input.to_lowercase().contains("password") && input.contains("=") {
            issues.push(SecurityIssue {
                severity: SecuritySeverity::Critical,
                issue_type: "Hardcoded Credentials".to_string(),
                line_number: 1,
                description: "Hardcoded password detected".to_string(),
                suggested_fix: "Use environment variables or secure config".to_string(),
            });
        }
        
        issues
    }

    fn suggest_code_improvements(&self, _input: &str, patterns: &[CodePattern], security_issues: &[SecurityIssue]) -> Vec<String> {
        let mut improvements = Vec::new();
        
        for pattern in patterns {
            if pattern.pattern_type == "Anti-Pattern" {
                improvements.push(format!("Refactor {} to improve maintainability", pattern.name));
            }
        }
        
        for issue in security_issues {
            improvements.push(format!("Address {} security issue: {}", issue.issue_type, issue.suggested_fix));
        }
        
        if improvements.is_empty() {
            improvements.push("Code quality looks good!".to_string());
        }
        
        improvements
    }

    // Data analysis helper methods
    fn validate_schema(&self, input: &str, format: &str) -> bool {
        match format {
            "json" => serde_json::from_str::<serde_json::Value>(input).is_ok(),
            "xml" => input.contains('<') && input.contains('>'),
            "csv" => input.lines().count() > 1 && input.contains(','),
            _ => true,
        }
    }

    fn calculate_data_quality(&self, input: &str) -> f64 {
        // Simple quality metrics
        let non_empty_ratio = if input.trim().is_empty() { 0.0 } else { 1.0 };
        let structure_score = if self.looks_like_structured_data(input) { 0.8 } else { 0.4 };
        
        (non_empty_ratio + structure_score) / 2.0
    }

    fn calculate_data_statistics(&self, input: &str, format: &str) -> HiveResult<DataStatistics> {
        match format {
            "json" => self.analyze_json_statistics(input),
            "csv" => self.analyze_csv_statistics(input),
            _ => Ok(DataStatistics {
                record_count: input.lines().count(),
                field_count: 1,
                missing_values_ratio: 0.0,
                type_distribution: std::collections::HashMap::new(),
                uniqueness_ratios: std::collections::HashMap::new(),
            }),
        }
    }

    fn analyze_json_statistics(&self, input: &str) -> HiveResult<DataStatistics> {
        let parsed: serde_json::Value = serde_json::from_str(input)
            .map_err(|e| HiveError::ProcessingError(format!("JSON parsing failed: {}", e)))?;
        
        let (record_count, field_count) = match &parsed {
            serde_json::Value::Object(obj) => (1, obj.len()),
            serde_json::Value::Array(arr) => {
                let max_fields = arr.iter()
                    .filter_map(|v| v.as_object())
                    .map(|obj| obj.len())
                    .max()
                    .unwrap_or(0);
                (arr.len(), max_fields)
            },
            _ => (1, 1),
        };

        Ok(DataStatistics {
            record_count,
            field_count,
            missing_values_ratio: 0.0, // Simplified
            type_distribution: std::collections::HashMap::new(),
            uniqueness_ratios: std::collections::HashMap::new(),
        })
    }

    fn analyze_csv_statistics(&self, input: &str) -> HiveResult<DataStatistics> {
        let lines: Vec<&str> = input.lines().collect();
        let record_count = lines.len().saturating_sub(1); // Subtract header
        let field_count = lines.first()
            .map(|header| header.split(',').count())
            .unwrap_or(0);

        Ok(DataStatistics {
            record_count,
            field_count,
            missing_values_ratio: 0.0, // Simplified
            type_distribution: std::collections::HashMap::new(),
            uniqueness_ratios: std::collections::HashMap::new(),
        })
    }

    fn detect_data_anomalies(&self, _input: &str, _format: &str) -> Vec<DataAnomaly> {
        // Simplified anomaly detection
        Vec::new()
    }

    fn suggest_data_transformations(&self, _input: &str, stats: &DataStatistics) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if stats.missing_values_ratio > 0.1 {
            suggestions.push("Consider handling missing values through imputation or removal".to_string());
        }
        
        if stats.field_count > 50 {
            suggestions.push("Consider dimensionality reduction techniques".to_string());
        }
        
        suggestions
    }

    // Cross-modal analysis helpers
    fn calculate_overall_quality(
        &self,
        text_analysis: &Option<ProcessedText>,
        code_analysis: &Option<CodeAnalysis>,
        data_analysis: &Option<StructuredDataAnalysis>,
    ) -> f64 {
        let mut total_quality = 0.0;
        let mut component_count = 0;

        if let Some(text) = text_analysis {
            // Text quality based on sentiment and keyword richness
            let text_quality = (text.sentiment + 1.0) / 2.0 * 0.7 + 
                              (text.keywords.len() as f64 / 10.0).min(1.0) * 0.3;
            total_quality += text_quality;
            component_count += 1;
        }

        if let Some(code) = code_analysis {
            // Code quality based on complexity and documentation
            let code_quality = (1.0 - code.complexity_score) * 0.6 + 
                              code.quality_metrics.documentation_coverage * 0.4;
            total_quality += code_quality;
            component_count += 1;
        }

        if let Some(data) = data_analysis {
            total_quality += data.quality_score;
            component_count += 1;
        }

        if component_count > 0 {
            total_quality / component_count as f64
        } else {
            0.5 // Default neutral quality
        }
    }

    fn calculate_total_complexity(
        &self,
        text_analysis: &Option<ProcessedText>,
        code_analysis: &Option<CodeAnalysis>,
        data_analysis: &Option<StructuredDataAnalysis>,
    ) -> f64 {
        let mut total_complexity = 0.0;
        let mut component_count = 0;

        if let Some(text) = text_analysis {
            // Text complexity based on vocabulary richness
            let text_complexity = (text.keywords.len() as f64 / 20.0).min(1.0);
            total_complexity += text_complexity;
            component_count += 1;
        }

        if let Some(code) = code_analysis {
            total_complexity += code.complexity_score;
            component_count += 1;
        }

        if let Some(data) = data_analysis {
            // Data complexity based on field count and records
            let data_complexity = ((data.statistics.field_count as f64 / 50.0) + 
                                 (data.statistics.record_count as f64 / 10000.0)).min(1.0) / 2.0;
            total_complexity += data_complexity;
            component_count += 1;
        }

        if component_count > 0 {
            total_complexity / component_count as f64
        } else {
            0.0
        }
    }

    // Learning and metrics methods
    fn update_performance_metrics(&mut self, analysis: &MultiModalAnalysis) {
        self.performance_metrics.total_analyses += 1;
        
        // Update modality-specific metrics
        for modality in &analysis.detected_modalities {
            let modality_key = format!("{:?}", modality);
            *self.performance_metrics.analyses_by_modality.entry(modality_key.clone()).or_insert(0) += 1;
            
            // Update average processing time
            let current_avg = self.performance_metrics.avg_processing_time.get(&modality_key).copied().unwrap_or(0.0);
            let count = self.performance_metrics.analyses_by_modality[&modality_key] as f64;
            let new_avg = (current_avg * (count - 1.0) + analysis.processing_time_ms as f64) / count;
            self.performance_metrics.avg_processing_time.insert(modality_key, new_avg);
        }
        
        // Update learning curve
        self.performance_metrics.learning_curve.push((Utc::now(), analysis.overall_quality));
        
        // Keep only recent learning data
        if self.performance_metrics.learning_curve.len() > 100 {
            self.performance_metrics.learning_curve.remove(0);
        }
    }

    async fn learn_from_analysis(&mut self, analysis: &MultiModalAnalysis) -> HiveResult<()> {
        // Update modality expertise based on analysis quality
        for modality in &analysis.detected_modalities {
            if let Some(expertise) = self.modality_expertise.get_mut(modality) {
                let learning_rate = 0.05;
                let expertise_adjustment = if analysis.overall_quality > 0.7 {
                    learning_rate
                } else {
                    -learning_rate * 0.5
                };
                *expertise = (*expertise + expertise_adjustment).clamp(0.0, 1.0);
            }
        }

        // Update agent capabilities
        let capability_names = ["text_processing", "code_analysis", "data_processing", "pattern_recognition"];
        for cap_name in &capability_names {
            if let Some(capability) = self.base.capabilities.iter_mut().find(|c| c.name == *cap_name) {
                let adjustment = if analysis.overall_quality > 0.7 {
                    capability.learning_rate * 0.1
                } else {
                    -capability.learning_rate * 0.05
                };
                capability.proficiency = (capability.proficiency + adjustment).clamp(0.0, 1.0);
            }
        }

        // Create experience for base agent
        let experience = Experience {
            timestamp: Utc::now(),
            task_type: "multimodal_analysis".to_string(),
            success: analysis.overall_quality > 0.6,
            context: format!("Analyzed {} modalities with quality score {:.2}", 
                           analysis.detected_modalities.len(), analysis.overall_quality),
            learned_insight: Some(format!(
                "Cross-modal insights: {}", 
                analysis.cross_modal_insights.join("; ")
            )),
        };
        
        self.base.learn_from_experience(experience);
        
        Ok(())
    }

    /// Get current expertise levels for each modality
    pub fn get_modality_expertise(&self) -> &HashMap<DataModality, f64> {
        &self.modality_expertise
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> &MultiModalMetrics {
        &self.performance_metrics
    }

    /// Get analysis history
    pub fn get_analysis_history(&self) -> &[MultiModalAnalysis] {
        &self.analysis_history
    }

    /// Clear analysis history (useful for memory management)
    pub fn clear_analysis_history(&mut self) {
        self.analysis_history.clear();
    }
}

#[async_trait]
impl AgentBehavior for MultiModalAgent {
    async fn execute_task(&mut self, task: Task) -> HiveResult<TaskResult> {
        // Use standardized state management from base agent
        self.base.execute_with_state_management(AgentState::Working, |agent| {
            // This is a synchronous closure, so we can't use async here
            // We'll handle the task execution synchronously or refactor if needed
            Ok(())
        })?;

        let start_time = std::time::Instant::now();
        
        // Determine if this task requires multi-modal analysis
        let requires_multimodal = task.required_capabilities.iter().any(|cap| {
            matches!(cap.name.as_str(), "text_processing" | "code_analysis" | "data_processing" | "pattern_recognition")
        });

        let result = if requires_multimodal {
            // Perform multi-modal analysis on task description and context
            let input_text = format!("{}\n{}", task.description, 
                                   task.context.values().cloned().collect::<Vec<_>>().join("\n"));
            
            match self.analyze_multimodal_data(&input_text).await {
                Ok(analysis) => {
                    let output = format!(
                        "Multi-modal analysis completed:\n\
                         - Primary modality: {:?}\n\
                         - Detected modalities: {:?}\n\
                         - Overall quality: {:.2}\n\
                         - Cross-modal insights: {}\n\
                         - Processing time: {}ms",
                        analysis.primary_modality,
                        analysis.detected_modalities,
                        analysis.overall_quality,
                        analysis.cross_modal_insights.join("; "),
                        analysis.processing_time_ms
                    );
                    
                    TaskResult::success(task.id, self.base.id, output, start_time.elapsed().as_millis() as u64)
                        .with_quality_score(analysis.overall_quality)
                        .with_insights(analysis.cross_modal_insights)
                }
                Err(e) => {
                    TaskResult::failure(
                        task.id, 
                        self.base.id, 
                        format!("Multi-modal analysis failed: {}", e),
                        start_time.elapsed().as_millis() as u64
                    )
                }
            }
        } else {
            // Fall back to standard agent behavior
            self.base.execute_task(task).await?
        };

        Ok(result)
    }

    async fn communicate(&mut self, envelope: MessageEnvelope) -> HiveResult<Option<MessageEnvelope>> {
        // Enhance communication with multi-modal capabilities
        match &envelope.payload {
            MessagePayload::Text(text) => {
                // Analyze the incoming message for modalities
                if let Ok(analysis) = self.analyze_multimodal_data(text).await {
                    let enhanced_response = format!(
                        "Multi-modal agent {} analyzed your message:\n\
                         Detected modalities: {:?}\n\
                         Quality score: {:.2}\n\
                         Insights: {}",
                        self.base.name,
                        analysis.detected_modalities,
                        analysis.overall_quality,
                        analysis.cross_modal_insights.join("; ")
                    );
                    
                    let response = MessageEnvelope::new_response(
                        &envelope,
                        self.base.id,
                        MessagePayload::Text(enhanced_response),
                    );
                    return Ok(Some(response));
                }
            }
            _ => {
                // For other message types, delegate to base agent
                return self.base.communicate(envelope).await;
            }
        }

        // Default response if analysis fails
        self.base.communicate(envelope).await
    }

    async fn request_response(
        &mut self,
        request: MessageEnvelope,
        timeout: std::time::Duration,
    ) -> HiveResult<MessageEnvelope> {
        self.base.request_response(request, timeout).await
    }

    async fn learn(&mut self, nlp_processor: &NLPProcessor) -> HiveResult<()> {
        // Enhanced learning that incorporates multi-modal analysis history
        self.base.learn(nlp_processor).await?;
        
        // Additional learning from analysis patterns
        if !self.analysis_history.is_empty() {
            let recent_analyses = self.analysis_history.iter().rev().take(5);
            let avg_quality: f64 = recent_analyses.clone().map(|a| a.overall_quality).sum::<f64>() / 5.0;
            
            // Adjust learning based on recent performance
            for capability in &mut self.base.capabilities {
                let adjustment = if avg_quality > 0.7 {
                    capability.learning_rate * 0.05
                } else {
                    -capability.learning_rate * 0.02
                };
                capability.proficiency = (capability.proficiency + adjustment).clamp(0.0, 1.0);
            }
        }
        
        Ok(())
    }

    async fn update_position(
        &mut self,
        swarm_center: (f64, f64),
        neighbors: &[Agent],
    ) -> HiveResult<()> {
        self.base.update_position(swarm_center, neighbors).await
    }
}