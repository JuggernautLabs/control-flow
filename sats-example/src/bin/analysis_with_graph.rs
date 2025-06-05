use sats_core::*;
use std::collections::HashMap;
use std::env;
use tracing::{info, error, warn, trace};
use dotenv::dotenv;
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::Semaphore;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct GraphNode {
    id: Option<String>,
    label: String,
    color: Option<String>,
    size: Option<f64>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
struct GraphEdge {
    id: Option<String>,
    source: String,
    target: String,
    label: Option<String>,
    weight: Option<f64>,
    color: Option<String>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

struct GraphClient {
    base_url: String,
    client: reqwest::Client,
}

impl GraphClient {
    fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    async fn clear_graph(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/api/clear", self.base_url);
        let response = self.client.post(&url).send().await?;
        let _: ApiResponse<String> = response.json().await?;
        Ok(())
    }

    async fn add_node(&self, node: GraphNode) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/api/nodes", self.base_url);
        let response = self.client
            .post(&url)
            .json(&node)
            .send()
            .await?;
        
        let api_response: ApiResponse<serde_json::Value> = response.json().await?;
        
        if !api_response.success {
            return Err(format!("Failed to add node: {:?}", api_response.error).into());
        }

        let node_data = api_response.data.unwrap();
        Ok(node_data["id"].as_str().unwrap().to_string())
    }

    async fn add_edge(&self, edge: GraphEdge) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/api/edges", self.base_url);
        let response = self.client
            .post(&url)
            .json(&edge)
            .send()
            .await?;
        
        let api_response: ApiResponse<serde_json::Value> = response.json().await?;
        
        if !api_response.success {
            return Err(format!("Failed to add edge: {:?}", api_response.error).into());
        }

        let edge_data = api_response.data.unwrap();
        Ok(edge_data["id"].as_str().unwrap().to_string())
    }
}

/// Create real-world artifacts (same as the original example)
fn create_real_world_artifacts() -> Vec<Artifact> {
    vec![
        // GitHub issue example
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Ticket,
            content: r#"
# Feature Request: Multi-Factor Authentication (MFA)

## Description
We need to implement multi-factor authentication to enhance security for user accounts. This should support both SMS and authenticator app-based TOTP.

## Acceptance Criteria
- [ ] Users can enable/disable MFA in their account settings
- [ ] Support SMS-based verification codes
- [ ] Support TOTP authenticator apps (Google Authenticator, Authy, etc.)
- [ ] Backup codes generation for account recovery
- [ ] Rate limiting on MFA verification attempts
- [ ] Audit logging for all MFA events
- [ ] Graceful fallback if MFA service is unavailable

## Technical Requirements
- Must integrate with existing JWT authentication
- Should work with current user session management
- Need secure storage for MFA secrets
- Rate limiting: max 5 attempts per 5 minutes
- Backup codes: 10 single-use codes per user

## Security Considerations
- MFA secrets must be encrypted at rest
- SMS codes expire after 5 minutes
- TOTP codes have 30-second windows
- Failed attempts should be logged and monitored
"#.to_string(),
            location: Location::Ticket { 
                system: "GitHub".to_string(), 
                id: "1247".to_string() 
            },
            created_at: chrono::Utc::now() - chrono::Duration::days(14),
            author: Some("security-team@company.com".to_string()),
            metadata: HashMap::from([
                ("priority".to_string(), "high".to_string()),
                ("labels".to_string(), "security,authentication".to_string()),
            ]),
        },

        // Implementation code
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Code,
            content: r#"
use crypto::{AES256, HMAC_SHA256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Multi-Factor Authentication manager
pub struct MfaManager {
    secret_key: Vec<u8>,
    sms_client: SmsClient,
    rate_limiter: RateLimiter,
}

impl MfaManager {
    /// Generate TOTP secret for a user
    pub fn generate_totp_secret(&self, user_id: &str) -> Result<String, MfaError> {
        info!("Generating TOTP secret for user: {}", user_id);
        
        let secret = self.generate_random_secret(32)?;
        let encrypted_secret = self.encrypt_secret(&secret)?;
        
        // Store encrypted secret in database
        self.store_user_secret(user_id, &encrypted_secret)?;
        
        Ok(base32::encode(&secret))
    }

    /// Verify TOTP code from authenticator app  
    pub fn verify_totp(&self, user_id: &str, code: &str) -> Result<bool, MfaError> {
        info!("Verifying TOTP code for user: {}", user_id);
        
        // Check rate limiting first
        if !self.rate_limiter.allow_request(user_id) {
            warn!("Rate limit exceeded for user: {}", user_id);
            return Err(MfaError::RateLimitExceeded);
        }

        let encrypted_secret = self.get_user_secret(user_id)?;
        let secret = self.decrypt_secret(&encrypted_secret)?;
        
        // TOTP verification with 30-second window tolerance
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let time_steps = current_time.as_secs() / 30;
        
        // Check current window and ±1 window for clock skew
        for step in [time_steps.wrapping_sub(1), time_steps, time_steps + 1] {
            if self.generate_totp(&secret, step)? == code {
                self.log_mfa_event(user_id, "totp_success");
                return Ok(true);
            }
        }
        
        self.log_mfa_event(user_id, "totp_failure");
        Ok(false)
    }

    /// Send SMS verification code
    pub async fn send_sms_code(&self, user_id: &str, phone: &str) -> Result<(), MfaError> {
        info!("Sending SMS code to user: {}", user_id);
        
        if !self.rate_limiter.allow_request(user_id) {
            return Err(MfaError::RateLimitExceeded);
        }

        let code = self.generate_sms_code();
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);
        
        // Store code with expiration
        self.store_sms_code(user_id, &code, expires_at)?;
        
        // Send via SMS service
        self.sms_client.send_message(phone, &format!("Your verification code: {}", code)).await?;
        
        self.log_mfa_event(user_id, "sms_sent");
        Ok(())
    }

    /// Generate backup recovery codes
    pub fn generate_backup_codes(&self, user_id: &str) -> Result<Vec<String>, MfaError> {
        info!("Generating backup codes for user: {}", user_id);
        
        let mut codes = Vec::new();
        for _ in 0..10 {
            codes.push(self.generate_backup_code()?);
        }
        
        // Store hashed versions of codes
        let hashed_codes: Vec<_> = codes.iter()
            .map(|code| self.hash_backup_code(code))
            .collect::<Result<Vec<_>, _>>()?;
        
        self.store_backup_codes(user_id, &hashed_codes)?;
        self.log_mfa_event(user_id, "backup_codes_generated");
        
        Ok(codes)
    }
}
"#.to_string(),
            location: Location::File { 
                path: "src/auth/mfa.rs".to_string(), 
                line_range: Some((1, 85)) 
            },
            created_at: chrono::Utc::now() - chrono::Duration::days(7),
            author: Some("alice@company.com".to_string()),
            metadata: HashMap::new(),
        },

        // Test code
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Test,
            content: r#"
#[cfg(test)]
mod mfa_tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_totp_generation_and_verification() {
        let mfa = MfaManager::new_for_testing();
        let user_id = "test_user_123";
        
        // Generate TOTP secret
        let secret = mfa.generate_totp_secret(user_id).unwrap();
        assert_eq!(secret.len(), 52); // Base32 encoded 32-byte secret
        
        // Generate a TOTP code using the secret
        let totp_code = generate_test_totp(&secret);
        
        // Verify the code
        let result = mfa.verify_totp(user_id, &totp_code).unwrap();
        assert!(result, "TOTP verification should succeed");
        
        // Test invalid code
        let invalid_result = mfa.verify_totp(user_id, "000000").unwrap();
        assert!(!invalid_result, "Invalid TOTP should fail");
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mfa = MfaManager::new_for_testing();
        let user_id = "test_user_789";
        
        // Make 5 failed attempts (should succeed)
        for i in 0..5 {
            let result = mfa.verify_totp(user_id, "000000");
            assert!(result.is_ok(), "Attempt {} should be allowed", i + 1);
        }
        
        // 6th attempt should be rate limited
        let result = mfa.verify_totp(user_id, "000000");
        assert!(matches!(result, Err(MfaError::RateLimitExceeded)));
    }

    #[test]
    fn test_backup_codes_generation() {
        let mfa = MfaManager::new_for_testing();
        let user_id = "test_user_backup";
        
        let codes = mfa.generate_backup_codes(user_id).unwrap();
        
        // Should generate exactly 10 codes
        assert_eq!(codes.len(), 10);
        
        // Each code should be unique
        let unique_codes: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique_codes.len(), 10);
        
        // Codes should be 8 characters long
        for code in &codes {
            assert_eq!(code.len(), 8);
        }
    }
}
"#.to_string(),
            location: Location::File { 
                path: "tests/auth/test_mfa.rs".to_string(), 
                line_range: Some((1, 95)) 
            },
            created_at: chrono::Utc::now() - chrono::Duration::days(5),
            author: Some("alice@company.com".to_string()),
            metadata: HashMap::new(),
        },
    ]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Get Claude API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY environment variable not set. Please create a .env file with ANTHROPIC_API_KEY=your-key or set the environment variable.")?;
    
    info!("🚀 Starting SATS Analysis with Live Graph Visualization");
    info!("🔑 Using Claude API for live claim extraction and alignment checking");
    info!("📊 Graph visualization available at http://127.0.0.1:3001");
    
    // Initialize graph client
    let graph_client = GraphClient::new("http://127.0.0.1:3001".to_string());
    
    // Clear the graph first
    info!("🧹 Clearing existing graph...");
    if let Err(e) = graph_client.clear_graph().await {
        warn!("Could not clear graph (server may not be running): {}", e);
        warn!("💡 Start the graph server with: cd graph-server && cargo run");
    }
    
    // Create real-world artifacts
    let artifacts = create_real_world_artifacts();
    info!("📁 Created {} real-world artifacts from MFA implementation project", artifacts.len());
    
    // Add artifacts as nodes to the graph
    info!("📊 Adding artifacts to graph visualization...");
    let mut artifact_node_ids = HashMap::new();
    
    for artifact in &artifacts {
        let color = match artifact.artifact_type {
            ArtifactType::Ticket => "#ff6b6b",      // Red for requirements
            ArtifactType::Code => "#4ecdc4",        // Teal for implementation
            ArtifactType::Test => "#45b7d1",        // Blue for tests
            ArtifactType::Documentation => "#96ceb4", // Green for docs
            _ => "#feca57",                          // Yellow for others
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), format!("{:?}", artifact.artifact_type));
        metadata.insert("location".to_string(), artifact.location.display());
        metadata.insert("created_at".to_string(), artifact.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string());
        
        if let Some(author) = &artifact.author {
            metadata.insert("author".to_string(), author.clone());
        }
        
        // Add artifact-specific metadata
        for (key, value) in &artifact.metadata {
            metadata.insert(format!("artifact_{}", key), value.clone());
        }
        
        // Add content preview
        let content_preview = if artifact.content.len() > 200 {
            format!("{}...", &artifact.content[..200])
        } else {
            artifact.content.clone()
        };
        metadata.insert("content_preview".to_string(), content_preview);

        let node = GraphNode {
            id: Some(artifact.id.to_string()),
            label: format!("{:?}: {}", artifact.artifact_type, artifact.location.display()),
            color: Some(color.to_string()),
            size: Some(25.0),
            metadata: Some(metadata),
        };
        
        if let Ok(node_id) = graph_client.add_node(node).await {
            artifact_node_ids.insert(artifact.id, node_id);
            trace!("Added artifact node: {}", artifact.location.display());
        }
    }
    
    // Initialize Claude-powered analyzers
    let claim_extractor = ClaudeClaimExtractor::new(api_key.clone());
    let alignment_checker = ClaudeAlignmentChecker::new(api_key);
    let config = AnalysisConfig::default();
    
    info!("🔍 Extracting claims using Claude in parallel...");
    
    // Extract claims from all artifacts in parallel
    let claim_extractor = Arc::new(claim_extractor);
    let config = Arc::new(config);
    let total_artifacts = artifacts.len();
    
    let claim_extraction_tasks: Vec<_> = artifacts.iter().enumerate().map(|(i, artifact)| {
        let extractor = Arc::clone(&claim_extractor);
        let cfg = Arc::clone(&config);
        let artifact = artifact.clone();
        
        tokio::spawn(async move {
            trace!("📄 Starting analysis of artifact {}/{}: {}", 
                     i + 1, total_artifacts, artifact.location.display());
            
            let result = extractor.extract_claims(&artifact, &cfg).await;
            (i, artifact.clone(), result)
        })
    }).collect();
    
    let claim_results = join_all(claim_extraction_tasks).await;
    let mut all_claims = Vec::new();
    let mut claim_node_ids = HashMap::new();
    
    for task_result in claim_results {
        match task_result {
            Ok((i, artifact, Ok(result))) => {
                trace!("   ✅ Artifact {}: Extracted {} claims in {}ms using {}", 
                         i + 1,
                         result.claims.len(), 
                         result.processing_time_ms, 
                         result.model_used);
                
                // Add each claim as a node to the graph
                for (j, claim) in result.claims.iter().enumerate() {
                    trace!("      {}. [{}] \"{}\" (confidence: {:.2})", 
                             j + 1,
                             format!("{:?}", claim.claim_type),
                             claim.statement,
                             claim.extraction_confidence.value());
                    
                    let color = match claim.claim_type {
                        ClaimType::Functional => "#ff9ff3",
                        ClaimType::Security => "#ff6b6b",
                        ClaimType::Performance => "#feca57",
                        ClaimType::Behavior => "#48dbfb",
                        ClaimType::Structure => "#ff9ff3",
                        ClaimType::Requirement => "#ff6348",
                        ClaimType::Testing => "#1dd1a1",
                    };
                    
                    let mut claim_metadata = HashMap::new();
                    claim_metadata.insert("type".to_string(), "claim".to_string());
                    claim_metadata.insert("claim_type".to_string(), format!("{:?}", claim.claim_type));
                    claim_metadata.insert("confidence".to_string(), format!("{:.3}", claim.extraction_confidence.value()));
                    claim_metadata.insert("statement".to_string(), claim.statement.clone());
                    claim_metadata.insert("source_excerpt".to_string(), claim.source_excerpt.clone());
                    claim_metadata.insert("extracted_at".to_string(), claim.extracted_at.format("%Y-%m-%d %H:%M:%S UTC").to_string());
                    
                    // Add source artifact info
                    if let Some(source_artifact) = artifacts.iter().find(|a| a.id == claim.artifact_id) {
                        claim_metadata.insert("source_artifact".to_string(), source_artifact.location.display());
                        claim_metadata.insert("source_type".to_string(), format!("{:?}", source_artifact.artifact_type));
                    }

                    let claim_node = GraphNode {
                        id: Some(claim.id.to_string()),
                        label: format!("Claim: {}", &claim.statement[..claim.statement.len().min(50)]),
                        color: Some(color.to_string()),
                        size: Some(15.0 + claim.extraction_confidence.value() * 10.0), // Size based on confidence
                        metadata: Some(claim_metadata),
                    };
                    
                    if let Ok(claim_node_id) = graph_client.add_node(claim_node).await {
                        claim_node_ids.insert(claim.id, claim_node_id);
                        
                        // Add edge from claim to its source artifact
                        if let Some(artifact_node_id) = artifact_node_ids.get(&artifact.id) {
                            let edge = GraphEdge {
                                id: None,
                                source: claim.id.to_string(),
                                target: artifact.id.to_string(),
                                label: Some("extracted_from".to_string()),
                                weight: Some(claim.extraction_confidence.value()),
                                color: Some("#999999".to_string()),
                                metadata: Some(HashMap::new()),
                            };
                            
                            if let Err(e) = graph_client.add_edge(edge).await {
                                trace!("Failed to add extraction edge: {}", e);
                            }
                        }
                    }
                }
                
                all_claims.extend(result.claims);
            }
            Ok((i, _artifact, Err(e))) => {
                error!("   ❌ Artifact {}: Failed to extract claims: {}", i + 1, e);
            }
            Err(e) => {
                error!("   ❌ Task failed: {}", e);
            }
        }
    }
    
    info!("✅ Total claims extracted: {}", all_claims.len());
    
    // Check alignments using Claude with limit and parallel processing
    info!("⚖️ Checking alignments using Claude...");
    
    // Collect all possible alignment pairs (excluding self-alignment)
    let mut alignment_pairs = Vec::new();
    for claim in &all_claims {
        for evidence in &artifacts {
            if claim.artifact_id != evidence.id {
                alignment_pairs.push((claim.clone(), evidence.clone()));
            }
        }
    }
    
    // Apply limit (reduced for demonstration)
    const MAX_ALIGNMENTS: usize = 3;
    let total_possible = alignment_pairs.len();
    let alignments_to_check = alignment_pairs.len().min(MAX_ALIGNMENTS);
    alignment_pairs.truncate(alignments_to_check);
    
    if total_possible > MAX_ALIGNMENTS {
        warn!("⚠️ Limiting alignment checks to {} out of {} possible pairs", MAX_ALIGNMENTS, total_possible);
    }
    
    info!("🔗 Checking {} alignments in parallel...", alignments_to_check);
    
    // Create semaphore to limit concurrent requests
    let semaphore = Arc::new(Semaphore::new(3)); // Max 3 concurrent requests
    let alignment_checker = Arc::new(alignment_checker);
    
    let alignment_tasks: Vec<_> = alignment_pairs.into_iter().enumerate().map(|(i, (claim, evidence))| {
        let checker = Arc::clone(&alignment_checker);
        let config_ref = Arc::clone(&config);
        let sem = Arc::clone(&semaphore);
        
        tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            
            trace!("🔗 Processing alignment {}/{} - \"{}\" ↔ {}", 
                         i + 1, alignments_to_check,
                         claim.statement,
                         evidence.location.display());
            
            let result = checker.check_alignment(&claim, &evidence, &config_ref).await;
            
            // Small delay between requests
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            
            (i, claim, evidence, result)
        })
    }).collect();
    
    let alignment_results = join_all(alignment_tasks).await;
    let mut alignments = Vec::new();
    
    for task_result in alignment_results {
        match task_result {
            Ok((i, claim, evidence, Ok(alignment))) => {
                let score = alignment.alignment_score.value();
                let strength = if score >= 0.7 { "Strong" } 
                              else if score >= 0.4 { "Moderate" } 
                              else { "Weak" };
                
                trace!("   ✅ Alignment {}: Score {:.2} ({}) - \"{}\" ↔ {}", 
                             i + 1, score, strength,
                             claim.statement,
                             evidence.location.display());
                
                // Add alignment edge to graph
                if let (Some(claim_node_id), Some(evidence_node_id)) = 
                    (claim_node_ids.get(&claim.id), artifact_node_ids.get(&evidence.id)) {
                    
                    let edge_color = if score >= 0.7 { "#27ae60" }      // Green for strong
                                    else if score >= 0.4 { "#f39c12" }  // Orange for moderate
                                    else { "#e74c3c" };                 // Red for weak
                    
                    let edge = GraphEdge {
                        id: None,
                        source: claim.id.to_string(),
                        target: evidence.id.to_string(),
                        label: Some(format!("aligned ({:.2})", score)),
                        weight: Some(score),
                        color: Some(edge_color.to_string()),
                        metadata: Some(HashMap::from([
                            ("type".to_string(), "alignment".to_string()),
                            ("score".to_string(), score.to_string()),
                            ("strength".to_string(), strength.to_string()),
                        ])),
                    };
                    
                    if let Err(e) = graph_client.add_edge(edge).await {
                        trace!("Failed to add alignment edge: {}", e);
                    }
                }
                
                alignments.push(alignment);
            }
            Ok((i, _claim, _evidence, Err(e))) => {
                error!("   ❌ Alignment {} failed: {}", i + 1, e);
            }
            Err(e) => {
                error!("   ❌ Task failed: {}", e);
            }
        }
    }
    
    info!("✅ Completed {} alignment checks", alignments.len());
    
    // Calculate and display results
    let strong_alignments = alignments.iter().filter(|a| a.alignment_score.value() >= 0.7).count();
    let moderate_alignments = alignments.iter().filter(|a| a.alignment_score.value() >= 0.4 && a.alignment_score.value() < 0.7).count();
    let weak_alignments = alignments.iter().filter(|a| a.alignment_score.value() < 0.4).count();
    
    let avg_alignment = if alignments.is_empty() { 
        0.0 
    } else { 
        alignments.iter().map(|a| a.alignment_score.value()).sum::<f64>() / alignments.len() as f64
    };
    
    info!("📊 ANALYSIS RESULTS");
    info!("==================");
    info!("🎯 PROJECT HEALTH SUMMARY:");
    info!("   Total Claims Extracted: {}", all_claims.len());
    info!("   Total Alignments Checked: {}", alignments.len());
    info!("   Average Alignment Score: {:.2}", avg_alignment);
    
    if !alignments.is_empty() {
        info!("   Strong Alignments (≥0.7): {} ({:.1}%)", 
                 strong_alignments, 
                 strong_alignments as f64 / alignments.len() as f64 * 100.0);
        info!("   Moderate Alignments (0.4-0.7): {} ({:.1}%)", 
                 moderate_alignments,
                 moderate_alignments as f64 / alignments.len() as f64 * 100.0);
        info!("   Weak Alignments (<0.4): {} ({:.1}%)", 
                 weak_alignments,
                 weak_alignments as f64 / alignments.len() as f64 * 100.0);
    }
    
    info!("🎉 SATS analysis with graph visualization completed!");
    info!("📊 View the interactive graph at: http://127.0.0.1:3001");
    info!("💡 The graph shows:");
    info!("   - Red nodes: Requirements/Tickets");
    info!("   - Teal nodes: Code Implementation");
    info!("   - Blue nodes: Tests");
    info!("   - Colored smaller nodes: Extracted Claims");
    info!("   - Green edges: Strong alignments (≥0.7)");
    info!("   - Orange edges: Moderate alignments (0.4-0.7)");
    info!("   - Red edges: Weak alignments (<0.4)");
    
    Ok(())
}