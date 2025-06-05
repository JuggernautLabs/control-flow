use sats_core::*;
use std::collections::HashMap;
use tracing::info;
use async_trait::async_trait;

/// Mock LLM-based claim extractor for demonstration
struct MockClaimExtractor;

#[async_trait::async_trait]
impl ClaimExtractor for MockClaimExtractor {
    async fn extract_claims(
        &self,
        artifact: &Artifact,
        _config: &AnalysisConfig,
    ) -> Result<ClaimExtractionResult, AnalysisError> {
        let claims = match artifact.artifact_type {
            ArtifactType::Code => extract_code_claims(artifact),
            ArtifactType::Test => extract_test_claims(artifact),
            ArtifactType::Documentation => extract_doc_claims(artifact),
            ArtifactType::Ticket => extract_ticket_claims(artifact),
            _ => Vec::new(),
        };

        Ok(ClaimExtractionResult {
            claims,
            processing_time_ms: 150,
            model_used: "mock-llm".to_string(),
            extraction_metadata: HashMap::new(),
        })
    }

    fn can_handle(&self, _artifact_type: &ArtifactType) -> bool {
        true
    }

    fn recommended_confidence_threshold(&self, _artifact_type: &ArtifactType) -> Confidence {
        Confidence::new(0.6).unwrap()
    }
}

/// Mock alignment checker for demonstration
struct MockAlignmentChecker;

#[async_trait::async_trait]
impl AlignmentChecker for MockAlignmentChecker {
    async fn check_alignment(
        &self,
        claim: &Claim,
        evidence_artifact: &Artifact,
        _config: &AnalysisConfig,
    ) -> Result<Alignment, AnalysisError> {
        let alignment_score = calculate_mock_alignment(claim, evidence_artifact);
        
        Ok(Alignment {
            id: uuid::Uuid::new_v4(),
            claim_id: claim.id,
            evidence_artifact_id: evidence_artifact.id,
            alignment_score,
            explanation: format!(
                "Mock analysis: Claim '{}' has {:.2} alignment with {} evidence",
                claim.statement.chars().take(50).collect::<String>(),
                alignment_score.value(),
                format!("{:?}", evidence_artifact.artifact_type)
            ),
            checked_at: chrono::Utc::now(),
        })
    }

    async fn check_batch_alignment(
        &self,
        claims: &[Claim],
        evidence_artifacts: &[Artifact],
        config: &AnalysisConfig,
    ) -> Result<AlignmentResult, AnalysisError> {
        let mut alignments = Vec::new();
        
        for claim in claims {
            for evidence in evidence_artifacts {
                if claim.artifact_id != evidence.id {
                    let alignment = self.check_alignment(claim, evidence, config).await?;
                    alignments.push(alignment);
                }
            }
        }

        Ok(AlignmentResult {
            alignments,
            processing_time_ms: 500,
            evidence_count: evidence_artifacts.len(),
            alignment_metadata: HashMap::new(),
        })
    }
}

/// Create sample artifacts representing a simple authentication system
fn create_sample_artifacts() -> Vec<Artifact> {
    vec![
        // Requirement ticket
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Ticket,
            content: r#"
# User Story: OAuth2 Authentication
**JIRA-123**: As a user, I want to authenticate using OAuth2 so that I can securely access the application.

## Acceptance Criteria:
- Support OAuth2 authorization code flow
- Handle token refresh automatically  
- Rate limit authentication attempts
- Log all authentication events
- Session timeout after 1 hour of inactivity
"#.to_string(),
            location: Location::Ticket { system: "JIRA".to_string(), id: "123".to_string() },
            created_at: chrono::Utc::now() - chrono::Duration::days(7),
            author: Some("product@company.com".to_string()),
            metadata: HashMap::from([("priority".to_string(), "high".to_string())]),
        },

        // Implementation code
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Code,
            content: r#"
/// OAuth2 authentication handler
pub struct OAuth2Handler {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl OAuth2Handler {
    /// Initiate OAuth2 authorization code flow
    pub async fn start_auth_flow(&self, user_id: &str) -> Result<String, AuthError> {
        info!("Starting OAuth2 flow for user: {}", user_id);
        // Generate authorization URL
        let auth_url = format!(
            "https://oauth.provider.com/auth?client_id={}&redirect_uri={}&response_type=code",
            self.client_id, self.redirect_uri
        );
        Ok(auth_url)
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse, AuthError> {
        info!("Exchanging authorization code for token");
        // Mock token exchange
        Ok(TokenResponse {
            access_token: "mock_token".to_string(),
            refresh_token: Some("mock_refresh".to_string()),
            expires_in: 3600, // 1 hour
        })
    }
}
"#.to_string(),
            location: Location::File { 
                path: "src/auth/oauth2.rs".to_string(), 
                line_range: Some((1, 30)) 
            },
            created_at: chrono::Utc::now() - chrono::Duration::days(3),
            author: Some("dev@company.com".to_string()),
            metadata: HashMap::new(),
        },

        // Test file
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Test,
            content: r#"
#[cfg(test)]
mod oauth2_tests {
    use super::*;

    #[tokio::test]
    async fn test_start_auth_flow() {
        let handler = OAuth2Handler::new("client123", "secret", "http://localhost/callback");
        let auth_url = handler.start_auth_flow("user123").await.unwrap();
        assert!(auth_url.contains("client_id=client123"));
        assert!(auth_url.contains("response_type=code"));
    }

    #[tokio::test] 
    async fn test_token_exchange() {
        let handler = OAuth2Handler::new("client123", "secret", "http://localhost/callback");
        let response = handler.exchange_code("auth_code_123").await.unwrap();
        assert!(!response.access_token.is_empty());
        assert_eq!(response.expires_in, 3600);
    }

    // TODO: Add test for token refresh
    // TODO: Add test for rate limiting
}
"#.to_string(),
            location: Location::File { 
                path: "tests/oauth2_test.rs".to_string(), 
                line_range: Some((1, 25)) 
            },
            created_at: chrono::Utc::now() - chrono::Duration::days(2),
            author: Some("dev@company.com".to_string()),
            metadata: HashMap::new(),
        },

        // Documentation
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Documentation,
            content: r#"
# Authentication API

## OAuth2 Support

Our application supports OAuth2 authentication using the authorization code flow.

### Endpoints

- `GET /auth/oauth2/start` - Initiates OAuth2 flow
- `POST /auth/oauth2/callback` - Handles OAuth2 callback

### Features

- Automatic token refresh ✅
- Rate limiting ❌ (not implemented yet)
- Comprehensive logging ✅
- Configurable session timeout ❌ (hardcoded to 1 hour)

### Security Considerations

All OAuth2 tokens are stored securely and encrypted at rest.
"#.to_string(),
            location: Location::File { 
                path: "docs/authentication.md".to_string(), 
                line_range: None 
            },
            created_at: chrono::Utc::now() - chrono::Duration::days(1),
            author: Some("tech-writer@company.com".to_string()),
            metadata: HashMap::new(),
        },
    ]
}

/// Extract claims from code artifacts using simple pattern matching
fn extract_code_claims(artifact: &Artifact) -> Vec<Claim> {
    let mut claims = Vec::new();
    
    if artifact.content.contains("OAuth2Handler") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "System implements OAuth2 authentication handler".to_string(),
            extraction_confidence: Confidence::new(0.9).unwrap(),
            claim_type: ClaimType::Functional,
            source_excerpt: "pub struct OAuth2Handler".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    if artifact.content.contains("start_auth_flow") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "System can initiate OAuth2 authorization flow".to_string(),
            extraction_confidence: Confidence::new(0.85).unwrap(),
            claim_type: ClaimType::Functional,
            source_excerpt: "pub async fn start_auth_flow".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    if artifact.content.contains("exchange_code") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "System can exchange authorization codes for tokens".to_string(),
            extraction_confidence: Confidence::new(0.85).unwrap(),
            claim_type: ClaimType::Functional,
            source_excerpt: "pub async fn exchange_code".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    if artifact.content.contains("expires_in: 3600") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "Tokens expire after 1 hour".to_string(),
            extraction_confidence: Confidence::new(0.8).unwrap(),
            claim_type: ClaimType::Behavior,
            source_excerpt: "expires_in: 3600".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    claims
}

/// Extract claims from test artifacts
fn extract_test_claims(artifact: &Artifact) -> Vec<Claim> {
    let mut claims = Vec::new();

    if artifact.content.contains("test_start_auth_flow") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "OAuth2 authorization flow is tested".to_string(),
            extraction_confidence: Confidence::new(0.9).unwrap(),
            claim_type: ClaimType::Testing,
            source_excerpt: "async fn test_start_auth_flow".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    if artifact.content.contains("test_token_exchange") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "Token exchange functionality is tested".to_string(),
            extraction_confidence: Confidence::new(0.9).unwrap(),
            claim_type: ClaimType::Testing,
            source_excerpt: "async fn test_token_exchange".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    if artifact.content.contains("TODO: Add test for token refresh") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "Token refresh testing is planned but not implemented".to_string(),
            extraction_confidence: Confidence::new(0.7).unwrap(),
            claim_type: ClaimType::Testing,
            source_excerpt: "TODO: Add test for token refresh".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    if artifact.content.contains("TODO: Add test for rate limiting") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "Rate limiting testing is planned but not implemented".to_string(),
            extraction_confidence: Confidence::new(0.7).unwrap(),
            claim_type: ClaimType::Testing,
            source_excerpt: "TODO: Add test for rate limiting".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    claims
}

/// Extract claims from documentation
fn extract_doc_claims(artifact: &Artifact) -> Vec<Claim> {
    let mut claims = Vec::new();

    if artifact.content.contains("supports OAuth2 authentication") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "Application supports OAuth2 authentication".to_string(),
            extraction_confidence: Confidence::new(0.8).unwrap(),
            claim_type: ClaimType::Functional,
            source_excerpt: "supports OAuth2 authentication using the authorization code flow".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    if artifact.content.contains("Automatic token refresh ✅") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "System implements automatic token refresh".to_string(),
            extraction_confidence: Confidence::new(0.9).unwrap(),
            claim_type: ClaimType::Functional,
            source_excerpt: "Automatic token refresh ✅".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    if artifact.content.contains("Rate limiting ❌") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "Rate limiting is not implemented".to_string(),
            extraction_confidence: Confidence::new(0.9).unwrap(),
            claim_type: ClaimType::Functional,
            source_excerpt: "Rate limiting ❌ (not implemented yet)".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    claims
}

/// Extract claims from ticket/requirement artifacts
fn extract_ticket_claims(artifact: &Artifact) -> Vec<Claim> {
    let mut claims = Vec::new();

    if artifact.content.contains("OAuth2 authorization code flow") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "System must support OAuth2 authorization code flow".to_string(),
            extraction_confidence: Confidence::new(0.95).unwrap(),
            claim_type: ClaimType::Requirement,
            source_excerpt: "Support OAuth2 authorization code flow".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    if artifact.content.contains("Handle token refresh automatically") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "System must handle token refresh automatically".to_string(),
            extraction_confidence: Confidence::new(0.9).unwrap(),
            claim_type: ClaimType::Requirement,
            source_excerpt: "Handle token refresh automatically".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    if artifact.content.contains("Rate limit authentication attempts") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "System must rate limit authentication attempts".to_string(),
            extraction_confidence: Confidence::new(0.9).unwrap(),
            claim_type: ClaimType::Requirement,
            source_excerpt: "Rate limit authentication attempts".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    if artifact.content.contains("Session timeout after 1 hour") {
        claims.push(Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement: "Sessions must timeout after 1 hour of inactivity".to_string(),
            extraction_confidence: Confidence::new(0.85).unwrap(),
            claim_type: ClaimType::Requirement,
            source_excerpt: "Session timeout after 1 hour of inactivity".to_string(),
            extracted_at: chrono::Utc::now(),
        });
    }

    claims
}

/// Calculate mock alignment between claim and evidence
fn calculate_mock_alignment(claim: &Claim, evidence: &Artifact) -> Confidence {
    let score = match (&claim.claim_type, &evidence.artifact_type) {
        // Requirements vs Implementation
        (ClaimType::Requirement, ArtifactType::Code) => {
            if claim.statement.contains("OAuth2") && evidence.content.contains("OAuth2Handler") {
                0.85
            } else if claim.statement.contains("token refresh") && evidence.content.contains("refresh_token") {
                0.7  // Mentioned but not fully implemented
            } else if claim.statement.contains("rate limit") {
                0.1  // Not implemented
            } else {
                0.3
            }
        },
        
        // Requirements vs Tests
        (ClaimType::Requirement, ArtifactType::Test) => {
            if claim.statement.contains("OAuth2") && evidence.content.contains("oauth2_tests") {
                0.6  // Partial coverage
            } else if claim.statement.contains("rate limit") && evidence.content.contains("TODO") {
                0.2  // Planned but not done
            } else {
                0.2
            }
        },

        // Implementation vs Tests
        (ClaimType::Functional, ArtifactType::Test) => {
            if claim.statement.contains("authorization flow") && evidence.content.contains("test_start_auth_flow") {
                0.9
            } else if claim.statement.contains("exchange") && evidence.content.contains("test_token_exchange") {
                0.9
            } else {
                0.3
            }
        },

        // Cross-check claims in documentation
        (ClaimType::Functional, ArtifactType::Documentation) => {
            if claim.statement.contains("OAuth2") && evidence.content.contains("OAuth2") {
                0.8
            } else if claim.statement.contains("automatic token refresh") && evidence.content.contains("Automatic token refresh ✅") {
                0.9
            } else {
                0.4
            }
        },

        _ => 0.3,
    };

    Confidence::new(score).unwrap()
}

/// Simple gap analyzer that identifies common issues
struct MockGapAnalyzer;

#[async_trait::async_trait]
impl GapAnalyzer for MockGapAnalyzer {
    async fn analyze_gaps(
        &self,
        claims: &[Claim],
        alignments: &[Alignment],
        _relationships: &[Relationship],
        config: &AnalysisConfig,
    ) -> Result<Vec<Gap>, AnalysisError> {
        let mut gaps = Vec::new();
        
        // Find claims with no or weak evidence
        for claim in claims {
            let claim_alignments: Vec<_> = alignments.iter()
                .filter(|a| a.claim_id == claim.id)
                .collect();
            
            if claim_alignments.is_empty() {
                gaps.push(Gap {
                    id: uuid::Uuid::new_v4(),
                    gap_type: GapType::NoEvidence,
                    severity: Severity::High,
                    primary_claim_id: claim.id,
                    related_artifact_ids: vec![],
                    description: format!("No evidence found for claim: '{}'", claim.statement),
                    detected_at: chrono::Utc::now(),
                });
            } else {
                let max_alignment = claim_alignments.iter()
                    .map(|a| a.alignment_score.value())
                    .fold(0.0, f64::max);
                
                if max_alignment < config.min_alignment_threshold.value() {
                    let severity = if max_alignment < 0.3 {
                        Severity::High
                    } else {
                        Severity::Medium
                    };
                    
                    gaps.push(Gap {
                        id: uuid::Uuid::new_v4(),
                        gap_type: GapType::WeakEvidence,
                        severity,
                        primary_claim_id: claim.id,
                        related_artifact_ids: claim_alignments.iter()
                            .map(|a| a.evidence_artifact_id)
                            .collect(),
                        description: format!(
                            "Weak evidence for claim: '{}' (best score: {:.2})",
                            claim.statement, max_alignment
                        ),
                        detected_at: chrono::Utc::now(),
                    });
                }
            }
        }
        
        Ok(gaps)
    }

    async fn calculate_project_health(
        &self,
        claims: &[Claim],
        alignments: &[Alignment],
        gaps: &[Gap],
    ) -> Result<ProjectHealth, AnalysisError> {
        let total_claims = claims.len();
        let avg_alignment = if alignments.is_empty() {
            0.0
        } else {
            alignments.iter().map(|a| a.alignment_score.value()).sum::<f64>() / alignments.len() as f64
        };

        let mut gaps_by_severity = HashMap::new();
        gaps_by_severity.insert(Severity::Low, 0);
        gaps_by_severity.insert(Severity::Medium, 0);
        gaps_by_severity.insert(Severity::High, 0);
        gaps_by_severity.insert(Severity::Critical, 0);
        
        for gap in gaps {
            *gaps_by_severity.get_mut(&gap.severity).unwrap() += 1;
        }

        // Simple heuristic for support classification
        let fully_supported = alignments.iter().filter(|a| a.alignment_score.value() >= 0.8).count();
        let partially_supported = alignments.iter().filter(|a| a.alignment_score.value() >= 0.5 && a.alignment_score.value() < 0.8).count();
        let unsupported = total_claims - fully_supported - partially_supported;

        Ok(ProjectHealth {
            total_claims,
            fully_supported_claims: fully_supported,
            partially_supported_claims: partially_supported,
            unsupported_claims: unsupported,
            average_alignment_score: avg_alignment,
            gaps_by_severity,
            coverage_metrics: CoverageMetrics {
                code_files_with_tests: 0.8,
                tests_with_documentation: 0.6,
                tickets_with_implementation: 0.7,
                commits_matching_changes: 0.9,
            },
            analyzed_at: chrono::Utc::now(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting SATS Example Analysis");
    
    // Create sample artifacts
    let artifacts = create_sample_artifacts();
    info!("📁 Created {} sample artifacts", artifacts.len());

    // Initialize analyzers
    let claim_extractor = MockClaimExtractor;
    let alignment_checker = MockAlignmentChecker;
    let gap_analyzer = MockGapAnalyzer;
    let config = AnalysisConfig::default();
    
    // Extract claims from all artifacts
    info!("🔍 Extracting claims from artifacts...");
    let mut all_claims = Vec::new();
    
    for artifact in &artifacts {
        let result = claim_extractor.extract_claims(artifact, &config).await?;
        info!("  📝 {} claims extracted from {} ({})", 
              result.claims.len(), 
              artifact.location.display(),
              format!("{:?}", artifact.artifact_type).to_lowercase());
        all_claims.extend(result.claims);
    }
    
    info!("✅ Total claims extracted: {}", all_claims.len());
    
    // Check alignments between claims and evidence
    info!("⚖️  Checking claim alignments...");
    let alignment_result = alignment_checker
        .check_batch_alignment(&all_claims, &artifacts, &config)
        .await?;
    
    info!("✅ {} alignments checked", alignment_result.alignments.len());
    
    // Analyze gaps
    info!("🔍 Analyzing gaps and inconsistencies...");
    let gaps = gap_analyzer
        .analyze_gaps(&all_claims, &alignment_result.alignments, &[], &config)
        .await?;
    
    info!("⚠️  {} gaps detected", gaps.len());
    
    // Calculate project health
    info!("📊 Calculating project health metrics...");
    let health = gap_analyzer
        .calculate_project_health(&all_claims, &alignment_result.alignments, &gaps)
        .await?;
    
    // Print detailed results
    println!("\n🎯 SATS ANALYSIS RESULTS");
    println!("========================");
    
    println!("\n📈 PROJECT HEALTH OVERVIEW:");
    println!("  Total Claims: {}", health.total_claims);
    println!("  Fully Supported: {} ({:.1}%)", 
             health.fully_supported_claims,
             health.fully_supported_claims as f64 / health.total_claims as f64 * 100.0);
    println!("  Partially Supported: {} ({:.1}%)", 
             health.partially_supported_claims,
             health.partially_supported_claims as f64 / health.total_claims as f64 * 100.0);
    println!("  Unsupported: {} ({:.1}%)", 
             health.unsupported_claims,
             health.unsupported_claims as f64 / health.total_claims as f64 * 100.0);
    println!("  Average Alignment Score: {:.2}", health.average_alignment_score);
    
    println!("\n🚨 GAPS BY SEVERITY:");
    for (severity, count) in &health.gaps_by_severity {
        if *count > 0 {
            println!("  {:?}: {}", severity, count);
        }
    }
    
    println!("\n📋 DETAILED CLAIM ANALYSIS:");
    for claim in &all_claims {
        let claim_alignments: Vec<_> = alignment_result.alignments.iter()
            .filter(|a| a.claim_id == claim.id)
            .collect();
        
        let max_alignment = claim_alignments.iter()
            .map(|a| a.alignment_score.value())
            .fold(0.0, f64::max);
        
        let status = if claim_alignments.is_empty() {
            "❌ NO EVIDENCE"
        } else if max_alignment >= 0.8 {
            "✅ STRONG"
        } else if max_alignment >= 0.5 {
            "⚠️  WEAK"
        } else {
            "❌ VERY WEAK"
        };
        
        println!("  {} [{}] \"{}\"", 
                status, 
                format!("{:?}", claim.claim_type),
                claim.statement.chars().take(60).collect::<String>());
        
        if !claim_alignments.is_empty() {
            println!("    Best alignment: {:.2}", max_alignment);
        }
    }
    
    println!("\n🔍 DETECTED GAPS:");
    for gap in &gaps {
        let severity_icon = match gap.severity {
            Severity::Critical => "🚨",
            Severity::High => "🔴",
            Severity::Medium => "🟡",
            Severity::Low => "🟢",
        };
        
        println!("  {} [{}] {}", 
                severity_icon,
                format!("{:?}", gap.severity),
                gap.description);
    }
    
    println!("\n💡 KEY INSIGHTS:");
    
    // Specific insights based on our analysis
    let rate_limiting_gaps: Vec<_> = gaps.iter()
        .filter(|g| g.description.contains("rate limit"))
        .collect();
    
    if !rate_limiting_gaps.is_empty() {
        println!("  • Rate limiting is required but not implemented or tested");
    }
    
    let todo_gaps: Vec<_> = all_claims.iter()
        .filter(|c| c.statement.contains("planned but not implemented"))
        .collect();
    
    if !todo_gaps.is_empty() {
        println!("  • {} features are planned but have incomplete test coverage", todo_gaps.len());
    }
    
    let token_refresh_implemented = all_claims.iter()
        .any(|c| c.statement.contains("automatic token refresh") && c.claim_type == ClaimType::Functional);
    
    let token_refresh_documented = all_claims.iter()
        .any(|c| c.statement.contains("implements automatic token refresh"));
        
    if token_refresh_implemented && token_refresh_documented {
        println!("  • Token refresh appears to be implemented and documented but lacks test coverage");
    }
    
    println!("\n🎯 RECOMMENDATIONS:");
    println!("  1. Implement rate limiting for authentication attempts");
    println!("  2. Add comprehensive tests for token refresh functionality"); 
    println!("  3. Add tests for rate limiting once implemented");
    println!("  4. Update documentation to reflect current implementation status");
    
    info!("🏁 SATS analysis completed successfully!");
    
    Ok(())
}