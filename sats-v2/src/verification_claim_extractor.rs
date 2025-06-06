//! Verification-Focused Claim Extractor for SATS v2
//! 
//! This module implements an LLM-based claim extractor specifically designed for
//! SATS v2's verification chain approach. Unlike traditional claim extraction,
//! this extractor focuses on claims that can be verified through implementation,
//! testing, and execution.

use crate::types::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};

/// Client implementations for LLM interaction
use client_implementations::claude::ClaudeClient;
use client_implementations::client::{QueryResolver, RetryConfig};

/// Import analysis interfaces from sats-core (for compatibility)
use sats_core::analysis::{ClaimExtractor, ClaimExtractionResult, AnalysisError};
use sats_core::types::AnalysisConfig;

/// Our own trait for verification-focused claim extraction using SATS v2 types
#[async_trait]
pub trait VerificationClaimExtractor: Send + Sync {
    /// Extract verification-focused claims from an artifact using our own types
    async fn extract_verification_claims(
        &self,
        artifact: &Artifact,
    ) -> Result<VerificationExtractionResult, VerificationError>;
}

/// Result of verification claim extraction using our own types
#[derive(Debug, Clone)]
pub struct VerificationExtractionResult {
    pub claims: Vec<Claim>,
    pub processing_time_ms: u64,
    pub model_used: String,
    pub verification_metadata: HashMap<String, String>,
}

/// Errors specific to verification claim extraction
#[derive(thiserror::Error, Debug)]
pub enum VerificationError {
    #[error("LLM query failed: {0}")]
    LlmQuery(String),
    #[error("Failed to parse response: {0}")]
    ResponseParsing(String),
    #[error("Invalid confidence value: {0}")]
    InvalidConfidence(String),
    #[error("Verification analysis failed: {0}")]
    AnalysisFailed(String),
}

/// Structured output format for verification-focused claim extraction
#[derive(Debug, Serialize, Deserialize)]
struct VerificationClaimOutput {
    claims: Vec<VerificationClaim>,
    confidence: f64,
    verification_context: VerificationContext,
    extraction_metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerificationClaim {
    statement: String,
    confidence: f64,
    claim_type: String,
    verifiability: VerifiabilityInfo,
    implementation_requirements: Option<RequirementAnalysis>,
    source_excerpt: String,
    reasoning: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifiabilityInfo {
    verification_method: String, // "execution", "compilation", "static_analysis", "manual_review"
    complexity_score: f64, // 0.0-1.0, how complex to verify
    automation_feasible: bool,
    required_artifacts: Vec<String>, // "implementation", "tests", "documentation"
    potential_work_items: Vec<String>, // What work might be needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequirementAnalysis {
    interface_requirements: Vec<String>,
    behavior_requirements: Vec<String>,
    quality_requirements: Vec<String>,
    test_requirements: Vec<String>,
    implementation_hints: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerificationContext {
    artifact_analysis: String,
    related_claims_context: Vec<String>,
    verification_chain_suggestions: Vec<String>,
    gap_indicators: Vec<String>,
}

/// LLM-powered claim extractor focused on verification chains
pub struct ClaudeVerificationExtractor {
    query_resolver: QueryResolver<ClaudeClient>,
}

impl ClaudeVerificationExtractor {
    pub fn new(api_key: String) -> Self {
        let client = ClaudeClient::new(api_key)
            .with_caching(true); // Enable caching for claim extraction
        let config = RetryConfig::default();
        let query_resolver = QueryResolver::new(client, config);
        
        Self { query_resolver }
    }

    /// Get standardized base instructions for verification-focused claim extraction
    fn get_base_verification_instructions(&self) -> String {
        r#"You are an expert software analyst specializing in extracting VERIFIABLE claims from software artifacts.

SATS v2 VERIFICATION FOCUS:
Your task is to extract claims that can be systematically verified through implementation chains:
Claim → Requirements → Implementation → Tests → Execution → Verification

VERIFIABLE CLAIM DEFINITION:
A verifiable claim is a statement that can be objectively validated through:
1. Code implementation existence/correctness
2. Test execution and results  
3. Static analysis or compilation
4. Performance measurement
5. Security analysis

EXTRACTION PRINCIPLES:
- Focus on claims that generate actionable work items
- Identify missing links in implementation chains
- Extract claims that can drive automated verification
- Prioritize claims that impact system behavior or quality

CLAIM TYPES FOR VERIFICATION:
1. FUNCTIONAL: "System implements X functionality"
2. BEHAVIORAL: "System behaves in Y way under Z conditions"  
3. QUALITY: "System meets performance/security/reliability standards"
4. TESTING: "System has adequate test coverage for X"
5. INTEGRATION: "System integrates correctly with external dependencies"
6. API_CONTRACT: "System provides specific interface guarantees"

VERIFIABILITY ASSESSMENT:
For each claim, evaluate:
- Verification method (execution, compilation, static analysis, etc.)
- Complexity to verify (0.0-1.0 scale)
- Whether automation is feasible
- What artifacts are needed (implementation, tests, docs)
- What work items might be required

OUTPUT FORMAT:
Respond in JSON using this exact structure:
{
  "claims": [
    {
      "statement": "Clear, specific claim that can be verified",
      "confidence": 0.0-1.0,
      "claim_type": "functional|behavioral|quality|testing|integration|api_contract",
      "verifiability": {
        "verification_method": "execution|compilation|static_analysis|performance_test|security_scan|manual_review",
        "complexity_score": 0.0-1.0,
        "automation_feasible": true/false,
        "required_artifacts": ["implementation", "tests", "documentation", "configuration"],
        "potential_work_items": ["implement_feature", "write_tests", "fix_bugs", "optimize_performance"]
      },
      "implementation_requirements": {
        "interface_requirements": ["specific function signatures needed"],
        "behavior_requirements": ["specific behaviors that must be implemented"],
        "quality_requirements": ["performance, security, reliability requirements"],
        "test_requirements": ["specific tests that must exist"],
        "implementation_hints": ["guidance for implementation"]
      },
      "source_excerpt": "Specific part of artifact this came from",
      "reasoning": "Why this is a verifiable claim and how to verify it"
    }
  ],
  "confidence": 0.0-1.0,
  "verification_context": {
    "artifact_analysis": "Overall analysis of the artifact's verification potential",
    "related_claims_context": ["how claims relate to each other"],
    "verification_chain_suggestions": ["suggested verification approaches"],
    "gap_indicators": ["potential gaps or missing pieces identified"]
  },
  "extraction_metadata": {
    "artifact_length": "length_value",
    "extraction_strategy": "verification_focused",
    "complexity_assessment": "low|medium|high"
  }
}"#.to_string()
    }

    /// Generate artifact-specific verification prompts
    fn build_verification_prompt(&self, artifact: &sats_core::types::Artifact) -> String {
        let base_instructions = self.get_base_verification_instructions();
        
        let artifact_specific = match artifact.artifact_type {
            sats_core::types::ArtifactType::Code => self.get_code_verification_prompt(),
            sats_core::types::ArtifactType::Test => self.get_test_verification_prompt(),
            sats_core::types::ArtifactType::Documentation => self.get_documentation_verification_prompt(),
            sats_core::types::ArtifactType::Ticket => self.get_ticket_verification_prompt(),
            sats_core::types::ArtifactType::Commit => self.get_commit_verification_prompt(),
            _ => "Extract verifiable claims from this artifact, focusing on implementation chain verification.".to_string(),
        };

        // Structure: Base instructions (cacheable) + Artifact-specific instructions (cacheable) + Variable data
        format!(
            "{}

ARTIFACT-SPECIFIC VERIFICATION GUIDANCE:
{}

--- BEGIN VERIFICATION ANALYSIS ---

Artifact Type: {:?}
Location: {}

Content:
```
{}
```

VERIFICATION ANALYSIS INSTRUCTIONS:
1. Extract claims that can be verified through execution or testing
2. Identify missing implementation gaps that need work items
3. Focus on claims that drive concrete verification steps
4. Consider the full implementation chain for each claim
5. Assess automation potential for verification

Verification Metadata:
- Artifact Length: {} characters
- Strategy: {:?}
- Expected Claims: 3-10 high-quality verifiable claims",
            base_instructions,
            artifact_specific,
            artifact.artifact_type,
            artifact.location.display(),
            artifact.content,
            artifact.content.len(),
            artifact.artifact_type
        )
    }

    fn get_code_verification_prompt(&self) -> String {
        r#"
CODE VERIFICATION FOCUS:
Extract claims about implementation that can be verified through:
- Compilation and static analysis
- Unit and integration test execution
- Performance benchmarking
- Security scanning
- Code coverage analysis

Look for verifiable claims like:
- "Function X implements algorithm Y correctly"
- "Error handling covers all specified edge cases"
- "Performance meets specified latency requirements"
- "Security validation prevents specified attack vectors"
- "Interface contract matches API specification"

For each claim, identify:
- Required test cases for verification
- Performance/security benchmarks needed
- Integration points that need validation
- Potential implementation gaps or technical debt
"#.to_string()
    }

    fn get_test_verification_prompt(&self) -> String {
        r#"
TEST VERIFICATION FOCUS:
Extract claims about test coverage and verification capabilities:
- What behaviors are actually tested vs claimed to be tested
- Test quality and adequacy for verification
- Missing test scenarios that impact verification
- Test execution requirements and constraints

Look for verifiable claims like:
- "Test suite covers all critical user flows"
- "Error conditions are adequately tested"
- "Performance tests validate system requirements"
- "Security tests cover identified threat vectors"
- "Integration tests verify external dependencies"

Identify verification gaps:
- Behaviors claimed but not tested
- Tests that don't actually verify their claims
- Missing edge case coverage
- Inadequate assertion depth
"#.to_string()
    }

    fn get_documentation_verification_prompt(&self) -> String {
        r#"
DOCUMENTATION VERIFICATION FOCUS:
Extract claims about system capabilities that can be verified against implementation:
- API behavior descriptions that can be tested
- Performance claims that can be benchmarked
- Security properties that can be validated
- Configuration options that can be verified

Look for verifiable claims like:
- "API endpoint returns specified response format"
- "System handles X concurrent users with Y response time"
- "Authentication prevents unauthorized access"
- "Configuration option Z affects behavior in W way"

Identify verification opportunities:
- Documented behaviors that need test validation
- Performance claims requiring benchmarks
- Security assertions needing verification
- Examples that can become automated tests
"#.to_string()
    }

    fn get_ticket_verification_prompt(&self) -> String {
        r#"
TICKET/REQUIREMENT VERIFICATION FOCUS:
Extract requirements that can be verified through implementation and testing:
- Acceptance criteria that can be automated
- Functional requirements with measurable outcomes
- Quality requirements with specific metrics
- Integration requirements with testable interfaces

Look for verifiable claims like:
- "User can complete workflow X in Y steps"
- "System processes Z requests per second"
- "Data is encrypted using specified algorithm"
- "Integration with service W follows protocol V"

For each requirement, identify:
- Specific test scenarios needed for verification
- Implementation components required
- Quality metrics for validation
- Integration points requiring verification
"#.to_string()
    }

    fn get_commit_verification_prompt(&self) -> String {
        r#"
COMMIT VERIFICATION FOCUS:
Extract claims about changes that can be verified through testing:
- Bug fixes that can be regression tested
- Features that can be functionally tested
- Performance improvements that can be benchmarked
- Security fixes that can be validated

Look for verifiable claims like:
- "Bug fix prevents error condition X"
- "New feature implements capability Y"
- "Optimization improves performance by Z%"
- "Security patch closes vulnerability W"

Identify verification needs:
- Test cases needed to verify the change
- Regression tests to prevent re-occurrence
- Performance benchmarks to validate improvements
- Security tests to confirm fixes
"#.to_string()
    }

    /// Convert extracted claims to our own types
    fn convert_to_sats_v2_claims(
        &self,
        output: VerificationClaimOutput,
        artifact: &Artifact,
    ) -> Result<Vec<Claim>, Box<dyn std::error::Error>> {
        let mut claims = Vec::new();

        for extracted_claim in output.claims {
            let confidence = Confidence::new(extracted_claim.confidence)
                .map_err(|e| format!("Invalid confidence: {}", e))?;

            let claim_type = match extracted_claim.claim_type.to_lowercase().as_str() {
                "functional" => ClaimType::Functional,
                "behavioral" => ClaimType::Behavior,
                "quality" => ClaimType::Performance, // Map quality to performance for now
                "testing" => ClaimType::Testing,
                "integration" => ClaimType::Functional, // Map integration to functional
                "api_contract" => ClaimType::Functional, // Map API contract to functional
                _ => {
                    warn!("Unknown claim type '{}', defaulting to Functional", extracted_claim.claim_type);
                    ClaimType::Functional
                }
            };

            // Create verification chain structure based on actual VerificationChain type
            let requirements = vec![]; // Extract requirements if needed
            let verification_chain = Some(VerificationChain {
                claim_id: uuid::Uuid::new_v4(), // Will be set after claim creation
                status: ChainStatus::NotStarted,
                links: vec![
                    ChainLink::Requirements(requirements),
                ],
                created_at: chrono::Utc::now(),
                last_verified_at: None,
                missing_links: vec![
                    WorkItemType::ImplementRequirements,
                    WorkItemType::CreateTests,
                ],
            });

            let claim = Claim {
                id: uuid::Uuid::new_v4(),
                artifact_id: artifact.id,
                statement: extracted_claim.statement,
                extraction_confidence: confidence,
                claim_type,
                source_excerpt: extracted_claim.source_excerpt,
                extracted_at: chrono::Utc::now(),
                verification_chain,
            };

            claims.push(claim);
        }

        Ok(claims)
    }

    /// Convert extracted claims to sats-core representation
    fn convert_to_internal_claims(
        &self,
        output: VerificationClaimOutput,
        artifact: &sats_core::types::Artifact,
    ) -> Result<Vec<sats_core::types::Claim>, Box<dyn std::error::Error>> {
        let mut claims = Vec::new();

        for extracted_claim in output.claims {
            let confidence = sats_core::types::Confidence::new(extracted_claim.confidence)
                .map_err(|e| format!("Invalid confidence: {}", e))?;

            let claim_type = match extracted_claim.claim_type.to_lowercase().as_str() {
                "functional" => sats_core::types::ClaimType::Functional,
                "behavioral" => sats_core::types::ClaimType::Behavior,
                "quality" => sats_core::types::ClaimType::Performance, // Map quality to performance for now
                "testing" => sats_core::types::ClaimType::Testing,
                "integration" => sats_core::types::ClaimType::Functional, // Map integration to functional
                "api_contract" => sats_core::types::ClaimType::Functional, // Map API contract to functional
                _ => {
                    warn!("Unknown claim type '{}', defaulting to Functional", extracted_claim.claim_type);
                    sats_core::types::ClaimType::Functional
                }
            };

            let claim = sats_core::types::Claim {
                id: uuid::Uuid::new_v4(),
                artifact_id: artifact.id,
                statement: extracted_claim.statement,
                extraction_confidence: confidence,
                claim_type,
                source_excerpt: extracted_claim.source_excerpt,
                extracted_at: chrono::Utc::now(),
            };

            claims.push(claim);
        }

        Ok(claims)
    }

    /// Extract requirements analysis from verification claim (for future use)
    /// This could be used to generate work items or verification chains
    fn _extract_requirements_analysis(&self, claim: &VerificationClaim) -> Option<RequirementAnalysis> {
        claim.implementation_requirements.clone()
    }

    /// Generate potential work items for a claim (placeholder for future implementation)
    /// TODO: Implement work item generation based on verification chain analysis
    pub async fn analyze_for_work_items(&self, claim: &Claim) -> Result<Vec<WorkItem>, Box<dyn std::error::Error>> {
        // For now, return empty work items list
        // This will be implemented once the work item types are properly defined
        Ok(vec![])
    }
}

/// Implementation of our own trait using SATS v2 types
#[async_trait]
impl VerificationClaimExtractor for ClaudeVerificationExtractor {
    async fn extract_verification_claims(
        &self,
        artifact: &Artifact,
    ) -> Result<VerificationExtractionResult, VerificationError> {
        info!("Extracting verification-focused claims from {}", artifact.location.display());
        
        let start_time = std::time::Instant::now();
        let prompt = self.build_verification_prompt_v2(artifact);
        
        debug!("Generated verification prompt (length: {})", prompt.len());
        
        // Query Claude for structured claim extraction using the same pattern as sats-core
        let (output, mut verification_metadata) = match self.query_resolver.query::<VerificationClaimOutput>(prompt).await {
            Ok(output) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                info!("Claude extracted {} verification claims in {}ms", output.claims.len(), processing_time);
                (output, HashMap::new())
            }
            Err(e) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                error!("Claude query failed: {}, creating fallback claim with 0 confidence", e);
                
                return Err(VerificationError::LlmQuery(e.to_string()));
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Convert to our claim format
        let claims = self.convert_to_sats_v2_claims(output, artifact)
            .map_err(|e| VerificationError::ResponseParsing(format!("Failed to convert claims: {}", e)))?;

        // Merge verification metadata
        verification_metadata.insert("extraction_type".to_string(), "verification_focused".to_string());
        verification_metadata.insert("final_processing_time_ms".to_string(), processing_time.to_string());

        Ok(VerificationExtractionResult {
            claims,
            processing_time_ms: processing_time,
            model_used: "claude-verification-optimized".to_string(),
            verification_metadata,
        })
    }
}

impl ClaudeVerificationExtractor {
    /// Generate artifact-specific verification prompts for our own types
    fn build_verification_prompt_v2(&self, artifact: &Artifact) -> String {
        let base_instructions = self.get_base_verification_instructions();
        
        let artifact_specific = match artifact.artifact_type {
            ArtifactType::Code => self.get_code_verification_prompt(),
            ArtifactType::Test => self.get_test_verification_prompt(),
            ArtifactType::Documentation => self.get_documentation_verification_prompt(),
            ArtifactType::Ticket => self.get_ticket_verification_prompt(),
            ArtifactType::Commit => self.get_commit_verification_prompt(),
            _ => "Extract verifiable claims from this artifact, focusing on implementation chain verification.".to_string(),
        };

        // Structure: Base instructions (cacheable) + Artifact-specific instructions (cacheable) + Variable data
        format!(
            "{}

ARTIFACT-SPECIFIC VERIFICATION GUIDANCE:
{}

--- BEGIN VERIFICATION ANALYSIS ---

Artifact Type: {:?}
Location: {}

Content:
```
{}
```

VERIFICATION ANALYSIS INSTRUCTIONS:
1. Extract claims that can be verified through execution or testing
2. Identify missing implementation gaps that need work items
3. Focus on claims that drive concrete verification steps
4. Consider the full implementation chain for each claim
5. Assess automation potential for verification

Verification Metadata:
- Artifact Length: {} characters
- Strategy: {:?}
- Expected Claims: 3-10 high-quality verifiable claims",
            base_instructions,
            artifact_specific,
            artifact.artifact_type,
            artifact.location.display(),
            artifact.content,
            artifact.content.len(),
            artifact.artifact_type
        )
    }
}

/// Implementation of sats-core trait for compatibility
#[async_trait]
impl ClaimExtractor for ClaudeVerificationExtractor {
    async fn extract_claims(
        &self,
        artifact: &sats_core::types::Artifact,
        _config: &AnalysisConfig,
    ) -> Result<ClaimExtractionResult, AnalysisError> {
        info!("Extracting verification-focused claims from {}", artifact.location.display());
        
        let start_time = std::time::Instant::now();
        let prompt = self.build_verification_prompt(artifact);
        
        debug!("Generated verification prompt (length: {})", prompt.len());
        
        // Query Claude for structured claim extraction using the same pattern as sats-core
        let (output, mut extraction_metadata) = match self.query_resolver.query::<VerificationClaimOutput>(prompt).await {
            Ok(output) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                info!("Claude extracted {} verification claims in {}ms", output.claims.len(), processing_time);
                (output, HashMap::new())
            }
            Err(e) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                error!("Claude query failed: {}, creating fallback claim with 0 confidence", e);
                
                // Create a fallback claim with 0 confidence for failed queries
                let fallback_output = VerificationClaimOutput {
                    claims: vec![VerificationClaim {
                        statement: format!("Failed to extract verification claims from {}: {}", 
                                         artifact.location.display(), e),
                        confidence: 0.0,
                        claim_type: "functional".to_string(),
                        verifiability: VerifiabilityInfo {
                            verification_method: "manual_review".to_string(),
                            complexity_score: 1.0,
                            automation_feasible: false,
                            required_artifacts: vec!["implementation".to_string()],
                            potential_work_items: vec!["debug_extraction_failure".to_string()],
                        },
                        implementation_requirements: None,
                        source_excerpt: artifact.content.chars().take(200).collect::<String>(),
                        reasoning: "Query failed, created fallback claim".to_string(),
                    }],
                    confidence: 0.0,
                    verification_context: VerificationContext {
                        artifact_analysis: "Extraction failed".to_string(),
                        related_claims_context: vec![],
                        verification_chain_suggestions: vec![],
                        gap_indicators: vec!["extraction_failure".to_string()],
                    },
                    extraction_metadata: HashMap::from([
                        ("extraction_failed".to_string(), "true".to_string()),
                        ("error".to_string(), e.to_string()),
                        ("processing_time_ms".to_string(), processing_time.to_string()),
                    ]),
                };
                
                warn!("Created fallback verification claim due to extraction failure");
                (fallback_output, HashMap::from([
                    ("extraction_failed".to_string(), "true".to_string()),
                    ("original_error".to_string(), e.to_string()),
                ]))
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Convert to internal claim format
        let claims = self.convert_to_internal_claims(output, artifact)
            .map_err(|e| AnalysisError::ClaimExtraction(format!("Failed to convert claims: {}", e)))?;

        // Merge extraction metadata
        extraction_metadata.insert("extraction_type".to_string(), "verification_focused".to_string());
        extraction_metadata.insert("final_processing_time_ms".to_string(), processing_time.to_string());

        Ok(ClaimExtractionResult {
            claims,
            processing_time_ms: processing_time,
            model_used: "claude-verification-optimized".to_string(),
            extraction_metadata,
        })
    }

    fn can_handle(&self, _artifact_type: &sats_core::types::ArtifactType) -> bool {
        true // Can handle all artifact types with verification focus
    }

    fn recommended_confidence_threshold(&self, artifact_type: &sats_core::types::ArtifactType) -> sats_core::types::Confidence {
        let threshold = match artifact_type {
            sats_core::types::ArtifactType::Code => 0.8,        // Code claims should be highly confident
            sats_core::types::ArtifactType::Test => 0.9,        // Test claims are usually explicit
            sats_core::types::ArtifactType::Documentation => 0.7, // Docs can be ambiguous
            sats_core::types::ArtifactType::Ticket => 0.9,      // Requirements should be clear
            sats_core::types::ArtifactType::Commit => 0.6,      // Commit messages vary in quality
            _ => 0.7,
        };
        sats_core::types::Confidence::new(threshold).unwrap()
    }
}