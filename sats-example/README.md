# SATS Example: OAuth2 Authentication Analysis

This example demonstrates the Semantic Alignment Tracking System (SATS) analyzing a simple OAuth2 authentication system.

## What This Example Shows

The SATS system analyzes relationships between different types of project artifacts:

### 📋 Sample Artifacts Analyzed

1. **JIRA Ticket (JIRA-123)**: OAuth2 authentication requirements
2. **Implementation Code** (`src/auth/oauth2.rs`): OAuth2Handler implementation
3. **Test Code** (`tests/oauth2_test.rs`): Unit tests for OAuth2 functionality  
4. **Documentation** (`docs/authentication.md`): API documentation

### 🔍 What SATS Extracted

The system automatically extracted **15 claims** across different types:

- **Requirements** (from ticket): "Must support OAuth2", "Must handle token refresh", "Must rate limit", etc.
- **Functional Claims** (from code): "Implements OAuth2 handler", "Can initiate auth flow", etc.
- **Testing Claims** (from tests): "OAuth2 flow is tested", "Token exchange is tested", etc.
- **Documentation Claims** (from docs): "Supports OAuth2", "Implements token refresh", etc.

### ⚖️ Alignment Analysis

SATS checked **45 alignment relationships** between claims and evidence:

- **Strong Alignments (0.8+)**: OAuth2 core functionality is well-implemented and tested
- **Weak Alignments (0.3-0.7)**: Token refresh exists but lacks comprehensive testing
- **Very Weak (0.1-0.3)**: Rate limiting is required but completely missing

### 🚨 Gaps Detected

The system found **10 gaps**, including:

- **Rate limiting**: Required in ticket but not implemented anywhere
- **Token refresh testing**: Mentioned in code but tests are incomplete (marked as TODO)
- **Session timeout**: Required but hardcoded, not configurable

## Key Insights Demonstrated

### 1. **Requirement Traceability**
```
JIRA-123: "Support OAuth2 authorization code flow"
    ↓ 85% alignment
src/auth/oauth2.rs: OAuth2Handler implementation
    ↓ 90% alignment  
tests/oauth2_test.rs: test_start_auth_flow()
```

### 2. **Gap Detection**
```
JIRA-123: "Rate limit authentication attempts"
    ↓ 10% alignment (MISSING!)
❌ No implementation found in codebase
❌ No tests found  
❌ Documentation acknowledges it's "not implemented yet"
```

### 3. **Inconsistency Detection**
```
docs/authentication.md: "Automatic token refresh ✅"
    ↓ 70% alignment
src/auth/oauth2.rs: refresh_token field exists
    ↓ 20% alignment  
tests/oauth2_test.rs: "TODO: Add test for token refresh"
```

## Running the Example

```bash
cd sats-example
cargo run
```

## Running with Real Claude API

To use the live Claude-powered analysis:

1. **Setup API key:**
   ```bash
   cp .env.example .env
   # Edit .env and add your Claude API key
   ```

2. **Run analysis:**
   ```bash
   cargo run --bin real_claude_analysis
   ```

## Sample Output

```
🎯 SATS ANALYSIS RESULTS
========================

📈 PROJECT HEALTH OVERVIEW:
  Total Claims: 15
  Fully Supported: 5 (33.3%)
  Partially Supported: 2 (13.3%)  
  Unsupported: 8 (53.3%)
  Average Alignment Score: 0.37

🚨 GAPS BY SEVERITY:
  Medium: 10

💡 KEY INSIGHTS:
  • Rate limiting is required but not implemented or tested
  • 2 features are planned but have incomplete test coverage
  • Token refresh appears to be implemented and documented but lacks test coverage

🎯 RECOMMENDATIONS:
  1. Implement rate limiting for authentication attempts
  2. Add comprehensive tests for token refresh functionality  
  3. Add tests for rate limiting once implemented
  4. Update documentation to reflect current implementation status
```

## What Makes This Powerful

Unlike traditional static analysis tools, SATS:

1. **Understands Natural Language**: Extracts claims from English text in tickets, comments, and docs
2. **Cross-Artifact Analysis**: Finds relationships between requirements, code, tests, and documentation
3. **Semantic Alignment**: Measures how well evidence supports claims, not just syntax
4. **Gap Detection**: Identifies missing implementations, tests, or documentation
5. **Actionable Insights**: Provides specific recommendations for improving alignment

This demonstrates how SATS can help maintain consistency across the entire software development lifecycle, from requirements to deployment.