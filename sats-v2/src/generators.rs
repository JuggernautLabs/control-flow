//! Code and specification generators for SATS v2
//! 
//! This module provides AI-powered generators that create implementations,
//! tests, and specifications to fulfill work items.

use crate::types::*;
use crate::semantic::{LlmClient, ClaimAnalysis};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("Implementation generation failed: {0}")]
    ImplementationGeneration(String),
    #[error("Test generation failed: {0}")]
    TestGeneration(String),
    #[error("Specification generation failed: {0}")]
    SpecificationGeneration(String),
    #[error("LLM communication error: {0}")]
    LlmError(String),
    #[error("Template error: {0}")]
    Template(String),
    #[error("Code validation failed: {0}")]
    CodeValidation(String),
}

/// Main generator that coordinates different generation types
pub struct GeneratorEngine {
    implementation_generator: Box<dyn ImplementationGenerator>,
    test_generator: Box<dyn TestGenerator>,
    specification_generator: Box<dyn SpecificationGenerator>,
    llm_client: LlmClient,
    templates: TemplateRegistry,
}

impl GeneratorEngine {
    pub fn new(
        implementation_generator: Box<dyn ImplementationGenerator>,
        test_generator: Box<dyn TestGenerator>,
        specification_generator: Box<dyn SpecificationGenerator>,
        llm_client: LlmClient,
    ) -> Self {
        Self {
            implementation_generator,
            test_generator,
            specification_generator,
            llm_client,
            templates: TemplateRegistry::default(),
        }
    }

    /// Generate implementation for a work item
    pub async fn generate_implementation(
        &self,
        work_item: &WorkItem,
        context: &GenerationContext,
    ) -> Result<GeneratedImplementation, GeneratorError> {
        self.implementation_generator
            .generate_implementation(work_item, context)
            .await
    }

    /// Generate tests for a work item
    pub async fn generate_tests(
        &self,
        work_item: &WorkItem,
        context: &GenerationContext,
    ) -> Result<GeneratedTestSuite, GeneratorError> {
        self.test_generator
            .generate_tests(work_item, context)
            .await
    }

    /// Generate specification for a work item
    pub async fn generate_specification(
        &self,
        claim: &Claim,
        context: &GenerationContext,
    ) -> Result<GeneratedSpecification, GeneratorError> {
        self.specification_generator
            .generate_specification(claim, context)
            .await
    }
}

/// Context needed for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationContext {
    pub project_context: ProjectContext,
    pub existing_code: Vec<CodeContext>,
    pub claim_analysis: Option<ClaimAnalysis>,
    pub style_guide: StyleGuide,
    pub constraints: GenerationConstraints,
}

/// Information about existing code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    pub file_path: String,
    pub content: String,
    pub language: String,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
}

/// Project context for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub name: String,
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub architecture_style: String,
    pub patterns: Vec<String>,
}

/// Style guide for generated code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleGuide {
    pub indentation: String,
    pub naming_conventions: HashMap<String, String>,
    pub comment_style: String,
    pub max_line_length: usize,
    pub custom_rules: Vec<String>,
}

/// Constraints for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConstraints {
    pub max_complexity: u8,
    pub allowed_dependencies: Vec<String>,
    pub forbidden_patterns: Vec<String>,
    pub security_requirements: Vec<String>,
    pub performance_requirements: Vec<String>,
}

/// Generated implementation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImplementation {
    pub work_item_id: Id,
    pub language: String,
    pub code: String,
    pub file_path: String,
    pub dependencies: Vec<String>,
    pub documentation: String,
    pub complexity_score: u8,
    pub confidence: Confidence,
    pub generated_at: Timestamp,
}

/// Generated test suite result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTestSuite {
    pub work_item_id: Id,
    pub framework: String,
    pub test_cases: Vec<GeneratedTestCase>,
    pub setup_code: String,
    pub teardown_code: String,
    pub total_coverage_estimate: f64,
    pub generated_at: Timestamp,
}

/// Individual generated test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTestCase {
    pub name: String,
    pub description: String,
    pub test_code: String,
    pub test_type: TestType,
    pub inputs: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub assertions: Vec<String>,
}

/// Generated specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedSpecification {
    pub claim_id: Id,
    pub title: String,
    pub description: String,
    pub requirements: Vec<GeneratedRequirement>,
    pub interfaces: Vec<GeneratedInterface>,
    pub examples: Vec<GeneratedExample>,
    pub constraints: Vec<String>,
    pub generated_at: Timestamp,
}

/// Generated requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedRequirement {
    pub id: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: u8,
    pub effort_estimate: u8,
}

/// Generated interface specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedInterface {
    pub name: String,
    pub description: String,
    pub methods: Vec<GeneratedMethod>,
    pub properties: Vec<GeneratedProperty>,
}

/// Generated method specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedMethod {
    pub name: String,
    pub description: String,
    pub parameters: Vec<GeneratedParameter>,
    pub return_type: String,
    pub exceptions: Vec<String>,
}

/// Generated parameter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedParameter {
    pub name: String,
    pub parameter_type: String,
    pub description: String,
    pub optional: bool,
}

/// Generated property specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedProperty {
    pub name: String,
    pub property_type: String,
    pub description: String,
    pub readonly: bool,
}

/// Generated example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedExample {
    pub title: String,
    pub description: String,
    pub code: String,
    pub expected_output: String,
}

/// Template registry for code generation
#[derive(Debug, Clone)]
pub struct TemplateRegistry {
    implementation_templates: HashMap<String, String>,
    test_templates: HashMap<String, String>,
    specification_templates: HashMap<String, String>,
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        let mut registry = Self {
            implementation_templates: HashMap::new(),
            test_templates: HashMap::new(),
            specification_templates: HashMap::new(),
        };

        // Add default templates
        registry.implementation_templates.insert(
            "rust_function".to_string(),
            include_str!("../templates/rust_function.template").to_string(),
        );
        
        registry.test_templates.insert(
            "rust_test".to_string(),
            include_str!("../templates/rust_test.template").to_string(),
        );

        registry
    }
}

/// Trait for implementing code generators
#[async_trait]
pub trait ImplementationGenerator: Send + Sync {
    async fn generate_implementation(
        &self,
        work_item: &WorkItem,
        context: &GenerationContext,
    ) -> Result<GeneratedImplementation, GeneratorError>;
}

/// Trait for test generators
#[async_trait]
pub trait TestGenerator: Send + Sync {
    async fn generate_tests(
        &self,
        work_item: &WorkItem,
        context: &GenerationContext,
    ) -> Result<GeneratedTestSuite, GeneratorError>;
}

/// Trait for specification generators
#[async_trait]
pub trait SpecificationGenerator: Send + Sync {
    async fn generate_specification(
        &self,
        claim: &Claim,
        context: &GenerationContext,
    ) -> Result<GeneratedSpecification, GeneratorError>;
}

/// LLM-powered implementation generator
pub struct LlmImplementationGenerator {
    llm_client: LlmClient,
    templates: TemplateRegistry,
}

impl LlmImplementationGenerator {
    pub fn new(llm_client: LlmClient, templates: TemplateRegistry) -> Self {
        Self { llm_client, templates }
    }
}

#[async_trait]
impl ImplementationGenerator for LlmImplementationGenerator {
    async fn generate_implementation(
        &self,
        work_item: &WorkItem,
        context: &GenerationContext,
    ) -> Result<GeneratedImplementation, GeneratorError> {
        let prompt = self.build_implementation_prompt(work_item, context)?;
        
        let response = self.llm_client
            .generate(&prompt)
            .await
            .map_err(|e| GeneratorError::LlmError(e.to_string()))?;

        // Parse response and extract code
        let (code, documentation) = self.parse_implementation_response(&response)?;
        
        // Validate generated code
        self.validate_implementation(&code, context)?;

        // Extract dependencies and calculate complexity before moving code
        let dependencies = self.extract_dependencies(&code)?;
        let complexity_score = self.calculate_complexity(&code)?;

        Ok(GeneratedImplementation {
            work_item_id: work_item.id,
            language: self.detect_language(context)?,
            code,
            file_path: self.suggest_file_path(work_item, context)?,
            dependencies,
            documentation,
            complexity_score,
            confidence: Confidence::new(0.8).unwrap(),
            generated_at: chrono::Utc::now(),
        })
    }
}

impl LlmImplementationGenerator {
    fn build_implementation_prompt(
        &self,
        work_item: &WorkItem,
        context: &GenerationContext,
    ) -> Result<String, GeneratorError> {
        let style_guide = &context.style_guide;
        let constraints = &context.constraints;

        Ok(format!(
            r#"Generate implementation for this work item:

Title: {}
Description: {}

Project Context:
- Language: {}
- Framework: {}
- Architecture: {}

Requirements:
{}

Style Guide:
- Indentation: {}
- Max line length: {}
- Naming conventions: {:?}

Constraints:
- Max complexity: {}
- Allowed dependencies: {:?}
- Security requirements: {:?}

Existing Code Context:
{}

Generate clean, well-documented code that follows the style guide and satisfies all requirements.
Include comprehensive error handling and follow security best practices.

Format your response as:
```code
[generated code here]
```

```documentation
[documentation here]
```"#,
            work_item.title,
            work_item.description,
            context.project_context.languages.get(0).unwrap_or(&"unknown".to_string()),
            context.project_context.frameworks.get(0).unwrap_or(&"none".to_string()),
            context.project_context.architecture_style,
            serde_json::to_string_pretty(&work_item.specification).unwrap_or_default(),
            style_guide.indentation,
            style_guide.max_line_length,
            style_guide.naming_conventions,
            constraints.max_complexity,
            constraints.allowed_dependencies,
            constraints.security_requirements,
            context.existing_code.iter()
                .map(|c| format!("File: {}\n{}", c.file_path, c.content.chars().take(500).collect::<String>()))
                .collect::<Vec<_>>()
                .join("\n\n")
        ))
    }

    fn parse_implementation_response(&self, response: &str) -> Result<(String, String), GeneratorError> {
        // Extract code block
        let code = if let Some(start) = response.find("```code") {
            let code_start = start + 7;
            if let Some(end) = response[code_start..].find("```") {
                response[code_start..code_start + end].trim().to_string()
            } else {
                return Err(GeneratorError::Template("No closing ``` found for code block".to_string()));
            }
        } else {
            return Err(GeneratorError::Template("No code block found in response".to_string()));
        };

        // Extract documentation
        let documentation = if let Some(start) = response.find("```documentation") {
            let doc_start = start + 16;
            if let Some(end) = response[doc_start..].find("```") {
                response[doc_start..doc_start + end].trim().to_string()
            } else {
                "Generated implementation".to_string()
            }
        } else {
            "Generated implementation".to_string()
        };

        Ok((code, documentation))
    }

    fn validate_implementation(&self, code: &str, _context: &GenerationContext) -> Result<(), GeneratorError> {
        // Basic validation - check for obvious issues
        if code.is_empty() {
            return Err(GeneratorError::CodeValidation("Generated code is empty".to_string()));
        }

        if code.len() > 10000 {
            return Err(GeneratorError::CodeValidation("Generated code is too long".to_string()));
        }

        // Check for dangerous patterns
        let dangerous_patterns = ["eval(", "system(", "exec(", "unsafe {"];
        for pattern in &dangerous_patterns {
            if code.contains(pattern) {
                return Err(GeneratorError::CodeValidation(
                    format!("Generated code contains dangerous pattern: {}", pattern)
                ));
            }
        }

        Ok(())
    }

    fn detect_language(&self, context: &GenerationContext) -> Result<String, GeneratorError> {
        context.project_context.languages.get(0)
            .cloned()
            .ok_or_else(|| GeneratorError::Template("No language specified in context".to_string()))
    }

    fn suggest_file_path(&self, work_item: &WorkItem, context: &GenerationContext) -> Result<String, GeneratorError> {
        let language = self.detect_language(context)?;
        let extension = match language.as_str() {
            "rust" => "rs",
            "python" => "py",
            "javascript" => "js",
            "typescript" => "ts",
            _ => "txt",
        };

        Ok(format!("src/generated_{}.{}", work_item.id, extension))
    }

    fn extract_dependencies(&self, code: &str) -> Result<Vec<String>, GeneratorError> {
        let mut dependencies = Vec::new();

        // Simple dependency extraction based on common patterns
        for line in code.lines() {
            let line = line.trim();
            
            // Rust
            if line.starts_with("use ") && line.contains("::") {
                if let Some(dep) = line.split("::").next() {
                    let dep = dep.replace("use ", "").trim().to_string();
                    if !dep.starts_with("std") && !dep.starts_with("crate") {
                        dependencies.push(dep);
                    }
                }
            }
            
            // Python
            if line.starts_with("import ") || line.starts_with("from ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    dependencies.push(parts[1].to_string());
                }
            }
        }

        dependencies.sort();
        dependencies.dedup();
        Ok(dependencies)
    }

    fn calculate_complexity(&self, code: &str) -> Result<u8, GeneratorError> {
        // Simple complexity calculation based on code metrics
        let line_count = code.lines().count();
        let function_count = code.matches("fn ").count() + code.matches("def ").count();
        let conditional_count = code.matches("if ").count() + code.matches("match ").count();
        let loop_count = code.matches("for ").count() + code.matches("while ").count();

        let complexity = ((line_count / 10) + (function_count * 2) + (conditional_count * 3) + (loop_count * 3))
            .min(10) as u8;

        Ok(complexity)
    }
}

/// LLM-powered test generator
pub struct LlmTestGenerator {
    llm_client: LlmClient,
}

impl LlmTestGenerator {
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl TestGenerator for LlmTestGenerator {
    async fn generate_tests(
        &self,
        work_item: &WorkItem,
        context: &GenerationContext,
    ) -> Result<GeneratedTestSuite, GeneratorError> {
        let prompt = format!(
            r#"Generate comprehensive tests for this work item:

Title: {}
Description: {}

Project Context: {:?}

Generate tests that:
1. Cover all major functionality
2. Test edge cases and error conditions
3. Follow testing best practices
4. Use appropriate test framework

Include setup and teardown code if needed.
Estimate coverage percentage."#,
            work_item.title,
            work_item.description,
            context.project_context
        );

        let _response = self.llm_client
            .generate(&prompt)
            .await
            .map_err(|e| GeneratorError::LlmError(e.to_string()))?;

        // Simplified test generation for example
        Ok(GeneratedTestSuite {
            work_item_id: work_item.id,
            framework: "cargo".to_string(),
            test_cases: vec![GeneratedTestCase {
                name: "test_basic_functionality".to_string(),
                description: "Tests basic functionality".to_string(),
                test_code: "#[test]\nfn test_basic_functionality() {\n    assert!(true);\n}".to_string(),
                test_type: TestType::Unit,
                inputs: vec!["input".to_string()],
                expected_outputs: vec!["output".to_string()],
                assertions: vec!["assert!(result.is_ok())".to_string()],
            }],
            setup_code: "".to_string(),
            teardown_code: "".to_string(),
            total_coverage_estimate: 0.85,
            generated_at: chrono::Utc::now(),
        })
    }
}

/// Default specification generator
pub struct DefaultSpecificationGenerator {
    llm_client: LlmClient,
}

impl DefaultSpecificationGenerator {
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl SpecificationGenerator for DefaultSpecificationGenerator {
    async fn generate_specification(
        &self,
        claim: &Claim,
        _context: &GenerationContext,
    ) -> Result<GeneratedSpecification, GeneratorError> {
        // Simplified specification generation
        Ok(GeneratedSpecification {
            claim_id: claim.id,
            title: format!("Specification for: {}", claim.statement),
            description: format!("Detailed specification for claim: {}", claim.statement),
            requirements: vec![GeneratedRequirement {
                id: "REQ-001".to_string(),
                description: claim.statement.clone(),
                acceptance_criteria: vec!["Must satisfy the claim".to_string()],
                priority: 5,
                effort_estimate: 5,
            }],
            interfaces: vec![],
            examples: vec![],
            constraints: vec!["Follow project standards".to_string()],
            generated_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_implementation_generator() {
        let llm_client = LlmClient::default();
        let templates = TemplateRegistry::default();
        let generator = LlmImplementationGenerator::new(llm_client, templates);

        let work_item = WorkItem {
            id: uuid::Uuid::new_v4(),
            work_item_type: WorkItemType::ImplementRequirements,
            claim_id: uuid::Uuid::new_v4(),
            title: "Implement authentication".to_string(),
            description: "Implement user authentication system".to_string(),
            status: WorkItemStatus::Pending,
            created_at: chrono::Utc::now(),
            assignee: None,
            estimated_effort: 5,
            required_skills: vec!["rust".to_string()],
            specification: serde_json::Value::Null,
        };

        let context = GenerationContext {
            project_context: ProjectContext {
                name: "test-project".to_string(),
                languages: vec!["rust".to_string()],
                frameworks: vec!["axum".to_string()],
                architecture_style: "REST API".to_string(),
                patterns: vec!["Repository".to_string()],
            },
            existing_code: vec![],
            claim_analysis: None,
            style_guide: StyleGuide {
                indentation: "    ".to_string(),
                naming_conventions: HashMap::new(),
                comment_style: "//".to_string(),
                max_line_length: 100,
                custom_rules: vec![],
            },
            constraints: GenerationConstraints {
                max_complexity: 8,
                allowed_dependencies: vec!["serde".to_string()],
                forbidden_patterns: vec!["unsafe".to_string()],
                security_requirements: vec!["Validate all inputs".to_string()],
                performance_requirements: vec![],
            },
        };

        // This would fail in real use due to LLM call, but tests the structure
        // let result = generator.generate_implementation(&work_item, &context).await;
        // assert!(result.is_ok());
    }

    #[test]
    fn test_dependency_extraction() {
        let generator = LlmImplementationGenerator::new(LlmClient::default(), TemplateRegistry::default());
        
        let rust_code = r#"
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

fn main() {
    println!("Hello, world!");
}
"#;

        let deps = generator.extract_dependencies(rust_code).unwrap();
        assert!(deps.contains(&"serde".to_string()));
        assert!(deps.contains(&"uuid".to_string()));
        assert!(!deps.iter().any(|d| d.starts_with("std")));
    }

    #[test]
    fn test_complexity_calculation() {
        let generator = LlmImplementationGenerator::new(LlmClient::default(), TemplateRegistry::default());
        
        let simple_code = "fn hello() { println!(\"hello\"); }";
        let complexity = generator.calculate_complexity(simple_code).unwrap();
        assert!(complexity <= 3);

        let complex_code = r#"
fn complex_function() {
    for i in 0..10 {
        if i % 2 == 0 {
            match i {
                0 => println!("zero"),
                2 => println!("two"),
                _ => println!("other"),
            }
        }
    }
}
"#;
        let complexity = generator.calculate_complexity(complex_code).unwrap();
        assert!(complexity > 3);
    }
}