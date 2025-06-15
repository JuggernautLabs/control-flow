use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

// Re-use the existing client interface from client.rs
use crate::client::{LowLevelClient, QueryResolver, RetryConfig};
use crate::error::{QueryResolverError, AIError};

// Raw execution result - no AI interpretation
// Core Primitives: Only 3 needed for complete generation cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Observation {
    UserQuery {
        question: String,
        expected_type: String, // "requirement", "preference", "constraint"
        context: String,
    },
    TestSpecReflection {
        tests: String,
        spec_extracted: String,
        alignment_score: f64,
    },
    ExecutionResult {
        stdout: String,
        stderr: String,
        exit_code: i32,
        success: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generation {
    pub content: String,
    pub generation_type: String, // "question", "code", "test", "analysis"
    pub confidence: f64,
    pub dependencies: Vec<String>, // What this generation depends on
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveState {
    pub observations: Vec<Observation>,
    pub generations: Vec<Generation>,
    pub current_phase: String,
    pub goal: String,
    pub is_complete: bool,
}

// The Three Core Primitives
pub struct CognitionMachine<C: LowLevelClient> {
    resolver: QueryResolver<C>,
}

impl<C: LowLevelClient + Send + Sync> CognitionMachine<C> {
    pub fn new(client: C) -> Self {
        let config = RetryConfig::default();
        let resolver = QueryResolver::new(client, config);
        Self { resolver }
    }

    // PRIMITIVE 1: OBSERVE - Extract information from any source using AI
    pub async fn observe(&self, input: &str, observation_type: &str) -> Result<Observation, QueryResolverError> {
        let prompt = match observation_type {
            "user_query" => format!(
                r#"Given this user input: "{}"

Generate a clarifying question to gather missing requirements. Focus on ONE specific aspect that's unclear.

Respond with JSON:
{{
    "question": "specific question about missing requirement",
    "expected_type": "requirement|preference|constraint", 
    "context": "why this question is important for implementation"
}}

Be concise and actionable."#, input
            ),
            "test_spec_reflection" => format!(
                r#"Analyze this test code to extract the implicit specification:

{}

What behavior does this test expect? Extract the specification that these tests imply.

Respond with JSON:
{{
    "tests": "the provided test code",
    "spec_extracted": "clear specification implied by the tests",
    "alignment_score": 0.0-1.0
}}

Focus on what the tests prove the system should do."#, input
            ),
            "execution_analysis" => format!(
                r#"Analyze this execution output and determine what happened:

{}

Identify:
- Did the program run successfully?
- What errors occurred?
- What does the output tell us about correctness?
- Are there any patterns or insights?

Respond with JSON:
{{
    "stdout": "stdout content",
    "stderr": "stderr content", 
    "exit_code": 0,
    "success": true
}}

Extract the execution details and assess success."#, input
            ),
            _ => return Err(QueryResolverError::Other("Unknown observation type".to_string())),
        };

        match observation_type {
            "user_query" => {
                #[derive(Deserialize)]
                struct UserQueryResponse {
                    question: String,
                    expected_type: String,
                    context: String,
                }
                
                let response: UserQueryResponse = self.resolver.query(prompt).await?;
                Ok(Observation::UserQuery {
                    question: response.question,
                    expected_type: response.expected_type,
                    context: response.context,
                })
            },
            "test_spec_reflection" => {
                #[derive(Deserialize)]
                struct TestSpecResponse {
                    tests: String,
                    spec_extracted: String,
                    alignment_score: f64,
                }
                
                let response: TestSpecResponse = self.resolver.query(prompt).await?;
                Ok(Observation::TestSpecReflection {
                    tests: response.tests,
                    spec_extracted: response.spec_extracted,
                    alignment_score: response.alignment_score,
                })
            },
            "execution_analysis" => {
                #[derive(Deserialize)]
                struct ExecutionAnalysisResponse {
                    stdout: String,
                    stderr: String,
                    exit_code: i32,
                    success: bool,
                }
                
                let response: ExecutionAnalysisResponse = self.resolver.query(prompt).await?;
                Ok(Observation::ExecutionResult {
                    stdout: response.stdout,
                    stderr: response.stderr,
                    exit_code: response.exit_code,
                    success: response.success,
                })
            },
            _ => unreachable!(),
        }
    }

    // PRIMITIVE 2: GENERATE - Create content based on observations
    pub async fn generate(&self, observations: &[Observation], generation_type: &str) -> Result<Generation, QueryResolverError> {
        let context = self.build_context(observations);
        
        let prompt = match generation_type {
            "requirements_questions" => format!(
                r#"Based on this context: {}

Generate 3-5 focused questions to clarify the requirements for building this system.
Each question should target a specific implementation decision.

Respond with JSON:
{{
    "content": "question1\nquestion2\nquestion3...",
    "generation_type": "requirements_questions",
    "confidence": 0.0-1.0,
    "dependencies": ["user_input", "goal_analysis"]
}}

Questions should be answerable and lead to implementable specifications."#, context
            ),
            "python_implementation" => format!(
                r#"Based on this context: {}

Generate a complete Python implementation that satisfies all requirements.
Include:
1. A main implementation file
2. Comprehensive tests using pytest
3. Clear function signatures and docstrings

The code should be production-ready and fully testable.

Respond with JSON:
{{
    "content": "complete python code with tests",
    "generation_type": "python_implementation", 
    "confidence": 0.0-1.0,
    "dependencies": ["requirements", "architecture_decisions"]
}}

Make it work correctly and completely."#, context
            ),
            "architecture_analysis" => format!(
                r#"Based on this context: {}

Analyze the requirements and determine the optimal architecture approach.
Consider API design, data structures, and testing strategy.

Respond with JSON:
{{
    "content": "architectural decisions and rationale",
    "generation_type": "architecture_analysis",
    "confidence": 0.0-1.0, 
    "dependencies": ["requirements_analysis"]
}}

Focus on implementable architectural decisions."#, context
            ),
            _ => return Err(QueryResolverError::Other("Unknown generation type".to_string())),
        };

        #[derive(Deserialize)]
        struct GenerationResponse {
            content: String,
            generation_type: String,
            confidence: f64,
            dependencies: Vec<String>,
        }

        let response: GenerationResponse = self.resolver.query(prompt).await?;
        Ok(Generation {
            content: response.content,
            generation_type: response.generation_type,
            confidence: response.confidence,
            dependencies: response.dependencies,
        })
    }

    // Raw execution - no AI involved, just runs code
    pub async fn execute_code(&self, code: &str) -> Result<RawExecutionResult, Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("generated_code.py");
        std::fs::write(&file_path, code)?;

        let output = std::process::Command::new("python3")
            .arg(&file_path)
            .output()?;

        let _ = std::fs::remove_file(&file_path);

        Ok(RawExecutionResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            success: output.status.success(),
        })
    }

    // PRIMITIVE 3: AI-powered observation of execution results
    pub async fn observe_execution(&self, raw_result: &RawExecutionResult) -> Result<Observation, QueryResolverError> {
        let input = format!("STDOUT:\n{}\n\nSTDERR:\n{}\n\nEXIT_CODE: {}", 
                           raw_result.stdout, raw_result.stderr, raw_result.exit_code);
        
        self.observe(&input, "execution_analysis").await
    }

    // Core Cognition Cycle: Observe -> Generate -> Execute
    pub async fn cognize(&self, goal: &str) -> Result<CognitiveState, Box<dyn std::error::Error>> {
        let mut state = CognitiveState {
            observations: Vec::new(),
            generations: Vec::new(),
            current_phase: "requirements_gathering".to_string(),
            goal: goal.to_string(),
            is_complete: false,
        };

        // Phase 1: Requirements Gathering
        println!("Phase 1: Gathering requirements...");
        let user_query = self.observe(goal, "user_query").await?;
        state.observations.push(user_query);

        let requirements_questions = self.generate(&state.observations, "requirements_questions").await?;
        state.generations.push(requirements_questions);

        // Phase 2: Architecture Analysis
        println!("Phase 2: Analyzing architecture...");
        state.current_phase = "architecture_analysis".to_string();
        let architecture = self.generate(&state.observations, "architecture_analysis").await?;
        state.generations.push(architecture);

        // Phase 3: Implementation Generation
        println!("Phase 3: Generating implementation...");
        state.current_phase = "implementation".to_string();
        let implementation = self.generate(&state.observations, "python_implementation").await?;
        state.generations.push(implementation.clone());

        // Phase 4: Execution and Verification
        println!("Phase 4: Executing and verifying...");
        state.current_phase = "verification".to_string();
        let raw_execution = self.execute_code(&implementation.content).await?;
        let execution_observation = self.observe_execution(&raw_execution).await?;
        state.observations.push(execution_observation.clone());

        // Phase 5: Test-Spec Reflection
        println!("Phase 5: Analyzing test-spec alignment...");
        let test_reflection = self.observe(&implementation.content, "test_spec_reflection").await?;
        state.observations.push(test_reflection);

        // Check if we're done
        if let Observation::ExecutionResult { success, .. } = execution_observation {
            state.is_complete = success;
            state.current_phase = if success { "complete" } else { "failed" }.to_string();
        }

        Ok(state)
    }

    // Helper method to build context from observations
    fn build_context(&self, observations: &[Observation]) -> String {
        let mut context = String::new();
        
        for (i, obs) in observations.iter().enumerate() {
            context.push_str(&format!("Observation {}: ", i + 1));
            match obs {
                Observation::UserQuery { question, expected_type, context: ctx } => {
                    context.push_str(&format!("Question: {} (Type: {}, Context: {})\n", question, expected_type, ctx));
                },
                Observation::TestSpecReflection { spec_extracted, alignment_score, .. } => {
                    context.push_str(&format!("Spec: {} (Alignment: {})\n", spec_extracted, alignment_score));
                },
                Observation::ExecutionResult { success, stdout, stderr, .. } => {
                    context.push_str(&format!("Execution: {} (stdout: {}, stderr: {})\n", 
                        if *success { "SUCCESS" } else { "FAILED" }, 
                        stdout.chars().take(100).collect::<String>(),
                        stderr.chars().take(100).collect::<String>()
                    ));
                },
            }
        }
        
        context
    }
}

// Example usage and test
#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockVoid; // Assuming you have a mock implementation

    #[tokio::test]
    async fn test_tictactoe_generation_cycle() {
        let mock_client = MockVoid; // Your mock implementation
        let cogmach = CognitionMachine::new(mock_client);
        
        let result = cogmach.cognize("create a tictactoe api").await;
        
        match result {
            Ok(state) => {
                assert!(!state.observations.is_empty());
                assert!(!state.generations.is_empty());
                println!("Final state: {:?}", state);
            },
            Err(e) => {
                println!("Error in cognition cycle: {}", e);
            }
        }
    }
}

// CLI for manual testing
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Cognition Machine Experiment");
    println!("Goal: Transform 'create a tictactoe api' into working Python code");
    println!();

    // You would instantiate with your actual Claude client here
    // let claude_client = ClaudeClient::new(api_key);
    // let cogmach = CognitionMachine::new(claude_client);
    
    // For now, demonstrate the structure
    println!("Phase breakdown:");
    println!("1. Observe user input -> Generate clarifying questions");
    println!("2. Generate architecture analysis");  
    println!("3. Generate Python implementation with tests");
    println!("4. Execute implementation");
    println!("5. Observe test-spec reflection");
    println!("6. Verify completion");
    
    println!();
    println!("Each phase uses only the three core primitives:");
    println!("- OBSERVE: Extract information from any source");
    println!("- GENERATE: Create content based on observations"); 
    println!("- EXECUTE: Run generated content and observe results");
    
    Ok(())
}