//! Semantic analysis for SATS v2
//! 
//! This module provides LLM-powered semantic analysis to verify that tests
//! actually test what they claim to test, and identify gaps in coverage.

use crate::types::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("LLM analysis failed: {0}")]
    LlmAnalysis(String),
    #[error("Claim analysis failed: {0}")]
    ClaimAnalysis(String),
    #[error("Test coverage analysis failed: {0}")]
    CoverageAnalysis(String),
    #[error("Gap analysis failed: {0}")]
    GapAnalysis(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Main semantic analyzer that orchestrates different analysis types
pub struct SemanticAnalyzer {
    claim_analyzer: Box<dyn ClaimAnalyzer>,
    coverage_analyzer: Box<dyn TestCoverageAnalyzer>,
    gap_analyzer: Box<dyn GapAnalyzer>,
    llm_client: LlmClient,
}

impl SemanticAnalyzer {
    pub fn new(
        claim_analyzer: Box<dyn ClaimAnalyzer>,
        coverage_analyzer: Box<dyn TestCoverageAnalyzer>,
        gap_analyzer: Box<dyn GapAnalyzer>,
        llm_client: LlmClient,
    ) -> Self {
        Self {
            claim_analyzer,
            coverage_analyzer,
            gap_analyzer,
            llm_client,
        }
    }

    /// Perform complete semantic verification of a claim against its tests
    pub async fn verify_claim_semantics(
        &self,
        claim: &Claim,
        test_suite: &TestSuite,
        execution_result: &ExecutionResult,
    ) -> Result<SemanticResult, SemanticError> {
        // Analyze the claim to understand what it actually means
        let claim_analysis = self.claim_analyzer.analyze_claim(claim).await?;
        
        // Analyze test coverage against the claim
        let coverage_analysis = self.coverage_analyzer
            .analyze_test_coverage(claim, test_suite, &claim_analysis)
            .await?;
        
        // Find gaps between claim and test coverage
        let gaps = self.gap_analyzer
            .find_semantic_gaps(claim, test_suite, &claim_analysis, &coverage_analysis)
            .await?;

        // Calculate overall coverage score
        let coverage_score = self.calculate_coverage_score(&coverage_analysis, &gaps)?;

        // Generate improvement suggestions
        let suggestions = self.generate_suggestions(&gaps, &coverage_analysis).await?;

        Ok(SemanticResult {
            claim_id: claim.id,
            test_suite_id: test_suite.id,
            coverage_score,
            gaps,
            verified_aspects: coverage_analysis.verified_aspects,
            suggestions,
            analyzed_at: chrono::Utc::now(),
        })
    }

    fn calculate_coverage_score(
        &self,
        coverage_analysis: &CoverageAnalysis,
        gaps: &[SemanticGap],
    ) -> Result<Confidence, SemanticError> {
        // Base score from coverage analysis
        let base_score = coverage_analysis.base_coverage_score;
        
        // Penalty for each gap type
        let gap_penalty = gaps.iter().map(|gap| match gap {
            SemanticGap::TestNameMismatch { .. } => 0.1,
            SemanticGap::MissingAssertion { .. } => 0.2,
            SemanticGap::UncoveredEdgeCase { .. } => 0.15,
            SemanticGap::IncorrectAssumption { .. } => 0.25,
        }).sum::<f64>();

        let final_score = (base_score - gap_penalty).max(0.0);
        
        Confidence::new(final_score)
            .map_err(|e| SemanticError::CoverageAnalysis(e.to_string()))
    }

    async fn generate_suggestions(
        &self,
        gaps: &[SemanticGap],
        _coverage_analysis: &CoverageAnalysis,
    ) -> Result<Vec<String>, SemanticError> {
        let mut suggestions = Vec::new();

        for gap in gaps {
            match gap {
                SemanticGap::TestNameMismatch { test_name, actual_behavior } => {
                    suggestions.push(format!(
                        "Rename test '{}' to better reflect actual behavior: '{}'",
                        test_name, actual_behavior
                    ));
                }
                SemanticGap::MissingAssertion { claim_aspect } => {
                    suggestions.push(format!(
                        "Add assertion to verify: {}",
                        claim_aspect
                    ));
                }
                SemanticGap::UncoveredEdgeCase { edge_case } => {
                    suggestions.push(format!(
                        "Add test case for edge case: {}",
                        edge_case
                    ));
                }
                SemanticGap::IncorrectAssumption { assumption, reality } => {
                    suggestions.push(format!(
                        "Fix assumption '{}' to match reality: '{}'",
                        assumption, reality
                    ));
                }
            }
        }

        Ok(suggestions)
    }
}

/// Analyzes claims to understand their semantic meaning
#[async_trait]
pub trait ClaimAnalyzer: Send + Sync {
    async fn analyze_claim(&self, claim: &Claim) -> Result<ClaimAnalysis, SemanticError>;
}

/// Analyzes test coverage against claim semantics
#[async_trait]
pub trait TestCoverageAnalyzer: Send + Sync {
    async fn analyze_test_coverage(
        &self,
        claim: &Claim,
        test_suite: &TestSuite,
        claim_analysis: &ClaimAnalysis,
    ) -> Result<CoverageAnalysis, SemanticError>;
}

/// Finds semantic gaps between claims and tests
#[async_trait]
pub trait GapAnalyzer: Send + Sync {
    async fn find_semantic_gaps(
        &self,
        claim: &Claim,
        test_suite: &TestSuite,
        claim_analysis: &ClaimAnalysis,
        coverage_analysis: &CoverageAnalysis,
    ) -> Result<Vec<SemanticGap>, SemanticError>;
}

/// Detailed analysis of a claim's semantic meaning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimAnalysis {
    pub claim_id: Id,
    pub semantic_aspects: Vec<SemanticAspect>,
    pub behavioral_expectations: Vec<BehavioralExpectation>,
    pub edge_cases: Vec<EdgeCase>,
    pub implicit_assumptions: Vec<String>,
    pub complexity_score: f64,
}

/// An aspect of behavior claimed by the statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAspect {
    pub aspect_type: AspectType,
    pub description: String,
    pub verifiable: bool,
    pub priority: u8, // 1-10
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectType {
    Functional,
    Performance,
    Security,
    UserExperience,
    DataIntegrity,
    ErrorHandling,
}

/// Expected behavior described in the claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralExpectation {
    pub description: String,
    pub inputs: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
}

/// Edge cases that should be considered for the claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCase {
    pub scenario: String,
    pub inputs: Vec<String>,
    pub expected_behavior: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Analysis of how well tests cover the claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAnalysis {
    pub base_coverage_score: f64,
    pub verified_aspects: Vec<String>,
    pub missing_aspects: Vec<String>,
    pub test_behavior_mapping: HashMap<Id, Vec<String>>, // test_case_id -> behaviors
    pub assertion_coverage: AssertionCoverage,
}

/// Analysis of assertion coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionCoverage {
    pub total_assertions: usize,
    pub meaningful_assertions: usize,
    pub weak_assertions: Vec<String>,
    pub missing_assertions: Vec<String>,
}

/// LLM-based claim analyzer
pub struct LlmClaimAnalyzer {
    llm_client: LlmClient,
}

impl LlmClaimAnalyzer {
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl ClaimAnalyzer for LlmClaimAnalyzer {
    async fn analyze_claim(&self, claim: &Claim) -> Result<ClaimAnalysis, SemanticError> {
        let prompt = format!(
            r#"Analyze this claim for semantic meaning and testing requirements:

Claim: "{}"
Type: {:?}
Source context: "{}"

Extract:
1. Semantic aspects - what behaviors/properties are being claimed
2. Behavioral expectations - inputs, outputs, pre/post conditions  
3. Edge cases that should be tested
4. Implicit assumptions being made
5. Complexity score (0-1)

Return as structured JSON."#,
            claim.statement,
            claim.claim_type,
            claim.source_excerpt
        );

        let response = self.llm_client.generate(&prompt).await
            .map_err(|e| SemanticError::LlmAnalysis(e.to_string()))?;

        // Parse response - simplified for example
        Ok(ClaimAnalysis {
            claim_id: claim.id,
            semantic_aspects: vec![SemanticAspect {
                aspect_type: AspectType::Functional,
                description: "Primary functionality".to_string(),
                verifiable: true,
                priority: 8,
            }],
            behavioral_expectations: vec![BehavioralExpectation {
                description: "Expected behavior from claim".to_string(),
                inputs: vec!["input parameter".to_string()],
                expected_outputs: vec!["expected result".to_string()],
                preconditions: vec!["system ready".to_string()],
                postconditions: vec!["state updated".to_string()],
            }],
            edge_cases: vec![EdgeCase {
                scenario: "Boundary condition".to_string(),
                inputs: vec!["edge input".to_string()],
                expected_behavior: "Graceful handling".to_string(),
                risk_level: RiskLevel::Medium,
            }],
            implicit_assumptions: vec!["System is initialized".to_string()],
            complexity_score: 0.6,
        })
    }
}

/// LLM-based test coverage analyzer
pub struct LlmTestCoverageAnalyzer {
    llm_client: LlmClient,
}

impl LlmTestCoverageAnalyzer {
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl TestCoverageAnalyzer for LlmTestCoverageAnalyzer {
    async fn analyze_test_coverage(
        &self,
        claim: &Claim,
        test_suite: &TestSuite,
        claim_analysis: &ClaimAnalysis,
    ) -> Result<CoverageAnalysis, SemanticError> {
        let prompt = format!(
            r#"Analyze how well these tests cover the claim:

Claim: "{}"
Claim aspects: {:?}

Tests:
{}

Determine:
1. Which aspects are adequately tested
2. Which aspects are missing or poorly tested  
3. Quality of assertions (meaningful vs weak)
4. Overall coverage score (0-1)

Return structured analysis."#,
            claim.statement,
            claim_analysis.semantic_aspects,
            test_suite.test_cases.iter()
                .map(|t| format!("- {}: {}", t.name, t.description))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let _response = self.llm_client.generate(&prompt).await
            .map_err(|e| SemanticError::LlmAnalysis(e.to_string()))?;

        // Simplified analysis for example
        Ok(CoverageAnalysis {
            base_coverage_score: 0.75,
            verified_aspects: vec!["Basic functionality".to_string()],
            missing_aspects: vec!["Error handling".to_string()],
            test_behavior_mapping: HashMap::new(),
            assertion_coverage: AssertionCoverage {
                total_assertions: test_suite.test_cases.len() * 2, // Estimate
                meaningful_assertions: test_suite.test_cases.len(),
                weak_assertions: vec!["Assertion needs strengthening".to_string()],
                missing_assertions: vec!["Should verify error conditions".to_string()],
            },
        })
    }
}

/// LLM-based gap analyzer
pub struct LlmGapAnalyzer {
    llm_client: LlmClient,
}

impl LlmGapAnalyzer {
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl GapAnalyzer for LlmGapAnalyzer {
    async fn find_semantic_gaps(
        &self,
        claim: &Claim,
        test_suite: &TestSuite,
        claim_analysis: &ClaimAnalysis,
        coverage_analysis: &CoverageAnalysis,
    ) -> Result<Vec<SemanticGap>, SemanticError> {
        let prompt = format!(
            r#"Find semantic gaps between the claim and its tests:

Claim: "{}"
Expected aspects: {:?}
Test coverage: {:?}

Find gaps like:
1. Test names that don't match what they actually test
2. Missing assertions for claimed behavior
3. Uncovered edge cases
4. Incorrect assumptions in tests

Return specific gaps found."#,
            claim.statement,
            claim_analysis.semantic_aspects,
            coverage_analysis.verified_aspects
        );

        let _response = self.llm_client.generate(&prompt).await
            .map_err(|e| SemanticError::LlmAnalysis(e.to_string()))?;

        // Simplified gap detection for example
        let mut gaps = Vec::new();

        // Check for missing assertions based on coverage analysis
        for missing_assertion in &coverage_analysis.assertion_coverage.missing_assertions {
            gaps.push(SemanticGap::MissingAssertion {
                claim_aspect: missing_assertion.clone(),
            });
        }

        // Check for uncovered edge cases
        for edge_case in &claim_analysis.edge_cases {
            if edge_case.risk_level != RiskLevel::Low {
                gaps.push(SemanticGap::UncoveredEdgeCase {
                    edge_case: edge_case.scenario.clone(),
                });
            }
        }

        Ok(gaps)
    }
}

/// LLM client for semantic analysis
#[derive(Clone)]
pub struct LlmClient {
    endpoint: Option<String>,
    model: String,
}

impl LlmClient {
    pub fn new(endpoint: Option<String>, model: String) -> Self {
        Self { endpoint, model }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // This would make actual LLM API calls
        // For now, return a placeholder response
        let _ = prompt; // Use the prompt to avoid warnings
        Ok("Generated semantic analysis response".to_string())
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self {
            endpoint: None,
            model: "claude-3-sonnet".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_claim_analyzer() {
        let llm_client = LlmClient::default();
        let analyzer = LlmClaimAnalyzer::new(llm_client);

        let claim = Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: uuid::Uuid::new_v4(),
            statement: "User authentication works correctly".to_string(),
            claim_type: ClaimType::Security,
            extraction_confidence: Confidence::new(0.9).unwrap(),
            source_excerpt: "// Implement secure authentication".to_string(),
            extracted_at: chrono::Utc::now(),
            verification_chain: None,
        };

        let analysis = analyzer.analyze_claim(&claim).await.unwrap();
        assert_eq!(analysis.claim_id, claim.id);
        assert!(!analysis.semantic_aspects.is_empty());
        assert!(!analysis.behavioral_expectations.is_empty());
    }

    #[tokio::test]
    async fn test_coverage_score_calculation() {
        let analyzer = SemanticAnalyzer::new(
            Box::new(LlmClaimAnalyzer::new(LlmClient::default())),
            Box::new(LlmTestCoverageAnalyzer::new(LlmClient::default())),
            Box::new(LlmGapAnalyzer::new(LlmClient::default())),
            LlmClient::default(),
        );

        let coverage_analysis = CoverageAnalysis {
            base_coverage_score: 0.8,
            verified_aspects: vec!["aspect1".to_string()],
            missing_aspects: vec![],
            test_behavior_mapping: HashMap::new(),
            assertion_coverage: AssertionCoverage {
                total_assertions: 5,
                meaningful_assertions: 4,
                weak_assertions: vec![],
                missing_assertions: vec![],
            },
        };

        let gaps = vec![
            SemanticGap::MissingAssertion {
                claim_aspect: "error handling".to_string(),
            }
        ];

        let score = analyzer.calculate_coverage_score(&coverage_analysis, &gaps).unwrap();
        assert_eq!(score.value(), 0.6); // 0.8 - 0.2 penalty for missing assertion
    }

    #[test]
    fn test_edge_case_risk_levels() {
        let low_risk = EdgeCase {
            scenario: "Normal input".to_string(),
            inputs: vec!["valid input".to_string()],
            expected_behavior: "Success".to_string(),
            risk_level: RiskLevel::Low,
        };

        let high_risk = EdgeCase {
            scenario: "Security boundary".to_string(),
            inputs: vec!["malicious input".to_string()],
            expected_behavior: "Reject safely".to_string(),
            risk_level: RiskLevel::Critical,
        };

        assert_eq!(low_risk.risk_level, RiskLevel::Low);
        assert_eq!(high_risk.risk_level, RiskLevel::Critical);
    }
}