//! Execution engine for SATS v2
//! 
//! This module handles actual execution of tests and code verification
//! in sandboxed environments to ensure claims are truly verified.

use crate::types::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use thiserror::Error;
use tokio::fs;
use tokio::time::timeout;

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Sandbox creation failed: {0}")]
    SandboxCreation(String),
    #[error("Test execution failed: {0}")]
    TestExecution(String),
    #[error("Execution timeout after {0:?}")]
    Timeout(Duration),
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Main execution engine that runs tests in isolated environments
pub struct ExecutionEngine {
    config: ExecutionConfig,
}

#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub sandbox_config: crate::verification::SandboxConfig,
    pub supported_languages: Vec<Language>,
    pub default_timeout: Duration,
    pub max_concurrent_executions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
}

impl ExecutionEngine {
    pub fn new(config: ExecutionConfig) -> Self {
        Self { config }
    }

    /// Execute a test suite and return detailed results
    pub async fn execute_test_suite(&self, test_suite: &TestSuite) -> Result<ExecutionResult, ExecutionError> {
        let start_time = Instant::now();
        
        // Create isolated execution environment
        let environment = self.create_environment(test_suite).await?;
        
        // Execute all test cases
        let mut results = Vec::new();
        let mut total_passed = 0;
        let mut total_failed = 0;
        let mut total_errors = 0;

        for test_case in &test_suite.test_cases {
            let result = self.execute_test_case(&environment, test_case).await?;
            
            match result.passed {
                true => total_passed += 1,
                false => {
                    if result.error_message.is_some() {
                        total_errors += 1;
                    } else {
                        total_failed += 1;
                    }
                }
            }
            
            results.push(result);
        }

        let execution_time = start_time.elapsed();
        let status = if total_errors > 0 {
            ExecutionStatus::Error
        } else if total_failed > 0 {
            ExecutionStatus::Failed
        } else {
            ExecutionStatus::Passed
        };

        // Calculate coverage if possible
        let coverage = self.calculate_coverage(&environment, test_suite).await?;

        Ok(ExecutionResult {
            test_suite_id: test_suite.id,
            status,
            results,
            total_passed,
            total_failed,
            total_errors,
            coverage,
            executed_at: chrono::Utc::now(),
            execution_time,
        })
    }

    /// Create an isolated execution environment
    async fn create_environment(&self, test_suite: &TestSuite) -> Result<ExecutionEnvironment, ExecutionError> {
        let temp_dir = TempDir::new()
            .map_err(|e| ExecutionError::SandboxCreation(e.to_string()))?;

        let language = self.detect_language(&test_suite.framework)?;
        
        // Set up the environment based on language and framework
        let setup_result = match language {
            Language::Rust => self.setup_rust_environment(&temp_dir, test_suite).await,
            Language::Python => self.setup_python_environment(&temp_dir, test_suite).await,
            Language::JavaScript => self.setup_javascript_environment(&temp_dir, test_suite).await,
            _ => Err(ExecutionError::SandboxCreation(format!("Unsupported language: {:?}", language))),
        }?;

        Ok(ExecutionEnvironment {
            temp_dir,
            language,
            framework: test_suite.framework.clone(),
            setup_result,
        })
    }

    /// Execute a single test case in the environment
    async fn execute_test_case(
        &self,
        environment: &ExecutionEnvironment,
        test_case: &TestCase,
    ) -> Result<TestResult, ExecutionError> {
        let start_time = Instant::now();

        // Write test case to file
        let test_file_path = environment.temp_dir.path().join(format!("test_{}.rs", test_case.id));
        fs::write(&test_file_path, &test_case.test_code).await?;

        // Execute based on language
        let execution_result = match environment.language {
            Language::Rust => self.execute_rust_test(environment, test_case, &test_file_path).await,
            Language::Python => self.execute_python_test(environment, test_case, &test_file_path).await,
            Language::JavaScript => self.execute_javascript_test(environment, test_case, &test_file_path).await,
            _ => return Err(ExecutionError::TestExecution("Unsupported language".to_string())),
        };

        let execution_time = start_time.elapsed();

        match execution_result {
            Ok(output) => Ok(TestResult {
                test_case_id: test_case.id,
                passed: true,
                output,
                error_message: None,
                execution_time,
                coverage: None,
            }),
            Err(error) => Ok(TestResult {
                test_case_id: test_case.id,
                passed: false,
                output: String::new(),
                error_message: Some(error.to_string()),
                execution_time,
                coverage: None,
            }),
        }
    }

    /// Execute Rust test
    async fn execute_rust_test(
        &self,
        environment: &ExecutionEnvironment,
        _test_case: &TestCase,
        test_file_path: &std::path::Path,
    ) -> Result<String, ExecutionError> {
        let cargo_test = timeout(
            self.config.default_timeout,
            tokio::task::spawn_blocking({
                let test_file_path = test_file_path.to_owned();
                let temp_dir = environment.temp_dir.path().to_owned();
                move || {
                    Command::new("cargo")
                        .args(&["test", "--manifest-path"])
                        .arg(temp_dir.join("Cargo.toml"))
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                }
            })
        ).await
        .map_err(|_| ExecutionError::Timeout(self.config.default_timeout))?
        .map_err(|e| ExecutionError::TestExecution(e.to_string()))?
        .map_err(|e| ExecutionError::TestExecution(e.to_string()))?;

        if cargo_test.status.success() {
            Ok(String::from_utf8_lossy(&cargo_test.stdout).to_string())
        } else {
            Err(ExecutionError::TestExecution(
                String::from_utf8_lossy(&cargo_test.stderr).to_string()
            ))
        }
    }

    /// Execute Python test  
    async fn execute_python_test(
        &self,
        _environment: &ExecutionEnvironment,
        _test_case: &TestCase,
        test_file_path: &std::path::Path,
    ) -> Result<String, ExecutionError> {
        let python_test = timeout(
            self.config.default_timeout,
            tokio::task::spawn_blocking({
                let test_file_path = test_file_path.to_owned();
                move || {
                    Command::new("python")
                        .args(&["-m", "pytest"])
                        .arg(&test_file_path)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                }
            })
        ).await
        .map_err(|_| ExecutionError::Timeout(self.config.default_timeout))?
        .map_err(|e| ExecutionError::TestExecution(e.to_string()))?
        .map_err(|e| ExecutionError::TestExecution(e.to_string()))?;

        if python_test.status.success() {
            Ok(String::from_utf8_lossy(&python_test.stdout).to_string())
        } else {
            Err(ExecutionError::TestExecution(
                String::from_utf8_lossy(&python_test.stderr).to_string()
            ))
        }
    }

    /// Execute JavaScript test
    async fn execute_javascript_test(
        &self,
        _environment: &ExecutionEnvironment,
        _test_case: &TestCase,
        test_file_path: &std::path::Path,
    ) -> Result<String, ExecutionError> {
        let node_test = timeout(
            self.config.default_timeout,
            tokio::task::spawn_blocking({
                let test_file_path = test_file_path.to_owned();
                move || {
                    Command::new("npm")
                        .args(&["test"])
                        .arg(&test_file_path)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                }
            })
        ).await
        .map_err(|_| ExecutionError::Timeout(self.config.default_timeout))?
        .map_err(|e| ExecutionError::TestExecution(e.to_string()))?
        .map_err(|e| ExecutionError::TestExecution(e.to_string()))?;

        if node_test.status.success() {
            Ok(String::from_utf8_lossy(&node_test.stdout).to_string())
        } else {
            Err(ExecutionError::TestExecution(
                String::from_utf8_lossy(&node_test.stderr).to_string()
            ))
        }
    }

    /// Set up Rust environment
    async fn setup_rust_environment(
        &self,
        temp_dir: &TempDir,
        _test_suite: &TestSuite,
    ) -> Result<EnvironmentSetupResult, ExecutionError> {
        let cargo_toml = r#"[package]
name = "sats-test"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).await?;
        fs::create_dir_all(temp_dir.path().join("src")).await?;
        fs::write(temp_dir.path().join("src").join("lib.rs"), "// Test library").await?;

        Ok(EnvironmentSetupResult {
            success: true,
            created_files: vec!["Cargo.toml".to_string(), "src/lib.rs".to_string()],
            environment_variables: HashMap::new(),
        })
    }

    /// Set up Python environment
    async fn setup_python_environment(
        &self,
        temp_dir: &TempDir,
        _test_suite: &TestSuite,
    ) -> Result<EnvironmentSetupResult, ExecutionError> {
        let requirements_txt = "pytest\n";
        fs::write(temp_dir.path().join("requirements.txt"), requirements_txt).await?;

        // Install dependencies
        let pip_install = Command::new("pip")
            .args(&["install", "-r", "requirements.txt"])
            .current_dir(temp_dir.path())
            .output()
            .map_err(|e| ExecutionError::SandboxCreation(e.to_string()))?;

        if !pip_install.status.success() {
            return Err(ExecutionError::SandboxCreation(
                String::from_utf8_lossy(&pip_install.stderr).to_string()
            ));
        }

        Ok(EnvironmentSetupResult {
            success: true,
            created_files: vec!["requirements.txt".to_string()],
            environment_variables: HashMap::new(),
        })
    }

    /// Set up JavaScript environment
    async fn setup_javascript_environment(
        &self,
        temp_dir: &TempDir,
        _test_suite: &TestSuite,
    ) -> Result<EnvironmentSetupResult, ExecutionError> {
        let package_json = r#"{
  "name": "sats-test",
  "version": "1.0.0",
  "scripts": {
    "test": "jest"
  },
  "devDependencies": {
    "jest": "^29.0.0"
  }
}"#;

        fs::write(temp_dir.path().join("package.json"), package_json).await?;

        // Install dependencies
        let npm_install = Command::new("npm")
            .args(&["install"])
            .current_dir(temp_dir.path())
            .output()
            .map_err(|e| ExecutionError::SandboxCreation(e.to_string()))?;

        if !npm_install.status.success() {
            return Err(ExecutionError::SandboxCreation(
                String::from_utf8_lossy(&npm_install.stderr).to_string()
            ));
        }

        Ok(EnvironmentSetupResult {
            success: true,
            created_files: vec!["package.json".to_string()],
            environment_variables: HashMap::new(),
        })
    }

    /// Detect language from test framework
    fn detect_language(&self, framework: &str) -> Result<Language, ExecutionError> {
        match framework.to_lowercase().as_str() {
            "cargo" | "rust" => Ok(Language::Rust),
            "pytest" | "unittest" | "python" => Ok(Language::Python),
            "jest" | "mocha" | "jasmine" | "javascript" => Ok(Language::JavaScript),
            "typescript" | "ts-jest" => Ok(Language::TypeScript),
            "go" | "testing" => Ok(Language::Go),
            "junit" | "java" => Ok(Language::Java),
            _ => Err(ExecutionError::SandboxCreation(
                format!("Unknown test framework: {}", framework)
            )),
        }
    }

    /// Calculate test coverage if possible
    async fn calculate_coverage(
        &self,
        _environment: &ExecutionEnvironment,
        _test_suite: &TestSuite,
    ) -> Result<Option<f64>, ExecutionError> {
        // Coverage calculation would be implemented based on language/framework
        // For now, return None
        Ok(None)
    }
}

/// Represents an isolated execution environment
pub struct ExecutionEnvironment {
    temp_dir: TempDir,
    language: Language,
    framework: String,
    setup_result: EnvironmentSetupResult,
}

/// Result of setting up an execution environment
#[derive(Debug, Clone)]
pub struct EnvironmentSetupResult {
    success: bool,
    created_files: Vec<String>,
    environment_variables: HashMap<String, String>,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            sandbox_config: crate::verification::SandboxConfig::default(),
            supported_languages: vec![
                Language::Rust,
                Language::Python,
                Language::JavaScript,
            ],
            default_timeout: Duration::from_secs(30),
            max_concurrent_executions: 4,
        }
    }
}

/// Trait for custom test executors
#[async_trait]
pub trait TestExecutor: Send + Sync {
    async fn execute(&self, test_suite: &TestSuite) -> Result<ExecutionResult, ExecutionError>;
    fn supported_languages(&self) -> Vec<Language>;
}

/// Default executor implementation
pub struct DefaultTestExecutor {
    engine: ExecutionEngine,
}

impl DefaultTestExecutor {
    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            engine: ExecutionEngine::new(config),
        }
    }
}

#[async_trait]
impl TestExecutor for DefaultTestExecutor {
    async fn execute(&self, test_suite: &TestSuite) -> Result<ExecutionResult, ExecutionError> {
        self.engine.execute_test_suite(test_suite).await
    }

    fn supported_languages(&self) -> Vec<Language> {
        self.engine.config.supported_languages.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert_eq!(config.default_timeout, Duration::from_secs(30));
        assert_eq!(config.max_concurrent_executions, 4);
        assert!(config.supported_languages.contains(&Language::Rust));
    }

    #[test]
    fn test_language_detection() {
        let engine = ExecutionEngine::new(ExecutionConfig::default());
        
        assert_eq!(engine.detect_language("cargo").unwrap(), Language::Rust);
        assert_eq!(engine.detect_language("pytest").unwrap(), Language::Python);
        assert_eq!(engine.detect_language("jest").unwrap(), Language::JavaScript);
        
        assert!(engine.detect_language("unknown").is_err());
    }

    #[tokio::test]
    async fn test_rust_environment_setup() {
        let engine = ExecutionEngine::new(ExecutionConfig::default());
        let temp_dir = TempDir::new().unwrap();
        
        let test_suite = TestSuite {
            id: uuid::Uuid::new_v4(),
            implementation_id: uuid::Uuid::new_v4(),
            test_cases: vec![],
            framework: "cargo".to_string(),
            total_tests: 0,
            detected_at: chrono::Utc::now(),
        };

        let result = engine.setup_rust_environment(&temp_dir, &test_suite).await.unwrap();
        assert!(result.success);
        assert!(result.created_files.contains(&"Cargo.toml".to_string()));
        
        // Verify files were created
        assert!(temp_dir.path().join("Cargo.toml").exists());
        assert!(temp_dir.path().join("src/lib.rs").exists());
    }

    #[test]
    fn test_execution_result_success_rate() {
        let result = ExecutionResult {
            test_suite_id: uuid::Uuid::new_v4(),
            status: ExecutionStatus::Passed,
            results: vec![
                TestResult {
                    test_case_id: uuid::Uuid::new_v4(),
                    passed: true,
                    output: "".to_string(),
                    error_message: None,
                    execution_time: Duration::from_millis(100),
                    coverage: None,
                },
                TestResult {
                    test_case_id: uuid::Uuid::new_v4(),
                    passed: false,
                    output: "".to_string(),
                    error_message: Some("Test failed".to_string()),
                    execution_time: Duration::from_millis(200),
                    coverage: None,
                },
            ],
            total_passed: 1,
            total_failed: 1,
            total_errors: 0,
            coverage: Some(0.75),
            executed_at: chrono::Utc::now(),
            execution_time: Duration::from_millis(300),
        };

        assert_eq!(result.success_rate(), 0.5); // 1 passed out of 2 total
    }
}