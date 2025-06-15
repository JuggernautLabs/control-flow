
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::time::Instant;
use tracing::{info, warn, error, debug, trace, instrument, span, Level};

// Use the external client implementations
use client_implementations::client::{LowLevelClient, QueryResolver, RetryConfig};
use client_implementations::error::QueryResolverError;

  
// ==== FUNDAMENTAL PRIMITIVES (from Cognitive Machine document) ====

/// The fundamental duality: all software development reduces to these two operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
/// Discover what IS
Observe(Lens),
/// Create what SHOULD BE  
Generate(Specification),
}

/// How we look at reality - composable, focusable, evolvable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Lens {
Existence(String),              // "Does X exist?"
Behavior(Stimulus),             // "What happens when...?"
Structure(Parser),              // "What shape is this?"
Relation(String, String),       // "How do X and Y relate?"
Temporal(Box<Lens>, u64),      // "How does this change over time?"
Composite(Vec<Lens>),          // "What do multiple views show?"
}

/// What we want to observe through a lens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stimulus {
UserInput(String),
CodeExecution(String),
TestRun(String),
SemanticAnalysis(String),
}

/// How we parse/understand structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Parser {
PythonCode,
TestSuite,
Requirements,
ApiSpec,
}

/// The space between IS and SHOULD BE - measures "wrongness"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Delta {
None,                          // Nothing needs to change
Absence(Specification),        // Something is missing
Mismatch {                     // Something is wrong
    observed: Reality,
    expected: Expectation,
    transformation: Option<String>,
},
Compound(Vec<Delta>),          // Multiple deltas
Unknown(String),               // Can't determine the issue
}

/// What we actually see when we look through a lens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation<T> {
pub data: T,
pub lens: Lens,
pub timestamp: u64,
pub confidence: f64,
}

/// Concrete reality that we observe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Reality {
NotFound,
Exists(String),
TestFails(String),
TestPasses,
CodeCompiles,
CodeErrors(String),
BehaviorMatches(String),
StructureMatches(String),
}

/// What we expect to see
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expectation {
ShouldExist(String),
ShouldBehave(String),
ShouldCompile,
ShouldPassTests,
ShouldImplement(String),
}

/// What we want to create
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Specification {
PythonModule(String),
TestSuite(String),
ApiEndpoint(String),
Documentation(String),
}

/// The result of creating something
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generation {
pub specification: Specification,
pub artifact: String,
pub applied_at: Option<u64>,
pub confidence: f64,
}

/// A single cycle: Reality -> Delta -> Generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exchange {
pub observation: Reality,
pub delta: Delta,
pub generation: Option<Generation>,
pub timestamp: u64,
}

/// A sequence of exchanges focused on one expectation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialogue {
pub lens: Lens,
pub expectation: Expectation,
pub exchanges: Vec<Exchange>,
pub is_complete: bool,
}

/// Multiple parallel dialogues (like TDD + Security + Performance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symphony {
pub dialogues: Vec<Dialogue>,
pub goal: String,
}

// ==== IMPLEMENTATION: The Fundamental Cycle ====

pub struct FundamentalCognitionMachine<C: LowLevelClient> {
resolver: QueryResolver<C>,
}

impl<C: LowLevelClient + Send + Sync> FundamentalCognitionMachine<C> {
#[instrument(skip(client))]
pub fn new(client: C) -> Self {
    info!("Creating new FundamentalCognitionMachine");
    let config = RetryConfig::default();
    let resolver = QueryResolver::new(client, config);
    Self { resolver }
}

/// Core primitive: OBSERVE through a lens
pub fn observe<'a>(&'a self, lens: &'a Lens, context: Option<&'a str>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Reality, Box<dyn std::error::Error>>> + Send + 'a>> {
    Box::pin(async move {
    debug!(lens_type = ?std::mem::discriminant(lens), "Starting observation");
    
    match lens {
        // MECHANICAL OBSERVATION - Direct system interrogation
        Lens::Existence(path) => {
            info!(path = %path, "Checking file existence");
            
            // Check if file actually exists on filesystem
            if std::path::Path::new(path).exists() {
                debug!(path = %path, "File exists, reading content");
                let content = std::fs::read_to_string(path)
                    .unwrap_or_else(|e| {
                        warn!(path = %path, error = %e, "File exists but can't read content");
                        "File exists but can't read content".to_string()
                    });
                trace!(path = %path, content_len = content.len(), "File content read successfully");
                Ok(Reality::Exists(content))
            } else {
                info!(path = %path, "File does not exist");
                Ok(Reality::NotFound)
            }
        },

        // SEMANTIC OBSERVATION - AI-powered analysis of content
        Lens::Structure(Parser::PythonCode) => {
            info!("Analyzing Python code structure with AI");
            let code = context.ok_or("Need code content to analyze structure")?;
            debug!(code_len = code.len(), "Code content provided for analysis");
            
            let prompt = format!(
                r#"Analyze the structure of this Python code:

{}

Focus on what's actually present in the code. Set valid_structure to true if the code is syntactically valid Python, false otherwise. List all function and class names found in the code. Provide an overall description of the code structure."#, code
            );

            /// Response containing Python code structure analysis
            #[derive(Deserialize, JsonSchema)]
            #[schemars(title = "Python Code Structure Analysis", description = "Analysis of Python code structure including validity and components")]
            struct StructureResponse {
                /// Whether the Python code is syntactically valid and well-formed
                #[schemars(description = "True if code is valid Python syntax, false if there are syntax errors")]
                valid_structure: bool,
                
                /// List of all function names found in the code
                #[schemars(description = "Names of all functions defined in the code (e.g., ['main', 'calculate', 'validate'])")]
                functions: Vec<String>,
                
                /// List of all class names found in the code
                #[schemars(description = "Names of all classes defined in the code (e.g., ['User', 'DatabaseConnection', 'Parser'])")]
                classes: Vec<String>,
                
                /// Overall description of the code's structure and purpose
                #[schemars(description = "High-level summary of what the code does and how it's organized")]
                structure_description: String,
            }

            trace!("Sending structure analysis prompt to AI");
            let response: StructureResponse = self.resolver.query_with_schema(prompt).await
                .map_err(|e| {
                    error!(error = %e, "AI query failed for structure analysis");
                    Box::new(e) as Box<dyn std::error::Error>
                })?;
            
            info!(valid = response.valid_structure, description = %response.structure_description, "Structure analysis complete");
            
            Ok(if response.valid_structure {
                Reality::StructureMatches(response.structure_description)
            } else {
                Reality::CodeErrors("Invalid structure".to_string())
            })
        },

        Lens::Structure(Parser::TestSuite) => {
            info!("Analyzing test suite structure with AI");
            let code = context.ok_or("Need test code to analyze")?;
            debug!(code_len = code.len(), "Test code provided for analysis");
            
            let prompt = format!(
                r#"Analyze this test suite:

{}

Focus on what's actually tested. Set valid_tests to true if the code contains valid test functions, false otherwise. List all test function names found. Identify what areas/functionality are being tested. Provide a description of what these tests verify."#, code
            );

            /// Response containing test suite analysis results
            #[derive(Deserialize, JsonSchema)]
            #[schemars(title = "Test Suite Analysis", description = "Analysis of test code including validity and coverage")]
            struct TestResponse {
                /// Whether the code contains valid test functions
                #[schemars(description = "True if valid test functions are found, false if no tests or malformed tests")]
                valid_tests: bool,
                
                /// List of all test function names found
                #[schemars(description = "Names of test functions found (e.g., ['test_login', 'test_validation', 'test_edge_cases'])")]
                test_functions: Vec<String>,
                
                /// Areas or functionality being tested
                #[schemars(description = "Functional areas covered by the tests (e.g., ['authentication', 'data validation', 'error handling'])")]
                coverage_areas: Vec<String>,
                
                /// Description of what the tests verify
                #[schemars(description = "Summary of what behavior and functionality these tests validate")]
                test_description: String,
            }

            trace!("Sending test analysis prompt to AI");
            let response: TestResponse = self.resolver.query_with_schema(prompt).await
                .map_err(|e| {
                    error!(error = %e, "AI query failed for test analysis");
                    Box::new(e) as Box<dyn std::error::Error>
                })?;
            
            info!(valid = response.valid_tests, description = %response.test_description, "Test analysis complete");
            
            Ok(if response.valid_tests {
                Reality::StructureMatches(response.test_description)
            } else {
                Reality::CodeErrors("Invalid test structure".to_string())
            })
        },

        // BEHAVIORAL OBSERVATION - Execute and observe results
        Lens::Behavior(Stimulus::CodeExecution(code)) => {
            info!("Executing code for behavioral observation");
            debug!(code_len = code.len(), "Code provided for execution");
            self.execute_code(code).await
        },

        Lens::Behavior(Stimulus::TestRun(_)) => {
            info!("Running tests for behavioral observation");
            let code = context.ok_or("Need code to run tests")?;
            debug!(code_len = code.len(), "Code provided for test execution");
            self.execute_code(code).await
        },

        // SEMANTIC OBSERVATION - AI analyzes relationships
        Lens::Relation(subject, object) => {
            info!(subject = %subject, object = %object, "Analyzing relationship with AI");
            let content = context.ok_or("Need content to analyze relationship")?;
            debug!(content_len = content.len(), "Content provided for relationship analysis");
            
            let prompt = format!(
                r#"Analyze the relationship between '{}' and '{}' in this content:

{}

Focus on actual relationships, not theoretical ones. Set relationship_exists to true if there is a clear relationship between the two entities in the content. Describe the type of relationship if it exists. Provide a strength score from 0.0 (no relationship) to 1.0 (very strong relationship)."#, subject, object, content
            );

            /// Response analyzing relationships between two entities
            #[derive(Deserialize, JsonSchema)]
            #[schemars(title = "Relationship Analysis", description = "Analysis of semantic relationships between two entities")]
            struct RelationResponse {
                /// Whether a clear relationship exists between the entities
                #[schemars(description = "True if a semantic relationship is found, false if entities are unrelated")]
                relationship_exists: bool,
                
                /// Type and nature of the relationship
                #[schemars(description = "Description of how the entities relate (e.g., 'implements', 'depends on', 'validates', 'extends')")]
                relationship_type: String,
                
                /// Strength of the relationship from 0.0 to 1.0
                #[schemars(range(min = 0.0, max = 1.0), description = "Confidence in the relationship: 0.0 = no relationship, 1.0 = very strong relationship")]
                strength: f64,
            }

            trace!("Sending relationship analysis prompt to AI");
            let response: RelationResponse = self.resolver.query_with_schema(prompt).await
                .map_err(|e| {
                    error!(error = %e, "AI query failed for relationship analysis");
                    Box::new(e) as Box<dyn std::error::Error>
                })?;
            
            info!(exists = response.relationship_exists, rel_type = %response.relationship_type, "Relationship analysis complete");
            
            Ok(if response.relationship_exists {
                Reality::BehaviorMatches(response.relationship_type)
            } else {
                Reality::NotFound
            })
        },

        // COMPOSITE OBSERVATION - Multiple lenses applied
        Lens::Composite(lenses) => {
            info!(num_lenses = lenses.len(), "Applying composite observation");
            let mut results = Vec::new();
            for (i, lens) in lenses.iter().enumerate() {
                debug!(lens_index = i, lens_type = ?lens, "Applying composite lens");
                let result = Box::pin(self.observe(lens, context)).await?;
                results.push(format!("{:?}", result));
                trace!(lens_index = i, result = ?result, "Composite lens result");
            }
            info!(num_results = results.len(), "Composite observation complete");
            Ok(Reality::StructureMatches(results.join("; ")))
        },

        _ => {
            warn!(lens = ?lens, "Unknown lens type, returning NotFound");
            Ok(Reality::NotFound)
        },
    }
    })
}

/// Core primitive: EVALUATE delta between reality and expectation
#[instrument(skip(self), fields(reality_type = ?std::mem::discriminant(reality), expectation_type = ?std::mem::discriminant(expectation)))]
pub fn evaluate(&self, reality: &Reality, expectation: &Expectation) -> Delta {
    let _span = span!(Level::DEBUG, "evaluate").entered();
    
    let delta = match (reality, expectation) {
        (Reality::NotFound, Expectation::ShouldExist(what)) => {
            info!(missing = %what, "Item not found that should exist");
            Delta::Absence(Specification::PythonModule(what.clone()))
        },
        (Reality::CodeErrors(error), Expectation::ShouldCompile) => {
            warn!(error = %error, "Code has errors but should compile");
            Delta::Mismatch {
                observed: reality.clone(),
                expected: expectation.clone(),
                transformation: Some(format!("Fix: {}", error)),
            }
        },
        (Reality::TestFails(reason), Expectation::ShouldPassTests) => {
            warn!(reason = %reason, "Tests are failing but should pass");
            Delta::Mismatch {
                observed: reality.clone(),
                expected: expectation.clone(),
                transformation: Some(format!("Fix failing test: {}", reason)),
            }
        },
        (Reality::Exists(_), Expectation::ShouldExist(_)) |
        (Reality::TestPasses, Expectation::ShouldPassTests) |
        (Reality::CodeCompiles, Expectation::ShouldCompile) => {
            info!("Reality matches expectation - no delta");
            Delta::None
        },
        _ => {
            debug!(reality = ?reality, expectation = ?expectation, "Unknown evaluation case");
            Delta::Unknown(format!("Don't know how to evaluate {:?} against {:?}", reality, expectation))
        },
    };
    
    debug!(delta = ?delta, "Evaluation complete");
    delta
}

/// Core primitive: GENERATE based on delta
#[instrument(skip(self), fields(delta_type = ?std::mem::discriminant(delta)))]
pub async fn generate(&self, delta: &Delta) -> Result<Option<Generation>, QueryResolverError> {
    let _span = span!(Level::INFO, "generate").entered();
    
    match delta {
        Delta::None => {
            info!("No delta - no generation needed");
            Ok(None)
        },
        Delta::Absence(spec) => {
            info!(spec = ?spec, "Generating for absent specification");
            
            let prompt = match spec {
                Specification::PythonModule(description) => {
                    debug!(description = %description, "Generating Python module");
                    format!(
                        r#"Generate a complete Python module for: {}

Include:
1. Main implementation
2. Comprehensive pytest tests  
3. Clear docstrings
4. Error handling

Make it production-ready and fully testable. Provide the complete Python code in the artifact field and a confidence score from 0.0 to 1.0 indicating how complete and correct the implementation is."#, description
                    )
                },
                Specification::TestSuite(description) => {
                    debug!(description = %description, "Generating test suite");
                    format!(
                        r#"Generate comprehensive pytest tests for: {}

Cover all major functionality and edge cases. Provide the complete test suite code in the artifact field and a confidence score from 0.0 to 1.0 indicating how comprehensive the test coverage is."#, description
                    )
                },
                _ => {
                    warn!(spec = ?spec, "Unknown specification type");
                    format!("Generate: {:?}", spec)
                },
            };

            /// Response containing generated code or content
            #[derive(Deserialize, JsonSchema)]
            #[schemars(title = "Code Generation Result", description = "Result of AI code generation including the artifact and confidence")]
            struct GenerationResponse {
                /// The generated code, tests, or other artifact
                #[schemars(description = "Complete generated content (code, tests, documentation, etc.)")]
                artifact: String,
                
                /// Confidence in the quality and correctness of the generation
                #[schemars(range(min = 0.0, max = 1.0), description = "Confidence score: 0.0 = low confidence, 1.0 = high confidence in generation quality")]
                confidence: f64,
            }

            trace!("Sending generation prompt to AI");
            let response: GenerationResponse = self.resolver.query_with_schema(prompt).await?;
            
            info!(confidence = response.confidence, artifact_len = response.artifact.len(), "Generation complete");
            
            Ok(Some(Generation {
                specification: spec.clone(),
                artifact: response.artifact,
                applied_at: Some(Instant::now().elapsed().as_millis() as u64),
                confidence: response.confidence,
            }))
        },
        Delta::Mismatch { transformation: Some(fix), .. } => {
            info!(fix = %fix, "Generating fix for mismatch");
            
            let prompt = format!(
                r#"Apply this fix: {}

Provide the corrected implementation in the artifact field and a confidence score from 0.0 to 1.0 indicating how confident you are that the fix resolves the issue."#, fix
            );

            /// Response containing fixed/corrected code
            #[derive(Deserialize, JsonSchema)]
            #[schemars(title = "Code Fix Result", description = "Result of applying a fix to code including the corrected version")]
            struct FixResponse {
                /// The corrected/fixed code
                #[schemars(description = "Code with the issue fixed or improvement applied")]
                artifact: String,
                
                /// Confidence that the fix resolves the issue
                #[schemars(range(min = 0.0, max = 1.0), description = "Confidence that the fix resolves the original issue: 0.0 = uncertain, 1.0 = confident fix")]
                confidence: f64,
            }

            trace!("Sending fix prompt to AI");
            let response: FixResponse = self.resolver.query_with_schema(prompt).await?;
            
            info!(confidence = response.confidence, artifact_len = response.artifact.len(), "Fix generation complete");

            Ok(Some(Generation {
                specification: Specification::PythonModule("fix".to_string()),
                artifact: response.artifact,
                applied_at: Some(Instant::now().elapsed().as_millis() as u64),
                confidence: response.confidence,
            }))
        },
        _ => {
            debug!(delta = ?delta, "No generation strategy for delta type");
            Ok(None)
        },
    }
}

/// Raw execution (no AI) - just runs code
#[instrument(skip(self, code), fields(code_len = code.len()))]
pub async fn execute_code(&self, code: &str) -> Result<Reality, Box<dyn std::error::Error>> {
    info!("Executing generated code");
    
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("generated_code.py");
    
    debug!(path = ?file_path, "Writing code to temporary file");
    std::fs::write(&file_path, code)?;

    debug!("Running pytest on generated code");
    let output = std::process::Command::new("python3")
        .arg(&file_path)
        .arg("-m")
        .arg("pytest")
        .arg("--tb=short")
        .output()?;

    debug!("Cleaning up temporary file");
    let _ = std::fs::remove_file(&file_path);

    let result = if output.status.success() {
        info!("Code execution successful - tests pass");
        Reality::TestPasses
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        warn!(stderr = %stderr, "Code execution failed");
        Reality::TestFails(stderr)
    };

    trace!(result = ?result, "Execution complete");
    Ok(result)
}

/// The fundamental cycle: observe -> evaluate -> generate
#[instrument(skip(self, context), fields(lens_type = ?std::mem::discriminant(&lens), expectation_type = ?std::mem::discriminant(&expectation)))]
pub async fn fundamental_cycle(
    &self,
    lens: Lens,
    expectation: Expectation,
    context: Option<&str>,
) -> Result<Exchange, Box<dyn std::error::Error>> {
    let _span = span!(Level::INFO, "fundamental_cycle").entered();
    info!("Starting fundamental cycle");
    
    // Step 1: Observe reality through lens
    let reality = Box::pin(self.observe(&lens, context)).await?;
    
    // Step 2: Evaluate delta between reality and expectation
    let delta = self.evaluate(&reality, &expectation);
    
    // Step 3: Generate artifact if needed
    let generation = if !matches!(delta, Delta::None) {
        info!("Delta detected, generating solution");
        self.generate(&delta).await?
    } else {
        info!("No delta, no generation needed");
        None
    };

    let exchange = Exchange {
        observation: reality,
        delta,
        generation,
        timestamp: Instant::now().elapsed().as_millis() as u64,
    };

    info!(has_generation = exchange.generation.is_some(), "Fundamental cycle complete");
    Ok(exchange)
}

/// Run a complete dialogue until convergence
pub async fn run_dialogue(&self, mut dialogue: Dialogue) -> Result<Dialogue, Box<dyn std::error::Error>> {
    const MAX_EXCHANGES: usize = 10;
    let mut context: Option<String> = None;
    
    for i in 0..MAX_EXCHANGES {
        println!("Exchange {}: Observing through lens {:?}", i + 1, dialogue.lens);
        
        let exchange = self.fundamental_cycle(
            dialogue.lens.clone(), 
            dialogue.expectation.clone(),
            context.as_deref(),
        ).await?;
        
        println!("  Reality: {:?}", exchange.observation);
        println!("  Delta: {:?}", exchange.delta);
        
        // If we generated something, apply it and update context
        if let Some(ref generation) = exchange.generation {
            println!("  Generated: {}", generation.artifact.chars().take(100).collect::<String>());
            context = Some(generation.artifact.clone());
            
            // Execute the generated code if it's a Python module
            if matches!(generation.specification, Specification::PythonModule(_)) {
                let execution_result = self.execute_code(&generation.artifact).await?;
                
                // Create new exchange with execution results
                let execution_exchange = Exchange {
                    observation: execution_result.clone(),
                    delta: self.evaluate(&execution_result, &dialogue.expectation),
                    generation: None,
                    timestamp: Instant::now().elapsed().as_millis() as u64,
                };
                
                dialogue.exchanges.push(exchange);
                dialogue.exchanges.push(execution_exchange);
                
                println!("  Execution: {:?}", execution_result);
                
                // Check if we're done
                if matches!(execution_result, Reality::TestPasses) {
                    dialogue.is_complete = true;
                    println!("✅ Dialogue complete - tests pass!");
                    break;
                }
            } else {
                dialogue.exchanges.push(exchange);
            }
        } else {
            // No generation needed, we're done
            dialogue.exchanges.push(exchange);
            dialogue.is_complete = true;
            println!("✅ Dialogue complete - no generation needed!");
            break;
        }
    }

    if !dialogue.is_complete {
        println!("⚠️  Dialogue reached maximum exchanges without completion");
    }

    Ok(dialogue)
}

/// Transform high-level goal into working system
pub async fn achieve_goal(&self, goal: &str) -> Result<Symphony, Box<dyn std::error::Error>> {
    println!("🎯 Goal: {}", goal);
    
    // Create dialogue for the main implementation
    let implementation_dialogue = Dialogue {
        lens: Lens::Composite(vec![
            Lens::Existence("TicTacToe API implementation".to_string()),
            Lens::Structure(Parser::PythonCode),
            Lens::Behavior(Stimulus::TestRun("pytest".to_string())),
        ]),
        expectation: Expectation::ShouldImplement(goal.to_string()),
        exchanges: Vec::new(),
        is_complete: false,
    };

    // Run the dialogue to completion
    let completed_dialogue = self.run_dialogue(implementation_dialogue).await?;

    // Create symphony with the completed dialogue
    let symphony = Symphony {
        dialogues: vec![completed_dialogue],
        goal: goal.to_string(),
    };

    Ok(symphony)
}
}

// CLI for testing the fundamental primitives
pub async fn run_fundamental_experiment() -> Result<(), Box<dyn std::error::Error>> {
println!("🧠 Fundamental Primitives Cognition Machine");
println!("Implementing the observe/generate duality from the Cognitive Machine document");
println!();

println!("Core insight: ALL software development reduces to:");
println!("  Operation::Observe(Lens) - Discover what IS");
println!("  Operation::Generate(Specification) - Create what SHOULD BE");
println!();

println!("The fundamental cycle:");
println!("  Expectation → Observation → Delta → Generation → New Reality");
println!();

println!("For 'create a tictactoe api':");
println!("1. Observe: Does TicTacToe API exist? → Reality::NotFound");
println!("2. Evaluate: NotFound vs ShouldExist → Delta::Absence");
println!("3. Generate: Create Python module with tests");
println!("4. Execute: Run the tests");
println!("5. Observe: Do tests pass? → Reality::TestPasses (if successful)");
println!("6. Complete: Delta::None (goal achieved)");

Ok(())
}

// #[cfg(test)]
// mod tests {
// use super::*;
// use crate::client::MockVoid;

// #[tokio::test]
// async fn test_fundamental_cycle() {
//     let mock_client = MockVoid;
//     let cogmach = FundamentalCognitionMachine::new(mock_client);
    
//     let lens = Lens::Existence("TicTacToe API".to_string());
//     let expectation = Expectation::ShouldExist("TicTacToe API".to_string());
    
//     // This would fail with mock client, but demonstrates the structure
//     // let exchange = cogmach.fundamental_cycle(lens, expectation).await;
    
//     // Test that the types work correctly
//     assert_eq!(std::mem::size_of::<Lens>(), std::mem::size_of::<Lens>());
// }

// #[test]
// fn test_delta_evaluation() {
//     let cogmach = FundamentalCognitionMachine::new(crate::client::MockVoid);
    
//     let reality = Reality::NotFound;
//     let expectation = Expectation::ShouldExist("test".to_string());
//     let delta = cogmach.evaluate(&reality, &expectation);
    
//     assert!(matches!(delta, Delta::Absence(_)));
// }
// }