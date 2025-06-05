# Real Claude-Powered SATS Analysis

This example demonstrates SATS using **live Claude API calls** to analyze a real multi-factor authentication (MFA) implementation.

## What This Shows

Unlike the mock example, this uses **actual Claude API calls** to:

1. **Extract Claims**: Claude analyzes real code, documentation, and tickets to identify implicit and explicit claims
2. **Check Alignments**: Claude evaluates how well evidence supports each claim with detailed reasoning
3. **Provide Insights**: Claude's natural language understanding identifies semantic relationships humans might miss

## Real-World Artifacts Analyzed

### 🎫 GitHub Issue #1247
Multi-factor authentication feature request with:
- User stories and acceptance criteria
- Technical requirements (JWT integration, rate limiting)
- Security considerations (encryption, audit logging)

### 💻 Implementation Code (`src/auth/mfa.rs`)
Real Rust code implementing:
- TOTP secret generation and verification
- Rate limiting (5 attempts per 5 minutes)
- Encrypted storage of secrets
- Audit logging for security events

### 🧪 Test Suite (`tests/auth/test_mfa.rs`)
Comprehensive tests covering:
- TOTP generation and verification flows
- SMS code workflows
- Rate limiting behavior
- Backup code generation
- Audit logging verification

### 📚 API Documentation (`docs/api/mfa.md`)
Production API documentation with:
- Endpoint specifications
- Request/response examples
- Rate limiting details
- Security feature status

### 📝 Git Commit
Real commit message describing:
- Features implemented
- Security improvements
- Remaining TODOs
- Issue resolution

## Running the Analysis

**Prerequisites:**
1. Set up your Claude API key using either method:

   **Option A: Using .env file (recommended)**
   ```bash
   cd sats-example
   cp .env.example .env
   # Edit .env and add your API key
   ```

   **Option B: Environment variable**
   ```bash
   export ANTHROPIC_API_KEY="your-api-key-here"
   ```

2. Run the analysis:
   ```bash
   cd sats-example
   cargo run --bin real_claude_analysis
   ```

## Sample Output

```
🤖 LIVE CLAUDE ANALYSIS
=======================

📄 Analyzing artifact 1/5: GitHub:1247
   ✅ Extracted 12 claims in 1847ms using claude-3-sonnet-20240229
   1. [Requirement] "System must support multi-factor authentication" (confidence: 0.95)
   2. [Requirement] "Users can enable/disable MFA in account settings" (confidence: 0.90)
   3. [Security] "MFA secrets must be encrypted at rest" (confidence: 0.92)
   ... and 9 more claims

📄 Analyzing artifact 2/5: src/auth/mfa.rs:1-85
   ✅ Extracted 8 claims in 2156ms using claude-3-sonnet-20240229
   1. [Functional] "System implements TOTP secret generation" (confidence: 0.88)
   2. [Security] "Secrets are encrypted using AES-256" (confidence: 0.85)
   3. [Behavioral] "Rate limiting prevents brute force attacks" (confidence: 0.82)
   ... and 5 more claims

🔍 ALIGNMENT ANALYSIS
=====================

🔗 Checking alignment 1: 
   Claim: "System must support multi-factor authentication"
   Evidence: src/auth/mfa.rs:1-85
   ✅ Alignment score: 0.89 - Strong
   💭 Claude's reasoning: The code directly implements an MfaManager struct with TOTP generation, verification...

🔗 Checking alignment 2: 
   Claim: "MFA secrets must be encrypted at rest"
   Evidence: src/auth/mfa.rs:1-85
   ✅ Alignment score: 0.91 - Strong
   💭 Claude's reasoning: The code shows explicit encryption of secrets using encrypt_secret() method and AES-256...

📊 ANALYSIS RESULTS
===================

🎯 PROJECT HEALTH SUMMARY:
   Total Claims Extracted: 47
   Total Alignments Checked: 188
   Average Alignment Score: 0.73
   Strong Alignments (≥0.7): 89 (47.3%)
   Moderate Alignments (0.4-0.7): 64 (34.0%)
   Weak Alignments (<0.4): 35 (18.6%)

🏆 STRONGEST ALIGNMENTS:
   1. Score: 0.94 - "TOTP codes have 30-second windows" ↔ src/auth/mfa.rs:1-85
   2. Score: 0.93 - "System implements rate limiting for MFA attempts" ↔ tests/auth/test_mfa.rs:1-95
   3. Score: 0.91 - "MFA secrets must be encrypted at rest" ↔ src/auth/mfa.rs:1-85
   4. Score: 0.89 - "System must support multi-factor authentication" ↔ src/auth/mfa.rs:1-85
   5. Score: 0.88 - "Backup codes generation for account recovery" ↔ docs/api/mfa.md

⚠️ POTENTIAL GAPS (Claims with weak evidence):
   1. [Requirement] "SMS service fallback capabilities" (best alignment: 0.12)
   2. [Requirement] "Hardware security key support (FIDO2/WebAuthn)" (best alignment: 0.15)
   3. [Security] "Admin override capabilities for MFA" (best alignment: 0.18)
   4. [Performance] "SMS rate limiting (3 per hour per phone)" (best alignment: 0.24)
   5. [Functional] "Bulk MFA operations for enterprise customers" (best alignment: 0.28)
```

## Key Insights from Claude Analysis

### 🎯 What Claude Discovered

1. **Strong Implementation Coverage**: Core MFA features (TOTP, rate limiting, encryption) show 85-94% alignment between requirements and implementation

2. **Documentation Gaps**: Some implemented features lack documentation, while some documented features aren't fully implemented

3. **Missing Features**: Claude identified several requirement claims with no supporting implementation:
   - SMS service fallback
   - Hardware security keys  
   - Admin override capabilities

4. **Test Coverage Analysis**: Claude found strong alignment between code and tests for core features, but weaker coverage for edge cases

### 🤖 Claude's Semantic Understanding

Claude's analysis goes beyond keyword matching:

- **Context Awareness**: Understands that "30-second windows" in requirements relates to TOTP time step verification in code
- **Implicit Claims**: Identifies security implications like "rate limiting prevents brute force attacks" from code structure
- **Cross-Artifact Reasoning**: Connects commit messages about features to actual implementations and tests

### 🔍 Advantages Over Traditional Analysis

1. **Natural Language Processing**: Understands human-written requirements and documentation
2. **Contextual Relationships**: Identifies semantic connections across different artifact types
3. **Quality Assessment**: Provides reasoning for alignment scores, not just binary matches
4. **Gap Detection**: Finds missing implementations, tests, or documentation based on claim analysis

## Performance Notes

- **API Calls**: Makes ~235 Claude API calls for this analysis (47 claim extractions + 188 alignments)
- **Processing Time**: Takes 5-8 minutes depending on API response times
- **Cost**: Approximately $3-5 in Claude API usage for this analysis
- **Rate Limiting**: Includes 100ms delays between calls to be respectful to the API

This demonstrates how SATS can provide deep, AI-powered analysis of software projects to maintain semantic consistency across the development lifecycle.