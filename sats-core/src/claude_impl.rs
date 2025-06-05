use crate::types::*;
use crate::analysis::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};

/// Claude client from client-implementations
use client_implementations::claude::ClaudeClient;
use client_implementations::client::{LowLevelClient, QueryResolver, RetryConfig};

/// Structured output format for claim extraction
#[derive(Debug, Serialize, Deserialize)]
struct ClaimExtractionOutput {
    claims: Vec<ExtractedClaim>,
    confidence: f64,
    extraction_metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExtractedClaim {
    statement: String,
    confidence: f64,
    claim_type: String,
    source_excerpt: String,
    reasoning: String,
}

/// Structured output for alignment checking
#[derive(Debug, Serialize, Deserialize)]
struct AlignmentOutput {
    alignment_score: f64,
    explanation: String,
    evidence_points: Vec<EvidencePointOutput>,
    dimensions: AlignmentDimensionsOutput,
}

#[derive(Debug, Serialize, Deserialize)]
struct EvidencePointOutput {
    excerpt: String,
    evidence_type: String,
    strength: f64,
    full_text: Option<String>,
    location: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AlignmentDimensionsOutput {
    semantic_alignment: f64,
    functional_alignment: f64,
    behavioral_alignment: f64,
    structural_alignment: f64,
    temporal_alignment: f64,
}

/// Claude-powered claim extractor that uses structured prompts
pub struct ClaudeClaimExtractor {
    query_resolver: QueryResolver<ClaudeClient>,
}

impl ClaudeClaimExtractor {
    pub fn new(api_key: String) -> Self {
        let client = ClaudeClient::new(api_key);
        let config = RetryConfig::default();
        let query_resolver = QueryResolver::new(client, config);
        
        Self { query_resolver }
    }

    /// Generate artifact-specific extraction prompt
    fn build_extraction_prompt(&self, artifact: &Artifact) -> String {
        let base_instructions = r#"
You are an expert software analyst tasked with extracting claims from software artifacts.

A CLAIM is any explicit or implicit statement about:
- What the system does or should do (functional claims)
- How the system behaves (behavioral claims)
- System architecture or structure (structural claims)
- Performance characteristics (performance claims)
- Security properties (security claims)
- Requirements or specifications (requirement claims)
- Testing coverage or validation (testing claims)

Extract ALL meaningful claims from the artifact, including:
- Explicit statements in comments or documentation
- Implicit claims from function/class names
- Behavioral implications from code logic
- Requirements implied by test cases
- Promises made by API signatures

For each claim:
1. State it clearly and specifically
2. Assign a confidence score (0.0-1.0) for how certain you are this claim is being made
3. Classify the claim type
4. Include the source excerpt that supports this claim
5. Provide brief reasoning for why this is a claim

Respond in JSON format only.
"#;

        let artifact_specific = match artifact.artifact_type {
            ArtifactType::Code => self.get_code_extraction_prompt(),
            ArtifactType::Test => self.get_test_extraction_prompt(),
            ArtifactType::Documentation => self.get_documentation_extraction_prompt(),
            ArtifactType::Ticket => self.get_ticket_extraction_prompt(),
            ArtifactType::Commit => self.get_commit_extraction_prompt(),
            _ => "Extract any claims found in this artifact.".to_string(),
        };

        format!(
            "{}\n\n{}\n\nArtifact Type: {:?}\nLocation: {}\n\nContent:\n```\n{}\n```\n\nExtract claims in this JSON format:\n{{\n  \"claims\": [\n    {{\n      \"statement\": \"Clear description of the claim\",\n      \"confidence\": 0.0-1.0,\n      \"claim_type\": \"functional|behavioral|structural|performance|security|requirement|testing\",\n      \"source_excerpt\": \"Specific part of artifact this came from\",\n      \"reasoning\": \"Why this constitutes a claim\"\n    }}\n  ],\n  \"confidence\": 0.0-1.0,\n  \"extraction_metadata\": {{\n    \"artifact_length\": \"{}\",\n    \"extraction_strategy\": \"{:?}\"\n  }}\n}}",
            base_instructions,
            artifact_specific,
            artifact.artifact_type,
            artifact.location.display(),
            artifact.content,
            artifact.content.len(),
            artifact.artifact_type
        )
    }

    fn get_code_extraction_prompt(&self) -> String {
        r#"
FOCUS FOR CODE ARTIFACTS:
- Function/method names and what they promise to do
- Class names and the abstractions they represent
- Comments describing behavior or purpose
- Error handling patterns and what they protect against
- API contracts implied by parameter types and return values
- Architectural patterns evident in the code structure
- Performance implications of algorithms used
- Security measures implemented (validation, sanitization, etc.)

Look for claims like:
- "This function handles user authentication"
- "This class manages database connections"
- "This method validates input parameters"
- "This code implements caching for performance"
"#.to_string()
    }

    fn get_test_extraction_prompt(&self) -> String {
        r#"
FOCUS FOR TEST ARTIFACTS:
- Test method names and what behavior they claim to verify
- Assertions and what they validate
- Test setup/teardown and what system state they assume
- Mock objects and what interfaces they simulate
- Test data and what scenarios they represent
- Comments describing test intent or coverage
- TODO comments about missing test coverage

Look for claims like:
- "This test verifies user login functionality"
- "This test checks error handling for invalid input"
- "This test ensures database transactions are atomic"
- "This test validates API response format"
"#.to_string()
    }

    fn get_documentation_extraction_prompt(&self) -> String {
        r#"
FOCUS FOR DOCUMENTATION ARTIFACTS:
- Feature descriptions and capabilities
- API endpoint descriptions and their behavior
- Configuration options and their effects
- Installation/setup requirements
- Usage examples and what they demonstrate
- Architecture diagrams and system boundaries
- Performance characteristics mentioned
- Security considerations documented

Look for claims like:
- "The API supports OAuth2 authentication"
- "The system can handle 1000 concurrent users"
- "Data is encrypted at rest"
- "The service provides real-time notifications"
"#.to_string()
    }

    fn get_ticket_extraction_prompt(&self) -> String {
        r#"
FOCUS FOR TICKET/REQUIREMENT ARTIFACTS:
- User stories and what capabilities they require
- Acceptance criteria and what must be satisfied
- Business requirements and their objectives
- Technical constraints mentioned
- Performance requirements specified
- Security requirements stated
- Integration requirements with other systems

Look for claims like:
- "Users must be able to reset their passwords"
- "The system must respond within 200ms"
- "All data must be backed up daily"
- "The API must support rate limiting"
"#.to_string()
    }

    fn get_commit_extraction_prompt(&self) -> String {
        r#"
FOCUS FOR COMMIT ARTIFACTS:
- Changes described in commit message
- Bug fixes and what issues they resolve
- Features added and their capabilities
- Refactoring changes and their impact
- Performance improvements claimed
- Security fixes implemented

Look for claims like:
- "Added user authentication feature"
- "Fixed memory leak in data processing"
- "Improved API response time by 50%"
- "Enhanced security for password handling"
"#.to_string()
    }
}

#[async_trait]
impl ClaimExtractor for ClaudeClaimExtractor {
    async fn extract_claims(
        &self,
        artifact: &Artifact,
        _config: &AnalysisConfig,
    ) -> Result<ClaimExtractionResult, AnalysisError> {
        info!("Extracting claims from {} using Claude", artifact.location.display());
        
        let start_time = std::time::Instant::now();
        let prompt = self.build_extraction_prompt(artifact);
        
        debug!("Generated prompt for artifact extraction (length: {})", prompt.len());
        
        // Query Claude with structured output
        let (output, mut extraction_metadata) = match self.query_resolver.query::<ClaimExtractionOutput>(prompt).await {
            Ok(output) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                info!("Claude extracted {} claims in {}ms", output.claims.len(), processing_time);
                (output, HashMap::new())
            }
            Err(e) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                error!("Claude query failed: {}, creating fallback claim with 0 confidence", e);
                
                // Create a fallback claim with 0 confidence for failed deserializations
                let fallback_output = ClaimExtractionOutput {
                    claims: vec![ExtractedClaim {
                        statement: format!("Failed to extract structured claims from {}: {}", 
                                         artifact.location.display(), e),
                        confidence: 0.0,
                        claim_type: "functional".to_string(),
                        source_excerpt: artifact.content.chars().take(200).collect::<String>(),
                        reasoning: "Deserialization failed, created fallback claim".to_string(),
                    }],
                    confidence: 0.0,
                    extraction_metadata: HashMap::from([
                        ("extraction_failed".to_string(), "true".to_string()),
                        ("error".to_string(), e.to_string()),
                        ("processing_time_ms".to_string(), processing_time.to_string()),
                    ]),
                };
                
                warn!("Created fallback claim due to extraction failure");
                (fallback_output, HashMap::from([
                    ("extraction_failed".to_string(), "true".to_string()),
                    ("original_error".to_string(), e.to_string()),
                ]))
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Convert to internal claim format
        let mut claims = Vec::new();
        for extracted_claim in output.claims {
            let confidence = Confidence::new(extracted_claim.confidence)
                .map_err(|e| AnalysisError::Configuration(format!("Invalid confidence: {}", e)))?;
            
            let claim_type = match extracted_claim.claim_type.to_lowercase().as_str() {
                "functional" => ClaimType::Functional,
                "behavioral" => ClaimType::Behavior,
                "structural" => ClaimType::Structure,
                "performance" => ClaimType::Performance,
                "security" => ClaimType::Security,
                "requirement" => ClaimType::Requirement,
                "testing" => ClaimType::Testing,
                _ => {
                    warn!("Unknown claim type '{}', defaulting to Functional", extracted_claim.claim_type);
                    ClaimType::Functional
                }
            };

            claims.push(Claim {
                id: uuid::Uuid::new_v4(),
                artifact_id: artifact.id,
                statement: extracted_claim.statement,
                extraction_confidence: confidence,
                claim_type,
                source_excerpt: extracted_claim.source_excerpt,
                extracted_at: chrono::Utc::now(),
            });
        }

        // Merge extraction metadata
        extraction_metadata.extend(output.extraction_metadata);
        extraction_metadata.insert("final_processing_time_ms".to_string(), processing_time.to_string());

        Ok(ClaimExtractionResult {
            claims,
            processing_time_ms: processing_time,
            model_used: "claude-3-sonnet-20240229".to_string(),
            extraction_metadata,
        })
    }

    fn can_handle(&self, _artifact_type: &ArtifactType) -> bool {
        true // Claude can handle all artifact types
    }

    fn recommended_confidence_threshold(&self, artifact_type: &ArtifactType) -> Confidence {
        let threshold = match artifact_type {
            ArtifactType::Code => 0.7,        // Code claims should be fairly confident
            ArtifactType::Test => 0.8,        // Test claims are usually explicit
            ArtifactType::Documentation => 0.6, // Docs can be ambiguous
            ArtifactType::Ticket => 0.9,      // Requirements should be clear
            ArtifactType::Commit => 0.5,      // Commit messages vary in quality
            _ => 0.6,
        };
        Confidence::new(threshold).unwrap()
    }
}

/// Claude-powered alignment checker
pub struct ClaudeAlignmentChecker {
    query_resolver: QueryResolver<ClaudeClient>,
}

impl ClaudeAlignmentChecker {
    pub fn new(api_key: String) -> Self {
        let client = ClaudeClient::new(api_key);
        let config = RetryConfig::default();
        let query_resolver = QueryResolver::new(client, config);
        
        Self { query_resolver }
    }

    fn build_alignment_prompt(&self, claim: &Claim, evidence_artifact: &Artifact) -> String {
        format!(
            r#"
You are an expert software analyst evaluating how well evidence supports a claim.

TASK: Determine if the evidence artifact provides support for the given claim.

CLAIM TO EVALUATE:
- Statement: "{}"
- Type: {:?}
- Source: {}
- Extracted from: "{}"

EVIDENCE ARTIFACT:
- Type: {:?}  
- Location: {}
- Content:
```
{}
```

EVALUATION CRITERIA:
1. SEMANTIC ALIGNMENT (0.0-1.0): Does the evidence directly address the same concept as the claim?
2. FUNCTIONAL ALIGNMENT (0.0-1.0): Does the evidence implement/test/document the claimed functionality?
3. BEHAVIORAL ALIGNMENT (0.0-1.0): Does the evidence exhibit the claimed behavior?
4. STRUCTURAL ALIGNMENT (0.0-1.0): Does the evidence follow the claimed structure/pattern?
5. TEMPORAL ALIGNMENT (0.0-1.0): Is the evidence current/relevant to the claim timeframe?

SCORING GUIDELINES:
- 0.9-1.0: Direct, complete support for the claim
- 0.7-0.9: Strong support with minor gaps
- 0.5-0.7: Moderate support, some aspects missing
- 0.3-0.5: Weak support, major gaps or contradictions
- 0.1-0.3: Little to no support
- 0.0: Evidence contradicts the claim

Provide detailed analysis in JSON format:

{{
  "alignment_score": 0.0-1.0,
  "explanation": "Detailed reasoning for the score",
  "evidence_points": [
    {{
      "excerpt": "Specific text from evidence",
      "evidence_type": "supporting|contradicting|neutral|outdated",
      "strength": 0.0-1.0,
      "location": "where in the artifact this was found"
    }}
  ],
  "dimensions": {{
    "semantic_alignment": 0.0-1.0,
    "functional_alignment": 0.0-1.0,
    "behavioral_alignment": 0.0-1.0,
    "structural_alignment": 0.0-1.0,
    "temporal_alignment": 0.0-1.0
  }}
}}
"#,
            claim.statement,
            claim.claim_type,
            claim.source_excerpt,
            claim.source_excerpt,
            evidence_artifact.artifact_type,
            evidence_artifact.location.display(),
            evidence_artifact.content
        )
    }
}

#[async_trait]
impl AlignmentChecker for ClaudeAlignmentChecker {
    async fn check_alignment(
        &self,
        claim: &Claim,
        evidence_artifact: &Artifact,
        _config: &AnalysisConfig,
    ) -> Result<Alignment, AnalysisError> {
        info!("Checking alignment between claim '{}' and evidence from {}", 
              claim.statement.chars().take(50).collect::<String>(),
              evidence_artifact.location.display());

        let prompt = self.build_alignment_prompt(claim, evidence_artifact);
        
        let output = match self.query_resolver.query::<AlignmentOutput>(prompt).await {
            Ok(output) => output,
            Err(e) => {
                error!("Alignment query failed: {}, creating fallback alignment with 0 score", e);
                
                // Create fallback alignment with 0 score for failed deserializations
                AlignmentOutput {
                    alignment_score: 0.0,
                    explanation: format!("Failed to deserialize alignment response: {}. Raw response analysis failed.", e),
                    evidence_points: vec![EvidencePointOutput {
                        excerpt: evidence_artifact.content.clone().chars().take(100).collect(),
                        full_text: Some(evidence_artifact.content.clone()),
                        evidence_type: "analysis_failed".to_string(),
                        strength: 0.0,
                        location: Some("deserialization_error".to_string()),
                    }],
                    dimensions: AlignmentDimensionsOutput {
                        semantic_alignment: 0.0,
                        functional_alignment: 0.0,
                        behavioral_alignment: 0.0,
                        structural_alignment: 0.0,
                        temporal_alignment: 0.0,
                    },
                }
            }
        };

        let alignment_score = Confidence::new(output.alignment_score)
            .map_err(|e| AnalysisError::Configuration(format!("Invalid alignment score: {}", e)))?;

        Ok(Alignment {
            id: uuid::Uuid::new_v4(),
            claim_id: claim.id,
            evidence_artifact_id: evidence_artifact.id,
            alignment_score,
            explanation: output.explanation,
            checked_at: chrono::Utc::now(),
        })
    }

    async fn check_batch_alignment(
        &self,
        claims: &[Claim],
        evidence_artifacts: &[Artifact],
        config: &AnalysisConfig,
    ) -> Result<AlignmentResult, AnalysisError> {
        let start_time = std::time::Instant::now();
        let mut alignments = Vec::new();
        
        info!("Checking batch alignment for {} claims against {} evidence artifacts", 
              claims.len(), evidence_artifacts.len());

        for claim in claims {
            for evidence in evidence_artifacts {
                // Don't check alignment between claim and its source artifact
                if claim.artifact_id != evidence.id {
                    let alignment = self.check_alignment(claim, evidence, config).await?;
                    alignments.push(alignment);
                }
            }
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(AlignmentResult {
            alignments,
            processing_time_ms: processing_time,
            evidence_count: evidence_artifacts.len(),
            alignment_metadata: HashMap::from([
                ("batch_size".to_string(), claims.len().to_string()),
                ("evidence_count".to_string(), evidence_artifacts.len().to_string()),
            ]),
        })
    }
}