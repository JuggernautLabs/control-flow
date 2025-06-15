//! Verification Claim Extraction Workflow Example
//! 
//! This example demonstrates SATS v2's verification-focused claim extraction
//! using actual LLM-based extraction from project artifacts instead of 
//! manually providing claims.

use sats_v2::*;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 SATS v2 Verification Claim Extraction Workflow");
    println!("==================================================");
    println!();

    // Check for API key
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set (check .env file)");

    // Step 1: Create project artifacts representing a real development scenario
    let artifacts = create_authentication_project_artifacts();
    println!("📁 Created {} project artifacts", artifacts.len());
    for artifact in &artifacts {
        println!("  • {} ({:?})", artifact.location.display(), artifact.artifact_type);
    }
    println!();

    // Step 2: Set up the verification claim extractor
    let claim_extractor = ClaudeVerificationExtractor::new(api_key);
    println!("🤖 Initialized Claude-based verification claim extractor");
    println!();

    // Step 3: Extract verification-focused claims from each artifact
    let mut all_claims = Vec::new();
    for artifact in &artifacts {
        println!("🔍 Extracting claims from {}", artifact.location.display());
        
        let result = claim_extractor.extract_verification_claims(artifact).await?;
        
        println!("  ✅ Extracted {} claims from {:?}:{} in {}ms", 
                 result.claims.len(), 
                 artifact.artifact_type, artifact.id,
                 result.processing_time_ms);
        
        for (i, claim) in result.claims.iter().enumerate() {
            println!("    {}. {}", i + 1, claim.statement);
            if let Some(chain) = &claim.verification_chain {
                println!("       Status: {:?}", chain.status);
                println!("       Missing: {:?}", chain.missing_links);
            }
        }
        
        all_claims.extend(result.claims);
        println!();
    }

    // Step 4: Process claims through code reference workflow
    println!("🔄 Processing {} extracted claims through verification workflow", all_claims.len());
    
    let mut agent_orchestrator = setup_ai_agents();
    
    for (i, claim) in all_claims.iter().enumerate() {
        println!("🎯 Processing claim {}: {}", i + 1, claim.statement);
        
        let result = process_claim_with_verification_chain(claim, &mut agent_orchestrator).await?;
        display_verification_result(&result);
        println!();
    }

    println!("✅ Verification extraction workflow completed!");
    Ok(())
}

/// Create realistic artifacts from an authentication project
fn create_authentication_project_artifacts() -> Vec<Artifact> {
    vec![
        // Product requirements ticket
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Ticket,
            content: r#"
# Epic: Multi-Factor Authentication Implementation

**TICKET-456**: As a security-conscious user, I want multi-factor authentication (MFA) so that my account is protected against unauthorized access.

## Acceptance Criteria:
- Users can enable/disable MFA through account settings
- Support for TOTP (Time-based One-Time Password) via authenticator apps
- SMS fallback option for users without authenticator apps  
- Recovery codes generation and validation
- MFA requirement bypass for trusted devices (optional)
- Audit logging for all MFA events

## Technical Requirements:
- TOTP implementation using RFC 6238 standard
- Secure storage of MFA secrets
- Rate limiting for MFA attempts
- Integration with existing session management
- API endpoints for MFA setup and verification

## Security Considerations:
- Secret keys must be stored encrypted
- Recovery codes must be hashed
- Implement timing attack protection
- Proper error handling without information leakage
"#.to_string(),
            location: Location::Ticket { 
                system: "JIRA".to_string(), 
                id: "TICKET-456".to_string() 
            },
            created_at: chrono::Utc::now() - chrono::Duration::days(5),
            author: Some("product@company.com".to_string()),
            metadata: HashMap::from([
                ("priority".to_string(), "high".to_string()),
                ("epic".to_string(), "security-enhancement".to_string()),
            ]),
        },

        // Implementation code 
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Code,
            content: r#"
use chrono::{Duration, Utc};
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};
use totp_rs::{Algorithm, TOTP};

/// Multi-Factor Authentication service
pub struct MfaService {
    secret_store: Box<dyn SecretStore>,
    rate_limiter: RateLimiter,
}

impl MfaService {
    /// Enable MFA for a user by generating a new secret
    pub async fn enable_mfa(&self, user_id: &str) -> Result<MfaSetupInfo, MfaError> {
        // Generate cryptographically secure secret
        let secret = self.generate_secret();
        
        // Create TOTP instance
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,  // 6 digit codes
            1,  // 30 second step
            30,
            secret.clone(),
            Some("MyApp".to_string()),
            user_id.to_string(),
        )?;
        
        // Store encrypted secret
        self.secret_store.store_secret(user_id, &secret).await?;
        
        // Generate recovery codes
        let recovery_codes = self.generate_recovery_codes();
        self.secret_store.store_recovery_codes(user_id, &recovery_codes).await?;
        
        Ok(MfaSetupInfo {
            qr_code_url: totp.get_qr_base64()?,
            recovery_codes,
            secret: secret, // For manual entry
        })
    }
    
    /// Verify TOTP code from user
    pub async fn verify_totp(&self, user_id: &str, code: &str) -> Result<bool, MfaError> {
        // Check rate limiting
        if !self.rate_limiter.allow_attempt(user_id) {
            return Err(MfaError::RateLimited);
        }
        
        // Retrieve user's secret
        let secret = self.secret_store.get_secret(user_id).await?
            .ok_or(MfaError::MfaNotEnabled)?;
        
        // Create TOTP verifier
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret,
            Some("MyApp".to_string()),
            user_id.to_string(),
        )?;
        
        // Verify with tolerance for clock skew
        let valid = totp.check_current(code)?;
        
        // Log verification attempt
        self.audit_log(user_id, "totp_verification", valid).await;
        
        Ok(valid)
    }
    
    /// Verify recovery code
    pub async fn verify_recovery_code(&self, user_id: &str, code: &str) -> Result<bool, MfaError> {
        let stored_codes = self.secret_store.get_recovery_codes(user_id).await?;
        
        for stored_code in stored_codes {
            if self.verify_recovery_hash(code, &stored_code) {
                // Remove used recovery code
                self.secret_store.remove_recovery_code(user_id, &stored_code).await?;
                self.audit_log(user_id, "recovery_code_used", true).await;
                return Ok(true);
            }
        }
        
        self.audit_log(user_id, "recovery_code_failed", false).await;
        Ok(false)
    }
    
    /// Generate cryptographically secure secret
    fn generate_secret(&self) -> Vec<u8> {
        let mut secret = vec![0u8; 32];
        thread_rng().fill(&mut secret[..]);
        secret
    }
    
    /// Generate recovery codes
    fn generate_recovery_codes(&self) -> Vec<String> {
        (0..10).map(|_| {
            let code: String = thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(8)
                .map(char::from)
                .collect();
            code.to_uppercase()
        }).collect()
    }
    
    /// Verify recovery code against hash
    fn verify_recovery_hash(&self, code: &str, hash: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        let computed_hash = format!("{:x}", hasher.finalize());
        computed_hash == hash
    }
    
    /// Audit log for security events
    async fn audit_log(&self, user_id: &str, event: &str, success: bool) {
        // Implementation would log to security audit system
        println!("AUDIT: user={} event={} success={}", user_id, event, success);
    }
}

#[derive(Debug)]
pub struct MfaSetupInfo {
    pub qr_code_url: String,
    pub recovery_codes: Vec<String>,
    pub secret: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum MfaError {
    #[error("MFA not enabled for user")]
    MfaNotEnabled,
    #[error("Rate limited")]
    RateLimited,
    #[error("TOTP error: {0}")]
    TotpError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
}

pub trait SecretStore: Send + Sync {
    async fn store_secret(&self, user_id: &str, secret: &[u8]) -> Result<(), MfaError>;
    async fn get_secret(&self, user_id: &str) -> Result<Option<Vec<u8>>, MfaError>;
    async fn store_recovery_codes(&self, user_id: &str, codes: &[String]) -> Result<(), MfaError>;
    async fn get_recovery_codes(&self, user_id: &str) -> Result<Vec<String>, MfaError>;
    async fn remove_recovery_code(&self, user_id: &str, code: &str) -> Result<(), MfaError>;
}

pub struct RateLimiter {
    // Implementation details...
}

impl RateLimiter {
    pub fn allow_attempt(&self, user_id: &str) -> bool {
        // Implementation would check rate limits
        true // Simplified
    }
}
"#.to_string(),
            location: Location::File { 
                path: "src/auth/mfa.rs".to_string(), 
                line_range: Some((1, 120)) 
            },
            created_at: chrono::Utc::now() - chrono::Duration::days(2),
            author: Some("dev@company.com".to_string()),
            metadata: HashMap::new(),
        },

        // API documentation
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Documentation,
            content: r#"
# Multi-Factor Authentication API

## Overview

Our MFA implementation provides TOTP-based two-factor authentication with SMS fallback and recovery codes.

## Endpoints

### Enable MFA
```
POST /api/v1/auth/mfa/enable
```

Enables MFA for the authenticated user and returns setup information.

**Response:**
```json
{
  "qr_code_url": "data:image/png;base64,iVBORw0KGgoAAAANS...",
  "recovery_codes": ["ABC12345", "DEF67890", ...],
  "manual_entry_key": "JBSWY3DPEHPK3PXP"
}
```

### Verify TOTP Code
```
POST /api/v1/auth/mfa/verify
```

Verifies a TOTP code from the user's authenticator app.

**Request:**
```json
{
  "code": "123456"
}
```

**Response:**
```json
{
  "valid": true,
  "message": "Code verified successfully"
}
```

### Use Recovery Code
```
POST /api/v1/auth/mfa/recovery
```

Uses a recovery code to bypass TOTP verification.

**Request:**
```json
{
  "recovery_code": "ABC12345"
}
```

## Security Features

- **Rate Limiting**: Max 5 verification attempts per minute per user
- **Audit Logging**: All MFA events are logged for security monitoring  
- **Encrypted Storage**: MFA secrets are encrypted at rest
- **Recovery Codes**: 10 single-use recovery codes generated
- **Clock Tolerance**: Accepts codes within 30-second window

## Implementation Status

- ✅ TOTP generation and verification
- ✅ QR code generation for easy setup
- ✅ Recovery code system
- ✅ Rate limiting protection
- ✅ Audit logging
- ❌ SMS fallback (planned for v2)
- ❌ Trusted device bypass (planned for v3)

## Error Handling

The API returns appropriate HTTP status codes:
- `200 OK`: Successful operation
- `400 Bad Request`: Invalid input
- `401 Unauthorized`: Authentication required
- `429 Too Many Requests`: Rate limited
- `500 Internal Server Error`: Server error
"#.to_string(),
            location: Location::File { 
                path: "docs/api/mfa.md".to_string(), 
                line_range: None 
            },
            created_at: chrono::Utc::now() - chrono::Duration::days(1),
            author: Some("tech-writer@company.com".to_string()),
            metadata: HashMap::new(),
        },

        // Recent commit
        Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Commit,
            content: r#"
feat: implement TOTP-based multi-factor authentication

- Add MfaService with TOTP generation and verification
- Implement secure secret storage with encryption
- Add recovery code system with proper hashing
- Include rate limiting for brute force protection
- Add comprehensive audit logging for security events
- Generate QR codes for easy authenticator app setup

Breaking changes:
- New database tables: mfa_secrets, mfa_recovery_codes
- New environment variables: MFA_ENCRYPTION_KEY

Security improvements:
- All MFA secrets encrypted at rest using AES-256
- Recovery codes are hashed with SHA-256
- Timing attack protection in verification logic
- Rate limiting prevents brute force attempts

Closes TICKET-456
"#.to_string(),
            location: Location::Commit { 
                hash: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(), 
                file_path: Some("src/auth/mfa.rs".to_string()) 
            },
            created_at: chrono::Utc::now() - chrono::Duration::hours(6),
            author: Some("dev@company.com".to_string()),
            metadata: HashMap::from([
                ("branch".to_string(), "feature/mfa-implementation".to_string()),
                ("pr_number".to_string(), "123".to_string()),
            ]),
        },
    ]
}

/// Set up AI agent orchestrator for verification
fn setup_ai_agents() -> AgentOrchestrator {
    let mut orchestrator = AgentOrchestrator::new();
    
    // Register agents for verification workflow
    orchestrator.register_agent(
        AgentType::CompilationAgent,
        Box::new(MockCompilationAgent::new()),
    );
    
    orchestrator.register_agent(
        AgentType::TestExecutionAgent,
        Box::new(MockTestExecutionAgent::new()),
    );
    
    orchestrator
}

/// Process a claim through the verification chain workflow
async fn process_claim_with_verification_chain(
    claim: &Claim,
    agent_orchestrator: &mut AgentOrchestrator,
) -> Result<ClaimVerificationResult, Box<dyn std::error::Error>> {
    let mut result = ClaimVerificationResult {
        claim_id: claim.id,
        verification_chain: claim.verification_chain.clone(),
        existing_code_found: vec![],
        tests_found: vec![],
        implementation_gaps: vec![],
        agent_tasks_completed: vec![],
        verification_status: VerificationStatus::InProgress,
    };

    // Step 1: Analyze verification chain if present
    if let Some(chain) = &claim.verification_chain {
        println!("  📊 Verification chain status: {:?}", chain.status);
        println!("  🔗 Missing links: {:?}", chain.missing_links);
        
        // Process each missing link
        for missing_link in &chain.missing_links {
            match missing_link {
                WorkItemType::ImplementRequirements => {
                    println!("    📝 Implementation needed");
                    result.implementation_gaps.push("Implementation required".to_string());
                }
                WorkItemType::CreateTests => {
                    println!("    🧪 Tests needed");
                    result.implementation_gaps.push("Tests required".to_string());
                }
                _ => {
                    println!("    ⚠️  Other work required: {:?}", missing_link);
                }
            }
        }
    }

    // Step 2: Search for existing implementations
    let existing_code = discover_existing_code_for_claim(claim).await?;
    result.existing_code_found = existing_code;

    // Step 3: Search for existing tests
    let existing_tests = discover_existing_tests_for_claim(claim).await?;
    result.tests_found = existing_tests;

    // Step 4: Use AI agents for verification tasks
    if !result.existing_code_found.is_empty() {
        let compile_task = create_compilation_task(&result.existing_code_found);
        let compile_result = agent_orchestrator.submit_task(compile_task).await?;
        result.agent_tasks_completed.push(compile_result);
        println!("    ✅ Compilation verification completed");
    }

    // Step 5: Determine final verification status
    result.verification_status = if result.implementation_gaps.is_empty() {
        VerificationStatus::Verified
    } else {
        VerificationStatus::ImplementationRequired
    };

    Ok(result)
}

/// Discover existing code that might implement the claim
async fn discover_existing_code_for_claim(claim: &Claim) -> Result<Vec<CodeReference>, Box<dyn std::error::Error>> {
    // In a real implementation, this would search the codebase
    // For this example, we'll simulate finding related code
    let code_refs = match claim.claim_type {
        ClaimType::Security => vec![
            CodeReference::new(
                CodeLocation::Git {
                    repository_url: "https://github.com/company/auth-service".to_string(),
                    commit_hash: "a1b2c3d4e5f6".to_string(),
                    file_path: "src/auth/mfa.rs".to_string(),
                    function_name: Some("verify_totp".to_string()),
                    line_range: Some((45, 65)),
                },
                ProgrammingLanguage::Rust,
                CodeType::Implementation,
            ),
        ],
        ClaimType::Functional => vec![
            CodeReference::new(
                CodeLocation::Git {
                    repository_url: "https://github.com/company/auth-service".to_string(),
                    commit_hash: "a1b2c3d4e5f6".to_string(),
                    file_path: "src/auth/mfa.rs".to_string(),
                    function_name: Some("enable_mfa".to_string()),
                    line_range: Some((20, 44)),
                },
                ProgrammingLanguage::Rust,
                CodeType::Implementation,
            ),
        ],
        _ => vec![],
    };

    Ok(code_refs)
}

/// Discover existing tests that might verify the claim
async fn discover_existing_tests_for_claim(claim: &Claim) -> Result<Vec<TestReference>, Box<dyn std::error::Error>> {
    // Simulate finding related tests
    if claim.statement.to_lowercase().contains("mfa") || claim.statement.to_lowercase().contains("totp") {
        Ok(vec![
            TestReference {
                id: uuid::Uuid::new_v4(),
                code_reference: CodeReference::new(
                    CodeLocation::Git {
                        repository_url: "https://github.com/company/auth-service".to_string(),
                        commit_hash: "a1b2c3d4e5f6".to_string(),
                        file_path: "tests/mfa_tests.rs".to_string(),
                        function_name: Some("test_totp_verification".to_string()),
                        line_range: Some((15, 30)),
                    },
                    ProgrammingLanguage::Rust,
                    CodeType::Test,
                ),
                test_name: "test_totp_verification".to_string(),
                test_type: TestType::Integration,
                test_specification: TestSpecification {
                    description: "Test TOTP code verification".to_string(),
                    inputs: vec![
                        TestInput {
                            name: "user_id".to_string(),
                            data_type: "String".to_string(),
                            value: "test_user_123".to_string(),
                            description: "User identifier for MFA verification".to_string(),
                        },
                        TestInput {
                            name: "totp_code".to_string(),
                            data_type: "String".to_string(),
                            value: "123456".to_string(),
                            description: "Six-digit TOTP code from authenticator".to_string(),
                        },
                    ],
                    expected_outputs: vec![
                        TestOutput {
                            name: "verification_result".to_string(),
                            data_type: "bool".to_string(),
                            expected_value: "true".to_string(),
                            validation_rule: "Must be boolean indicating verification success".to_string(),
                        },
                    ],
                    preconditions: vec!["User has MFA enabled".to_string()],
                    postconditions: vec!["Verification logged".to_string()],
                    edge_cases: vec![
                        EdgeCase {
                            scenario: "Invalid code format".to_string(),
                            inputs: vec![
                                TestInput {
                                    name: "totp_code".to_string(),
                                    data_type: "String".to_string(),
                                    value: "abc123".to_string(),
                                    description: "Non-numeric TOTP code".to_string(),
                                },
                            ],
                            expected_behavior: "Returns false".to_string(),
                            severity: sats_v2::code_references::RiskLevel::Medium,
                        },
                        EdgeCase {
                            scenario: "Expired code".to_string(),
                            inputs: vec![
                                TestInput {
                                    name: "totp_code".to_string(),
                                    data_type: "String".to_string(),
                                    value: "000000".to_string(),
                                    description: "Code past time window".to_string(),
                                },
                            ],
                            expected_behavior: "Returns false".to_string(),
                            severity: sats_v2::code_references::RiskLevel::High,
                        },
                    ],
                    performance_criteria: None,
                },
                related_implementations: vec![],
            },
        ])
    } else {
        Ok(vec![])
    }
}

/// Create a compilation task for AI agents
fn create_compilation_task(code_references: &[CodeReference]) -> AgentTask {
    AgentTask {
        id: uuid::Uuid::new_v4(),
        task_type: AgentType::CompilationAgent,
        description: "Verify code compilation for verification chain".to_string(),
        input: AgentTaskInput::CompileCode {
            code_references: code_references.to_vec(),
            build_configuration: BuildConfiguration {
                build_system: "cargo".to_string(),
                target: None,
                profile: Some("debug".to_string()),
                features: vec!["mfa".to_string()],
                environment_variables: HashMap::new(),
            },
        },
        expected_output: AgentTaskOutput::CompilationResult {
            success: true,
            artifacts: vec!["target/debug/auth-service".to_string()],
            errors: vec![],
            warnings: vec![],
        },
        constraints: TaskConstraints {
            max_execution_time: std::time::Duration::from_secs(300),
            max_memory_usage_mb: 1024,
            allowed_network_access: true,
            allowed_file_access: vec!["src/".to_string(), "Cargo.toml".to_string()],
            required_capabilities: vec!["rust-compilation".to_string()],
        },
        priority: TaskPriority::High,
        timeout: std::time::Duration::from_secs(600),
        created_at: chrono::Utc::now(),
    }
}

/// Display verification results
fn display_verification_result(result: &ClaimVerificationResult) {
    println!("  📊 Verification Results:");
    println!("    Existing Code: {}", result.existing_code_found.len());
    for code_ref in &result.existing_code_found {
        println!("      • {}", code_ref.location.display());
    }
    
    println!("    Tests Found: {}", result.tests_found.len());
    for test_ref in &result.tests_found {
        println!("      • {}", test_ref.test_name);
    }
    
    println!("    Implementation Gaps: {}", result.implementation_gaps.len());
    for gap in &result.implementation_gaps {
        println!("      • {}", gap);
    }
    
    println!("    AI Tasks: {}", result.agent_tasks_completed.len());
    for task_result in &result.agent_tasks_completed {
        let status = if task_result.success { "✅" } else { "❌" };
        println!("      {} {} ({}ms)", status, task_result.task_id, task_result.execution_time.as_millis());
    }
    
    println!("    Status: {:?}", result.verification_status);
}

// Helper types for this example

#[derive(Debug)]
struct ClaimVerificationResult {
    claim_id: uuid::Uuid,
    verification_chain: Option<VerificationChain>,
    existing_code_found: Vec<CodeReference>,
    tests_found: Vec<TestReference>,
    implementation_gaps: Vec<String>,
    agent_tasks_completed: Vec<AgentTaskResult>,
    verification_status: VerificationStatus,
}

#[derive(Debug)]
enum VerificationStatus {
    InProgress,
    Verified,
    ImplementationRequired,
    TestsRequired,
    Failed,
}