//! Code reference system for SATS v2
//! 
//! Instead of storing code directly, we store references to code within projects.
//! This allows us to track claims against actual codebases.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Universal code location that can reference code in various systems
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodeLocation {
    /// Git repository location
    Git {
        repository_url: String,
        commit_hash: String,
        file_path: String,
        function_name: Option<String>,
        line_range: Option<(u32, u32)>,
    },
    /// Local file system location
    Local {
        project_root: String,
        file_path: String,
        function_name: Option<String>,
        line_range: Option<(u32, u32)>,
    },
    /// GitHub/GitLab specific location
    Remote {
        platform: String, // github, gitlab, etc.
        owner: String,
        repository: String,
        commit_hash: String,
        file_path: String,
        function_name: Option<String>,
        line_range: Option<(u32, u32)>,
    },
    /// Package/module location (for published packages)
    Package {
        package_name: String,
        version: String,
        module_path: String,
        function_name: Option<String>,
    },
}

impl CodeLocation {
    /// Create a human-readable display of the location
    pub fn display(&self) -> String {
        match self {
            CodeLocation::Git { repository_url, commit_hash, file_path, function_name, line_range } => {
                let mut result = format!("{}@{}:{}", repository_url, &commit_hash[..8], file_path);
                if let Some(func) = function_name {
                    result.push_str(&format!("::{}", func));
                }
                if let Some((start, end)) = line_range {
                    result.push_str(&format!("({}:{})", start, end));
                }
                result
            }
            CodeLocation::Local { project_root, file_path, function_name, line_range } => {
                let mut result = format!("{}:{}", project_root, file_path);
                if let Some(func) = function_name {
                    result.push_str(&format!("::{}", func));
                }
                if let Some((start, end)) = line_range {
                    result.push_str(&format!("({}:{})", start, end));
                }
                result
            }
            CodeLocation::Remote { platform, owner, repository, commit_hash, file_path, function_name, line_range } => {
                let mut result = format!("{}/{}/{}@{}:{}", platform, owner, repository, &commit_hash[..8], file_path);
                if let Some(func) = function_name {
                    result.push_str(&format!("::{}", func));
                }
                if let Some((start, end)) = line_range {
                    result.push_str(&format!("({}:{})", start, end));
                }
                result
            }
            CodeLocation::Package { package_name, version, module_path, function_name } => {
                let mut result = format!("{}@{}:{}", package_name, version, module_path);
                if let Some(func) = function_name {
                    result.push_str(&format!("::{}", func));
                }
                result
            }
        }
    }

    /// Extract repository information if available
    pub fn repository_info(&self) -> Option<RepositoryInfo> {
        match self {
            CodeLocation::Git { repository_url, commit_hash, .. } => {
                Some(RepositoryInfo {
                    url: repository_url.clone(),
                    commit_hash: commit_hash.clone(),
                })
            }
            CodeLocation::Remote { platform, owner, repository, commit_hash, .. } => {
                Some(RepositoryInfo {
                    url: format!("{}/{}/{}", platform, owner, repository),
                    commit_hash: commit_hash.clone(),
                })
            }
            _ => None,
        }
    }

    /// Get the file path within the project
    pub fn file_path(&self) -> &str {
        match self {
            CodeLocation::Git { file_path, .. } => file_path,
            CodeLocation::Local { file_path, .. } => file_path,
            CodeLocation::Remote { file_path, .. } => file_path,
            CodeLocation::Package { module_path, .. } => module_path,
        }
    }

    /// Get the function name if specified
    pub fn function_name(&self) -> Option<&str> {
        match self {
            CodeLocation::Git { function_name, .. } => function_name.as_deref(),
            CodeLocation::Local { function_name, .. } => function_name.as_deref(),
            CodeLocation::Remote { function_name, .. } => function_name.as_deref(),
            CodeLocation::Package { function_name, .. } => function_name.as_deref(),
        }
    }
}

/// Repository information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub url: String,
    pub commit_hash: String,
}

/// Reference to code with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReference {
    pub id: uuid::Uuid,
    pub location: CodeLocation,
    pub language: ProgrammingLanguage,
    pub code_type: CodeType,
    pub metadata: CodeMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Programming language
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProgrammingLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Cpp,
    C,
    Other(String),
}

/// Type of code being referenced
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodeType {
    Implementation,
    Test,
    Documentation,
    Configuration,
    Script,
    Other(String),
}

/// Metadata about the code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetadata {
    /// Brief description of what this code does
    pub description: String,
    /// Dependencies required to run this code
    pub dependencies: Vec<String>,
    /// How to execute this code
    pub execution_info: ExecutionInfo,
    /// Test framework used (for test code)
    pub test_framework: Option<String>,
    /// Build system information
    pub build_system: Option<BuildSystem>,
    /// Additional tags for classification
    pub tags: Vec<String>,
    /// Estimated complexity (1-10)
    pub complexity: u8,
}

/// Information about how to execute code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    /// Command to run the code/tests
    pub command: String,
    /// Arguments to pass to the command
    pub args: Vec<String>,
    /// Working directory (relative to project root)
    pub working_directory: Option<String>,
    /// Environment variables needed
    pub environment: HashMap<String, String>,
    /// Setup commands to run first
    pub setup_commands: Vec<String>,
    /// Cleanup commands to run after
    pub cleanup_commands: Vec<String>,
}

/// Build system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSystem {
    pub name: String, // cargo, npm, gradle, etc.
    pub build_file: String, // Cargo.toml, package.json, etc.
    pub build_command: String,
    pub test_command: String,
}

/// Reference to a test with specific execution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReference {
    pub id: uuid::Uuid,
    pub code_reference: CodeReference,
    pub test_name: String,
    pub test_type: TestType,
    pub test_specification: TestSpecification,
    pub related_implementations: Vec<uuid::Uuid>, // CodeReference IDs
}

/// Type of test
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestType {
    Unit,
    Integration,
    EndToEnd,
    Performance,
    Security,
    Property, // Property-based testing
    Regression,
}

/// Specification for what a test should verify
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSpecification {
    pub description: String,
    pub inputs: Vec<TestInput>,
    pub expected_outputs: Vec<TestOutput>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub edge_cases: Vec<EdgeCase>,
    pub performance_criteria: Option<PerformanceCriteria>,
}

/// Test input specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInput {
    pub name: String,
    pub data_type: String,
    pub value: String,
    pub description: String,
}

/// Test output specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOutput {
    pub name: String,
    pub data_type: String,
    pub expected_value: String,
    pub validation_rule: String,
}

/// Edge case specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCase {
    pub scenario: String,
    pub inputs: Vec<TestInput>,
    pub expected_behavior: String,
    pub severity: RiskLevel,
}

/// Performance criteria for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCriteria {
    pub max_execution_time_ms: u64,
    pub max_memory_usage_mb: u64,
    pub throughput_requirements: Option<ThroughputSpec>,
}

/// Throughput specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputSpec {
    pub operations_per_second: u64,
    pub concurrent_users: u64,
}

/// Risk level for edge cases
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Reference to generated/modified code that fulfills claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationReference {
    pub id: uuid::Uuid,
    /// Where the implementation should be placed
    pub target_location: CodeLocation,
    /// References to tests that should pass
    pub test_references: Vec<uuid::Uuid>,
    /// What this implementation should do
    pub specification: ImplementationSpecification,
    /// How to verify the implementation works
    pub verification_info: VerificationInfo,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Specification for implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationSpecification {
    pub description: String,
    pub interface: InterfaceSpecification,
    pub behavior_requirements: Vec<BehaviorRequirement>,
    pub quality_requirements: QualityRequirements,
}

/// Interface that implementation must provide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSpecification {
    pub functions: Vec<FunctionSpec>,
    pub types: Vec<TypeSpec>,
    pub constants: Vec<ConstantSpec>,
}

/// Function specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSpec {
    pub name: String,
    pub parameters: Vec<ParameterSpec>,
    pub return_type: String,
    pub visibility: Visibility,
    pub documentation: String,
    pub behavior: String,
}

/// Parameter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub name: String,
    pub parameter_type: String,
    pub constraints: Vec<String>,
    pub default_value: Option<String>,
}

/// Type specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSpec {
    pub name: String,
    pub kind: TypeKind,
    pub fields: Vec<FieldSpec>,
    pub documentation: String,
}

/// Kind of type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeKind {
    Struct,
    Enum,
    Union,
    Trait,
    Interface,
    Class,
}

/// Field specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSpec {
    pub name: String,
    pub field_type: String,
    pub visibility: Visibility,
    pub constraints: Vec<String>,
    pub documentation: String,
}

/// Constant specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantSpec {
    pub name: String,
    pub constant_type: String,
    pub visibility: Visibility,
    pub documentation: String,
}

/// Visibility level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Package,
}

/// Behavior requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorRequirement {
    pub description: String,
    pub scenarios: Vec<BehaviorScenario>,
    pub constraints: Vec<String>,
}

/// Behavior scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorScenario {
    pub given: String,
    pub when: String,
    pub then: String,
    pub examples: Vec<ScenarioExample>,
}

/// Example for a behavior scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioExample {
    pub inputs: HashMap<String, String>,
    pub expected_outputs: HashMap<String, String>,
}

/// Quality requirements for implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub performance: Option<PerformanceRequirements>,
    pub security: Option<SecurityRequirements>,
    pub reliability: Option<ReliabilityRequirements>,
    pub maintainability: Option<MaintainabilityRequirements>,
}

/// Performance requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    pub max_response_time_ms: u64,
    pub min_throughput: u64,
    pub max_memory_usage_mb: u64,
    pub scalability_requirements: Vec<String>,
}

/// Security requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    pub authentication_required: bool,
    pub authorization_levels: Vec<String>,
    pub data_protection: Vec<String>,
    pub input_validation: Vec<String>,
}

/// Reliability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityRequirements {
    pub availability_percentage: f64,
    pub error_handling: Vec<String>,
    pub recovery_procedures: Vec<String>,
    pub monitoring_requirements: Vec<String>,
}

/// Maintainability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainabilityRequirements {
    pub documentation_coverage: f64,
    pub test_coverage: f64,
    pub code_style_compliance: bool,
    pub complexity_limits: ComplexityLimits,
}

/// Complexity limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityLimits {
    pub max_cyclomatic_complexity: u8,
    pub max_function_length: u32,
    pub max_file_length: u32,
    pub max_nesting_depth: u8,
}

/// Information about how to verify implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationInfo {
    pub compilation_check: CompilationCheck,
    pub test_execution: TestExecution,
    pub quality_checks: Vec<QualityCheck>,
}

/// Compilation verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationCheck {
    pub build_command: String,
    pub expected_artifacts: Vec<String>,
    pub allowed_warnings: Vec<String>,
}

/// Test execution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecution {
    pub test_command: String,
    pub required_test_names: Vec<String>,
    pub minimum_coverage: f64,
    pub performance_benchmarks: Vec<PerformanceBenchmark>,
}

/// Performance benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmark {
    pub name: String,
    pub command: String,
    pub max_time_ms: u64,
    pub max_memory_mb: u64,
}

/// Quality check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheck {
    pub name: String,
    pub command: String,
    pub passing_criteria: String,
}

#[derive(Error, Debug)]
pub enum CodeReferenceError {
    #[error("Invalid code location: {0}")]
    InvalidLocation(String),
    #[error("Code not found at location: {0}")]
    CodeNotFound(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

impl CodeReference {
    /// Create a new code reference
    pub fn new(location: CodeLocation, language: ProgrammingLanguage, code_type: CodeType) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            location,
            language,
            code_type,
            metadata: CodeMetadata {
                description: String::new(),
                dependencies: Vec::new(),
                execution_info: ExecutionInfo {
                    command: String::new(),
                    args: Vec::new(),
                    working_directory: None,
                    environment: HashMap::new(),
                    setup_commands: Vec::new(),
                    cleanup_commands: Vec::new(),
                },
                test_framework: None,
                build_system: None,
                tags: Vec::new(),
                complexity: 1,
            },
            created_at: chrono::Utc::now(),
        }
    }

    /// Check if this code reference exists and is accessible
    pub async fn validate(&self) -> Result<bool, CodeReferenceError> {
        // Implementation would check if the code actually exists at the location
        // For now, return true as placeholder
        Ok(true)
    }

    /// Get the code content (when needed for analysis)
    pub async fn get_content(&self) -> Result<String, CodeReferenceError> {
        // Implementation would fetch the actual code content
        // This might involve git clone, file read, API calls, etc.
        Ok("// Code content would be fetched here".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_location_display() {
        let git_location = CodeLocation::Git {
            repository_url: "https://github.com/user/repo".to_string(),
            commit_hash: "abcdef1234567890".to_string(),
            file_path: "src/main.rs".to_string(),
            function_name: Some("main".to_string()),
            line_range: Some((10, 20)),
        };

        assert_eq!(
            git_location.display(),
            "https://github.com/user/repo@abcdef12:src/main.rs::main(10:20)"
        );
    }

    #[test]
    fn test_code_reference_creation() {
        let location = CodeLocation::Local {
            project_root: "/home/user/project".to_string(),
            file_path: "src/lib.rs".to_string(),
            function_name: None,
            line_range: None,
        };

        let code_ref = CodeReference::new(
            location,
            ProgrammingLanguage::Rust,
            CodeType::Implementation,
        );

        assert_eq!(code_ref.language, ProgrammingLanguage::Rust);
        assert_eq!(code_ref.code_type, CodeType::Implementation);
    }

    #[test]
    fn test_repository_info_extraction() {
        let git_location = CodeLocation::Git {
            repository_url: "https://github.com/user/repo".to_string(),
            commit_hash: "abc123".to_string(),
            file_path: "src/main.rs".to_string(),
            function_name: None,
            line_range: None,
        };

        let repo_info = git_location.repository_info().unwrap();
        assert_eq!(repo_info.url, "https://github.com/user/repo");
        assert_eq!(repo_info.commit_hash, "abc123");
    }
}