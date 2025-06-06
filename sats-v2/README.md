# SATS v2: Semantic Alignment Tracking System v2

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

SATS v2 transforms semantic alignment tracking from passive measurement into active work generation. Instead of just measuring whether code aligns with claims, SATS v2 discovers implementation gaps and generates concrete work items to close them.

## Key Innovation

SATS v2 tracks **implementation chains** for every claim:

```
Claim → Requirements → Implementation → Tests → Execution → Verification
```

Each broken link generates actionable work items that can be assigned to humans or AI agents.

## Features

- **🔗 Verification Chains**: Track complete implementation lifecycle for claims
- **🤖 Work Generation**: Automatically create concrete tasks from gaps
- **⚡ Execution Verification**: Actually run tests to verify claims
- **🧠 Semantic Analysis**: LLM-powered analysis of claim meaning vs test coverage
- **👥 Smart Assignment**: Assign work to AI agents or human developers
- **🏗️ Code Generation**: AI-powered implementation and test generation

## Quick Start

Add SATS v2 to your project:

```toml
[dependencies]
sats-v2 = "0.1.0"
```

Basic usage:

```rust
use sats_v2::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a claim from your code/tickets/commits
    let claim = Claim {
        statement: "User authentication validates passwords correctly".to_string(),
        claim_type: ClaimType::Security,
        // ... other fields
    };

    // Set up verification engine
    let verification_engine = VerificationEngine::new(config, extractors...);
    
    // Verify the claim and get work items for gaps
    let result = verification_engine.verify_claim(&claim).await?;
    
    match result.status {
        ChainStatus::NotStarted => {
            println!("Need to implement: {}", result.work_items[0].title);
        }
        ChainStatus::Verified => {
            println!("Claim fully verified!");
        }
        _ => {
            println!("Gaps found, {} work items generated", result.work_items.len());
        }
    }

    Ok(())
}
```

## Architecture

### Core Components

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Verification    │────▶│ Work Items      │────▶│ Execution       │
│ Engine          │     │ Manager         │     │ Engine          │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                       │                        │
         ▼                       ▼                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Semantic        │     │ Code            │     │ Types &         │
│ Analyzer        │     │ Generators      │     │ Storage         │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Verification Chain Status

- **NotStarted**: No implementation found → Generate implementation work item
- **NeedsTests**: Implementation exists, no tests → Generate test creation work item  
- **TestsFailing**: Tests exist but fail → Generate fix implementation work item
- **TestsInadequate**: Tests pass but don't verify claim → Generate improve tests work item
- **Verified**: Complete chain verified ✅

### Work Item Types

1. **ImplementRequirements**: Generate implementation code
2. **CreateTests**: Generate comprehensive test suite
3. **FixImplementation**: Fix failing implementation
4. **ImproveTests**: Enhance test coverage to match claims
5. **Documentation**: Update documentation
6. **Performance**: Optimize performance
7. **Security**: Add security measures

## Supported Languages

- **Rust**: Cargo, custom test runners
- **Python**: pytest, unittest
- **JavaScript**: Jest, Mocha, Jasmine
- **TypeScript**: ts-jest
- **Go**: go test
- **Java**: JUnit

## Configuration

Configure the verification engine:

```rust
let config = VerificationConfig {
    min_implementation_confidence: Confidence::new(0.7).unwrap(),
    min_test_coverage: Confidence::new(0.8).unwrap(),
    min_semantic_coverage: Confidence::new(0.8).unwrap(),
    enable_ai_generation: true,
    llm_endpoint: Some("https://api.anthropic.com/v1/messages".to_string()),
    ..Default::default()
};
```

## Examples

Run the basic usage example:

```bash
cargo run --example basic_usage
```

Expected output:
```
SATS v2 Basic Usage Example
===========================
Created claim: User authentication system validates passwords correctly

Verifying claim...
Verification Status: NotStarted
Work Items Generated: 1
  1. Implement: User authentication system validates passwords correctly (ImplementRequirements)
     Effort: 6/10, Skills: ["programming", "architecture"]

Managing work items...
Assigned work item to: AiAgent { agent_type: "CodeGen AI", capabilities: ["rust", "security"] }
Active assignments: 1
```

## API Documentation

### VerificationEngine

```rust
impl VerificationEngine {
    pub async fn verify_claim(&self, claim: &Claim) -> Result<VerificationResult, VerificationError>;
}
```

### WorkItemManager

```rust
impl WorkItemManager {
    pub async fn generate_work_item(
        &self, 
        work_type: WorkItemType, 
        claim: &Claim, 
        context: &WorkGenerationContext
    ) -> Result<WorkItem, WorkItemError>;
    
    pub async fn assign_work_item(&mut self, work_item: &WorkItem) -> Result<WorkItemAssignment, WorkItemError>;
}
```

### ExecutionEngine

```rust
impl ExecutionEngine {
    pub async fn execute_test_suite(&self, test_suite: &TestSuite) -> Result<ExecutionResult, ExecutionError>;
}
```

### SemanticAnalyzer

```rust
impl SemanticAnalyzer {
    pub async fn verify_claim_semantics(
        &self, 
        claim: &Claim, 
        test_suite: &TestSuite, 
        execution_result: &ExecutionResult
    ) -> Result<SemanticResult, SemanticError>;
}
```

## Development

### Building

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Run examples
cargo run --example basic_usage
```

### Project Structure

```
sats-v2/
├── src/
│   ├── lib.rs              # Main library exports
│   ├── types.rs            # Core type definitions
│   ├── verification.rs     # Verification chain engine
│   ├── work_items.rs       # Work item management
│   ├── execution.rs        # Test execution engine
│   ├── semantic.rs         # Semantic analysis
│   └── generators.rs       # Code generation
├── examples/
│   └── basic_usage.rs      # Basic usage example
└── templates/
    ├── rust_function.template
    └── rust_test.template
```

## Roadmap

### Phase 1: Foundation ✅
- [x] Core type system
- [x] Verification engine
- [x] Work item management
- [x] Basic execution engine
- [x] Semantic analysis framework

### Phase 2: Integration (In Progress)
- [ ] Real LLM integration
- [ ] Multi-language test runners
- [ ] CLI interface
- [ ] Git integration

### Phase 3: Advanced Features
- [ ] IDE plugins
- [ ] CI/CD integration  
- [ ] Performance optimization
- [ ] Enterprise features

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Citation

If you use SATS v2 in your research, please cite:

```bibtex
@software{sats_v2,
  title = {SATS v2: Semantic Alignment Tracking System v2},
  author = {SATS Team},
  year = {2024},
  url = {https://github.com/sats-project/sats-v2}
}
```

---

**SATS v2**: From claims to verified implementation, automatically. 🚀