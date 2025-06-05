use sats_core::*;
use std::collections::HashMap;
use std::env;
use tracing::{info, error, warn, trace};
use dotenv::dotenv;
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::Semaphore;
use serde::Serialize;

#[derive(Serialize)]
struct AnalysisResults {
    artifacts: Vec<Artifact>,
    claims: Vec<Claim>,
    alignments: Vec<Alignment>,
    analysis_summary: AnalysisSummary,
    timestamp: chrono::DateTime<chrono::Utc>,
    total_execution_time_ms: u64,
}

#[derive(Serialize)]
struct AnalysisSummary {
    total_claims: usize,
    total_alignments: usize,
    alignments_checked: usize,
    alignment_limit_reached: bool,
    average_alignment_score: f64,
    strong_alignments_count: usize,
    moderate_alignments_count: usize,
    weak_alignments_count: usize,
}

/// Real-world artifacts from an actual authentication system
/// 
/// # Usage with different log levels:
/// 
/// ## Default (INFO level) - Key progress only:
/// ```bash
/// cargo run --bin real_claude_analysis 2>&1 | tee analysis.log
/// ```
/// 
/// ## TRACE level - Full detailed output (all claims, alignments, reasoning):
/// ```bash
/// RUST_LOG=trace cargo run --bin real_claude_analysis 2>&1 | tee analysis_full.log
/// ```
/// 
/// ## Minimal (WARN level) - Errors and warnings only:
/// ```bash
/// RUST_LOG=warn cargo run --bin real_claude_analysis 2>&1 | tee analysis_minimal.log
/// ```
fn create_real_world_artifacts() -> Vec<Artifact> {
    vec![
        // Real GitHub issue example
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

        // Real implementation code
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

        // Real test code
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
    async fn test_sms_code_flow() {
        let mut mfa = MfaManager::new_for_testing();
        let user_id = "test_user_456";
        let phone = "+1234567890";
        
        // Send SMS code
        mfa.send_sms_code(user_id, phone).await.unwrap();
        
        // Verify the code was stored
        let stored_code = mfa.get_stored_sms_code(user_id).unwrap();
        assert!(stored_code.is_some());
        
        // Test verification
        let code = stored_code.unwrap();
        let result = mfa.verify_sms_code(user_id, &code).unwrap();
        assert!(result, "SMS code verification should succeed");
        
        // Code should be consumed after use
        let second_result = mfa.verify_sms_code(user_id, &code).unwrap();
        assert!(!second_result, "SMS code should only work once");
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

    #[tokio::test]
    async fn test_mfa_audit_logging() {
        let mfa = MfaManager::new_for_testing();
        let user_id = "test_user_audit";
        
        // Generate secret (should log event)
        mfa.generate_totp_secret(user_id).unwrap();
        
        // Failed verification (should log event)
        mfa.verify_totp(user_id, "000000").unwrap();
        
        // Check audit logs
        let logs = mfa.get_audit_logs(user_id);
        assert!(logs.len() >= 2);
        assert!(logs.iter().any(|log| log.event_type == "totp_secret_generated"));
        assert!(logs.iter().any(|log| log.event_type == "totp_failure"));
    }

    // TODO: Test SMS service failover
    // TODO: Test encryption/decryption of secrets
    // TODO: Test backup code verification and consumption
    // TODO: Test concurrent rate limiting
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

        // Real API documentation
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Documentation,
            content: r#"
# Multi-Factor Authentication API

## Overview

Our MFA system provides additional security for user accounts through SMS and TOTP-based verification.

## Setup Endpoints

### POST /api/v1/mfa/totp/setup
Generates a TOTP secret for the authenticated user.

**Response:**
```json
{
  "secret": "JBSWY3DPEHPK3PXP",
  "qr_code": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...",
  "backup_codes": ["12345678", "87654321", ...]
}
```

### POST /api/v1/mfa/sms/setup
Enables SMS-based MFA for a phone number.

**Request:**
```json
{
  "phone_number": "+1234567890"
}
```

## Verification Endpoints

### POST /api/v1/mfa/verify
Verifies an MFA code during login.

**Request:**
```json
{
  "type": "totp",  // or "sms" or "backup"
  "code": "123456",
  "user_id": "user123"
}
```

**Response:**
```json
{
  "valid": true,
  "remaining_attempts": 4
}
```

## Rate Limiting

- **TOTP/SMS verification**: 5 attempts per 5 minutes per user
- **SMS sending**: 3 SMS per hour per phone number
- **Backup codes**: 3 attempts per hour per user

Rate limit exceeded responses return HTTP 429 with:
```json
{
  "error": "rate_limit_exceeded",
  "retry_after": 300
}
```

## Security Features

✅ **Implemented:**
- TOTP secrets encrypted at rest using AES-256
- SMS codes expire after 5 minutes
- TOTP codes accept ±30 seconds for clock skew
- Comprehensive audit logging
- Backup codes for account recovery

⚠️ **Partial Implementation:**
- Rate limiting (basic implementation, needs refinement)
- SMS service fallback (single provider only)

❌ **Not Yet Implemented:**
- Hardware security key support (FIDO2/WebAuthn)
- Admin override capabilities
- Bulk MFA operations for enterprise customers

## Error Codes

| Code | Description |
|------|-------------|
| `invalid_code` | The provided MFA code is incorrect |
| `expired_code` | The code has expired (SMS only) |
| `rate_limit_exceeded` | Too many verification attempts |
| `mfa_not_enabled` | User has not set up MFA |
| `service_unavailable` | MFA service temporarily unavailable |

## Integration Example

```javascript
// Enable TOTP for user
const setupResponse = await fetch('/api/v1/mfa/totp/setup', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${userToken}`,
    'Content-Type': 'application/json'
  }
});

const { secret, qr_code, backup_codes } = await setupResponse.json();

// Show QR code to user for scanning with authenticator app
displayQRCode(qr_code);

// Store backup codes securely
securelyStoreBackupCodes(backup_codes);
```
"#.to_string(),
            location: Location::File { 
                path: "docs/api/mfa.md".to_string(), 
                line_range: None 
            },
            created_at: chrono::Utc::now() - chrono::Duration::days(3),
            author: Some("docs-team@company.com".to_string()),
            metadata: HashMap::new(),
        },

        // Real commit message
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Commit,
            content: r#"
feat(auth): implement TOTP-based multi-factor authentication

- Add MfaManager with TOTP secret generation and verification
- Implement rate limiting for MFA verification attempts (5 per 5 minutes)
- Add encrypted storage for TOTP secrets using AES-256
- Support ±30 second window for TOTP verification to handle clock skew
- Add comprehensive audit logging for all MFA events
- Generate 10 single-use backup codes for account recovery

Security improvements:
- All MFA secrets are encrypted at rest
- Rate limiting prevents brute force attacks
- Audit logs provide security monitoring capabilities

Still TODO:
- SMS-based MFA implementation
- Hardware security key support
- Admin override capabilities

Resolves: #1247
Co-authored-by: Security Team <security@company.com>
"#.to_string(),
            location: Location::Commit { 
                hash: "a7b3c9d2e1f4567890abcdef1234567890abcdef".to_string(), 
                file_path: Some("src/auth/mfa.rs".to_string()) 
            },
            created_at: chrono::Utc::now() - chrono::Duration::days(2),
            author: Some("alice@company.com".to_string()),
            metadata: HashMap::from([
                ("branch".to_string(), "feature/mfa-implementation".to_string()),
                ("lines_added".to_string(), "145".to_string()),
                ("lines_removed".to_string(), "0".to_string()),
            ]),
        },
    ]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start timing the entire analysis
    let analysis_start_time = std::time::Instant::now();
    
    // Load environment variables from .env file
    dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Get Claude API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY environment variable not set. Please create a .env file with ANTHROPIC_API_KEY=your-key or set the environment variable.")?;
    
    info!("🚀 Starting Real Claude-Powered SATS Analysis");
    info!("🔑 Using Claude API for live claim extraction and alignment checking");
    
    // Create real-world artifacts
    let artifacts = create_real_world_artifacts();
    info!("📁 Created {} real-world artifacts from MFA implementation project", artifacts.len());
    
    // Initialize Claude-powered analyzers
    let claim_extractor = ClaudeClaimExtractor::new(api_key.clone());
    let alignment_checker = ClaudeAlignmentChecker::new(api_key);
    let config = AnalysisConfig::default();
    
    info!("🔍 Extracting claims using Claude in parallel...");
    info!("🤖 LIVE CLAUDE ANALYSIS");
    info!("=======================");
    
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
            (i, artifact.location.display(), result)
        })
    }).collect();
    
    let claim_results = join_all(claim_extraction_tasks).await;
    let mut all_claims = Vec::new();
    
    for task_result in claim_results {
        match task_result {
            Ok((i, _location, Ok(result))) => {
                trace!("   ✅ Artifact {}: Extracted {} claims in {}ms using {}", 
                         i + 1,
                         result.claims.len(), 
                         result.processing_time_ms, 
                         result.model_used);
                
                // Show all claims, not just first 3
                for (j, claim) in result.claims.iter().enumerate() {
                    trace!("      {}. [{}] \"{}\" (confidence: {:.2})", 
                             j + 1,
                             format!("{:?}", claim.claim_type),
                             claim.statement,
                             claim.extraction_confidence.value());
                    trace!("         Source: {}", claim.source_excerpt);
                }
                
                all_claims.extend(result.claims);
            }
            Ok((i, _location, Err(e))) => {
                error!("   ❌ Artifact {}: Failed to extract claims: {}", i + 1, e);
            }
            Err(e) => {
                error!("   ❌ Task failed: {}", e);
            }
        }
    }
    
    info!("✅ Total claims extracted: {}", all_claims.len());
    
    // Check alignments using Claude with limit and parallel processing
    info!("⚖️ Checking alignments using Claude in parallel...");
    info!("🔍 ALIGNMENT ANALYSIS");
    info!("=====================");
    
    // Collect all possible alignment pairs (excluding self-alignment)
    let mut alignment_pairs = Vec::new();
    for claim in &all_claims {
        for evidence in &artifacts {
            if claim.artifact_id != evidence.id {
                alignment_pairs.push((claim.clone(), evidence.clone()));
            }
        }
    }
    
    // Apply 1query limit (this is set by a human! please keep to 1)
    const MAX_ALIGNMENTS: usize = 1;
    let total_possible = alignment_pairs.len();
    let alignments_to_check = alignment_pairs.len().min(MAX_ALIGNMENTS);
    alignment_pairs.truncate(alignments_to_check);
    
    let alignment_limit_reached = total_possible > MAX_ALIGNMENTS;
    if alignment_limit_reached {
        warn!("⚠️ Limiting alignment checks to {} out of {} possible pairs", MAX_ALIGNMENTS, total_possible);
    }
    
    info!("🔗 Checking {} alignments in parallel...", alignments_to_check);
    
    // Create semaphore to limit concurrent requests (be respectful to API)
    let semaphore = Arc::new(Semaphore::new(5)); // Max 5 concurrent requests
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
                
                trace!("      💭 Claude's reasoning: {}", alignment.explanation);
                
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
    
    // Calculate analysis summary
    let strong_alignments_count = alignments.iter().filter(|a| a.alignment_score.value() >= 0.7).count();
    let moderate_alignments_count = alignments.iter().filter(|a| a.alignment_score.value() >= 0.4 && a.alignment_score.value() < 0.7).count();
    let weak_alignments_count = alignments.iter().filter(|a| a.alignment_score.value() < 0.4).count();
    
    let avg_alignment = if alignments.is_empty() { 
        0.0 
    } else { 
        alignments.iter().map(|a| a.alignment_score.value()).sum::<f64>() / alignments.len() as f64
    };
    
    let analysis_summary = AnalysisSummary {
        total_claims: all_claims.len(),
        total_alignments: total_possible,
        alignments_checked: alignments.len(),
        alignment_limit_reached,
        average_alignment_score: avg_alignment,
        strong_alignments_count,
        moderate_alignments_count,
        weak_alignments_count,
    };
    
    // Display results
    info!("📊 ANALYSIS RESULTS");
    info!("===================");
    
    info!("🎯 PROJECT HEALTH SUMMARY:");
    info!("   Total Claims Extracted: {}", analysis_summary.total_claims);
    if analysis_summary.alignment_limit_reached {
        info!("   Total Possible Alignments: {} (limited to {})", analysis_summary.total_alignments, analysis_summary.alignments_checked);
    } else {
        info!("   Total Alignments Checked: {}", analysis_summary.alignments_checked);
    }
    info!("   Average Alignment Score: {:.2}", analysis_summary.average_alignment_score);
    
    if !alignments.is_empty() {
        info!("   Strong Alignments (≥0.7): {} ({:.1}%)", 
                 analysis_summary.strong_alignments_count, 
                 analysis_summary.strong_alignments_count as f64 / alignments.len() as f64 * 100.0);
        info!("   Moderate Alignments (0.4-0.7): {} ({:.1}%)", 
                 analysis_summary.moderate_alignments_count,
                 analysis_summary.moderate_alignments_count as f64 / alignments.len() as f64 * 100.0);
        info!("   Weak Alignments (<0.4): {} ({:.1}%)", 
                 analysis_summary.weak_alignments_count,
                 analysis_summary.weak_alignments_count as f64 / alignments.len() as f64 * 100.0);
    }
    
    // Find top alignments
    info!("🏆 STRONGEST ALIGNMENTS:");
    let mut sorted_alignments = alignments.clone();
    sorted_alignments.sort_by(|a, b| b.alignment_score.value().partial_cmp(&a.alignment_score.value()).unwrap());
    
    for (i, alignment) in sorted_alignments.iter().enumerate() {
        let claim = all_claims.iter().find(|c| c.id == alignment.claim_id).unwrap();
        let evidence = artifacts.iter().find(|a| a.id == alignment.evidence_artifact_id).unwrap();
        
        trace!("   {}. Score: {:.2} - \"{}\" ↔ {}", 
                 i + 1,
                 alignment.alignment_score.value(),
                 claim.statement,
                 evidence.location.display());
        trace!("      Reasoning: {}", alignment.explanation);
    }
    
    // Find potential gaps
    info!("⚠️ POTENTIAL GAPS (Claims with weak evidence):");
    let mut claim_max_alignments: HashMap<uuid::Uuid, f64> = HashMap::new();
    
    for alignment in &alignments {
        let current_max = claim_max_alignments.get(&alignment.claim_id).unwrap_or(&0.0);
        if alignment.alignment_score.value() > *current_max {
            claim_max_alignments.insert(alignment.claim_id, alignment.alignment_score.value());
        }
    }
    
    let mut weak_claims: Vec<_> = all_claims.iter()
        .filter(|claim| {
            claim_max_alignments.get(&claim.id).unwrap_or(&0.0) < &0.5
        })
        .collect();
    
    weak_claims.sort_by(|a, b| {
        let a_score = claim_max_alignments.get(&a.id).unwrap_or(&0.0);
        let b_score = claim_max_alignments.get(&b.id).unwrap_or(&0.0);
        a_score.partial_cmp(b_score).unwrap()
    });
    
    for (i, claim) in weak_claims.iter().enumerate() {
        let max_score = claim_max_alignments.get(&claim.id).unwrap_or(&0.0);
        trace!("   {}. [{}] \"{}\" (best alignment: {:.2})", 
                 i + 1,
                 format!("{:?}", claim.claim_type),
                 claim.statement,
                 max_score);
        trace!("      Source: {}", claim.source_excerpt);
    }
    
    // Calculate total execution time
    let total_execution_time = analysis_start_time.elapsed().as_millis() as u64;
    
    // Save results to file
    let results = AnalysisResults {
        artifacts: artifacts.clone(),
        claims: all_claims.clone(),
        alignments: alignments.clone(),
        analysis_summary,
        timestamp: chrono::Utc::now(),
        total_execution_time_ms: total_execution_time,
    };
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("claude_analysis_results_{}.json", timestamp);
    
    info!("💾 Saving results to file: {}", filename);
    match serde_json::to_string_pretty(&results) {
        Ok(json_content) => {
            match std::fs::write(&filename, json_content) {
                Ok(()) => {
                    info!("   ✅ Results saved successfully!");
                }
                Err(e) => {
                    error!("   ❌ Failed to write file: {}", e);
                }
            }
        }
        Err(e) => {
            error!("   ❌ Failed to serialize results: {}", e);
        }
    }
    
    info!("🎉 Claude-powered SATS analysis completed!");
    info!("💡 This analysis used live Claude API calls to extract {} claims and check {} alignments.", 
             all_claims.len(), alignments.len());
    info!("⏱️ Total execution time: {:.2} seconds ({} ms)", 
             total_execution_time as f64 / 1000.0, total_execution_time);
    if alignment_limit_reached {
        warn!("⚠️ Note: Analysis was limited to {} alignment checks due to the {} query limit.", alignments_to_check, MAX_ALIGNMENTS);
    }
    
    Ok(())
}