//! Work item management for SATS v2
//! 
//! This module handles generation, assignment, and tracking of work items
//! that are created when verification chains have gaps.

use crate::types::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkItemError {
    #[error("Failed to generate work item: {0}")]
    Generation(String),
    #[error("Assignment failed: {0}")]
    Assignment(String),
    #[error("Invalid work item specification: {0}")]
    InvalidSpec(String),
    #[error("AI agent not available: {0}")]
    AgentUnavailable(String),
}

/// Manages work items and their assignment
pub struct WorkItemManager {
    generators: HashMap<WorkItemType, Box<dyn WorkItemGenerator>>,
    assignment_strategy: Box<dyn AssignmentStrategy>,
    active_assignments: HashMap<Id, WorkItemAssignment>,
}

impl WorkItemManager {
    pub fn new(assignment_strategy: Box<dyn AssignmentStrategy>) -> Self {
        Self {
            generators: HashMap::new(),
            assignment_strategy,
            active_assignments: HashMap::new(),
        }
    }

    pub fn register_generator(&mut self, work_type: WorkItemType, generator: Box<dyn WorkItemGenerator>) {
        self.generators.insert(work_type, generator);
    }

    /// Generate a work item for a verification gap
    pub async fn generate_work_item(
        &self,
        work_type: WorkItemType,
        claim: &Claim,
        context: &WorkGenerationContext,
    ) -> Result<WorkItem, WorkItemError> {
        let generator = self.generators.get(&work_type)
            .ok_or_else(|| WorkItemError::Generation(format!("No generator for {:?}", work_type)))?;

        generator.generate(claim, context).await
    }

    /// Assign a work item to the most appropriate agent/person
    pub async fn assign_work_item(&mut self, work_item: &WorkItem) -> Result<WorkItemAssignment, WorkItemError> {
        let assignment = self.assignment_strategy.assign(work_item).await?;
        self.active_assignments.insert(work_item.id, assignment.clone());
        Ok(assignment)
    }

    /// Get all active assignments
    pub fn get_active_assignments(&self) -> &HashMap<Id, WorkItemAssignment> {
        &self.active_assignments
    }

    /// Mark a work item as completed
    pub fn complete_work_item(&mut self, work_item_id: Id) -> Option<WorkItemAssignment> {
        self.active_assignments.remove(&work_item_id)
    }
}

/// Context needed for generating work items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkGenerationContext {
    pub verification_result: VerificationResult,
    pub project_context: ProjectContext,
    pub existing_artifacts: Vec<Artifact>,
}

/// Project context for work generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub test_frameworks: Vec<String>,
    pub coding_standards: HashMap<String, String>,
    pub architecture_patterns: Vec<String>,
}

/// Assignment of a work item to an agent or person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemAssignment {
    pub work_item_id: Id,
    pub assignee: Assignee,
    pub assigned_at: Timestamp,
    pub estimated_completion: Timestamp,
    pub status: AssignmentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Assignee {
    Human { name: String, email: String },
    AiAgent { agent_type: String, capabilities: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentStatus {
    Assigned,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Trait for generating specific types of work items
#[async_trait]
pub trait WorkItemGenerator: Send + Sync {
    async fn generate(&self, claim: &Claim, context: &WorkGenerationContext) -> Result<WorkItem, WorkItemError>;
}

/// Trait for assigning work items to appropriate agents
#[async_trait]
pub trait AssignmentStrategy: Send + Sync {
    async fn assign(&self, work_item: &WorkItem) -> Result<WorkItemAssignment, WorkItemError>;
}

/// Specific work item types with their specifications

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationWorkItem {
    pub requirements: Vec<Requirement>,
    pub suggested_location: Location,
    pub interface_spec: String,
    pub dependencies: Vec<String>,
    pub complexity_estimate: u8,
}

impl WorkItemSpec for ImplementationWorkItem {
    fn to_prompt(&self) -> String {
        format!(
            r#"Implement the following requirements:

Requirements:
{}

Suggested location: {}
Interface specification:
{}

Dependencies: {}

Generate implementation that fulfills these requirements."#,
            self.requirements.iter()
                .map(|r| format!("- {}: {}", r.description, r.acceptance_criteria.join(", ")))
                .collect::<Vec<_>>()
                .join("\n"),
            self.suggested_location.display(),
            self.interface_spec,
            self.dependencies.join(", ")
        )
    }

    fn estimated_effort(&self) -> u8 {
        self.complexity_estimate
    }

    fn required_skills(&self) -> Vec<String> {
        vec!["programming".to_string(), "architecture".to_string()]
    }

    fn is_suitable_for_ai(&self) -> bool {
        self.complexity_estimate <= 7 && self.dependencies.len() <= 3
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCreationWorkItem {
    pub implementation: Implementation,
    pub test_types: Vec<TestType>,
    pub coverage_requirements: f64,
    pub framework: String,
}

impl WorkItemSpec for TestCreationWorkItem {
    fn to_prompt(&self) -> String {
        format!(
            r#"Create tests for the following implementation:

Implementation location: {}
Test types needed: {:?}
Coverage requirement: {:.0}%
Test framework: {}

Implementation code:
{}

Generate comprehensive tests that verify the claimed behavior."#,
            self.implementation.location.display(),
            self.test_types,
            self.coverage_requirements * 100.0,
            self.framework,
            self.implementation.code_snippets.join("\n\n")
        )
    }

    fn estimated_effort(&self) -> u8 {
        match self.test_types.len() {
            1 => 3,
            2..=3 => 5,
            _ => 7,
        }
    }

    fn required_skills(&self) -> Vec<String> {
        vec!["testing".to_string(), "programming".to_string()]
    }

    fn is_suitable_for_ai(&self) -> bool {
        true // Test creation is generally good for AI
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixImplementationWorkItem {
    pub failing_tests: Vec<TestResult>,
    pub implementation: Implementation,
    pub error_analysis: String,
    pub suggested_fixes: Vec<String>,
}

impl WorkItemSpec for FixImplementationWorkItem {
    fn to_prompt(&self) -> String {
        format!(
            r#"Fix the implementation to make failing tests pass:

Failing tests:
{}

Current implementation:
{}

Error analysis:
{}

Suggested fixes:
{}

Fix the implementation so all tests pass."#,
            self.failing_tests.iter()
                .map(|t| format!("- Test failed: {}", t.error_message.as_deref().unwrap_or("Unknown error")))
                .collect::<Vec<_>>()
                .join("\n"),
            self.implementation.code_snippets.join("\n\n"),
            self.error_analysis,
            self.suggested_fixes.join("\n- ")
        )
    }

    fn estimated_effort(&self) -> u8 {
        match self.failing_tests.len() {
            1 => 3,
            2..=5 => 5,
            _ => 8,
        }
    }

    fn required_skills(&self) -> Vec<String> {
        vec!["debugging".to_string(), "programming".to_string()]
    }

    fn is_suitable_for_ai(&self) -> bool {
        self.failing_tests.len() <= 3 && !self.error_analysis.contains("complex")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImproveTestsWorkItem {
    pub existing_tests: TestSuite,
    pub semantic_gaps: Vec<SemanticGap>,
    pub coverage_gaps: Vec<String>,
    pub suggestions: Vec<String>,
}

impl WorkItemSpec for ImproveTestsWorkItem {
    fn to_prompt(&self) -> String {
        format!(
            r#"Improve the existing tests to better verify the claim:

Current tests: {} test cases
Semantic gaps found:
{}

Coverage gaps:
{}

Improvement suggestions:
{}

Enhance the test suite to address these gaps."#,
            self.existing_tests.test_cases.len(),
            self.semantic_gaps.iter()
                .map(|g| format!("- {:?}", g))
                .collect::<Vec<_>>()
                .join("\n"),
            self.coverage_gaps.join("\n- "),
            self.suggestions.join("\n- ")
        )
    }

    fn estimated_effort(&self) -> u8 {
        match self.semantic_gaps.len() {
            0..=2 => 2,
            3..=5 => 4,
            _ => 6,
        }
    }

    fn required_skills(&self) -> Vec<String> {
        vec!["testing".to_string()]
    }

    fn is_suitable_for_ai(&self) -> bool {
        true // Test improvement is usually good for AI
    }
}

/// Generator for implementation work items
pub struct ImplementationGenerator {
    llm_client: crate::verification::LlmClient,
}

impl ImplementationGenerator {
    pub fn new(llm_client: crate::verification::LlmClient) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl WorkItemGenerator for ImplementationGenerator {
    async fn generate(&self, claim: &Claim, context: &WorkGenerationContext) -> Result<WorkItem, WorkItemError> {
        // Extract requirements from verification result
        let requirements = if let Some(evidence) = &context.verification_result.evidence {
            // Use existing requirements if available
            vec![] // Would extract from evidence
        } else {
            // Generate new requirements
            vec![Requirement {
                id: uuid::Uuid::new_v4(),
                claim_id: claim.id,
                requirement_type: RequirementType::Functional,
                description: format!("Implement: {}", claim.statement),
                acceptance_criteria: vec!["Must satisfy the claim".to_string()],
                priority: 5,
                extracted_at: chrono::Utc::now(),
            }]
        };

        let spec = ImplementationWorkItem {
            requirements: requirements.clone(),
            suggested_location: Location::File {
                path: "src/generated.rs".to_string(),
                line_range: None,
            },
            interface_spec: "To be determined based on claim analysis".to_string(),
            dependencies: vec![],
            complexity_estimate: 5,
        };

        Ok(WorkItem {
            id: uuid::Uuid::new_v4(),
            work_item_type: WorkItemType::ImplementRequirements,
            claim_id: claim.id,
            title: format!("Implement: {}", claim.statement),
            description: spec.to_prompt(),
            status: WorkItemStatus::Pending,
            created_at: chrono::Utc::now(),
            assignee: None,
            estimated_effort: spec.estimated_effort(),
            required_skills: spec.required_skills(),
            specification: serde_json::to_value(spec).unwrap(),
        })
    }
}

/// Smart assignment strategy that chooses between AI and human
pub struct SmartAssignmentStrategy {
    ai_agents: Vec<AvailableAgent>,
    human_developers: Vec<AvailableDeveloper>,
}

#[derive(Debug, Clone)]
pub struct AvailableAgent {
    pub name: String,
    pub capabilities: Vec<String>,
    pub max_complexity: u8,
    pub available: bool,
}

#[derive(Debug, Clone)]
pub struct AvailableDeveloper {
    pub name: String,
    pub email: String,
    pub skills: Vec<String>,
    pub availability: f64, // 0.0 to 1.0
}

impl SmartAssignmentStrategy {
    pub fn new(ai_agents: Vec<AvailableAgent>, human_developers: Vec<AvailableDeveloper>) -> Self {
        Self {
            ai_agents,
            human_developers,
        }
    }
}

#[async_trait]
impl AssignmentStrategy for SmartAssignmentStrategy {
    async fn assign(&self, work_item: &WorkItem) -> Result<WorkItemAssignment, WorkItemError> {
        // Parse the specification to determine if suitable for AI
        let suitable_for_ai = work_item.estimated_effort <= 7; // Simple heuristic

        if suitable_for_ai {
            // Find available AI agent
            if let Some(agent) = self.ai_agents.iter().find(|a| a.available && a.max_complexity >= work_item.estimated_effort) {
                return Ok(WorkItemAssignment {
                    work_item_id: work_item.id,
                    assignee: Assignee::AiAgent {
                        agent_type: agent.name.clone(),
                        capabilities: agent.capabilities.clone(),
                    },
                    assigned_at: chrono::Utc::now(),
                    estimated_completion: chrono::Utc::now() + chrono::Duration::hours(2),
                    status: AssignmentStatus::Assigned,
                });
            }
        }

        // Fall back to human developer
        if let Some(developer) = self.human_developers.iter()
            .filter(|d| d.availability > 0.5)
            .max_by(|a, b| a.availability.partial_cmp(&b.availability).unwrap())
        {
            Ok(WorkItemAssignment {
                work_item_id: work_item.id,
                assignee: Assignee::Human {
                    name: developer.name.clone(),
                    email: developer.email.clone(),
                },
                assigned_at: chrono::Utc::now(),
                estimated_completion: chrono::Utc::now() + chrono::Duration::days(1),
                status: AssignmentStatus::Assigned,
            })
        } else {
            Err(WorkItemError::Assignment("No available assignees".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implementation_work_item_spec() {
        let spec = ImplementationWorkItem {
            requirements: vec![],
            suggested_location: Location::File {
                path: "src/test.rs".to_string(),
                line_range: None,
            },
            interface_spec: "Test interface".to_string(),
            dependencies: vec!["serde".to_string()],
            complexity_estimate: 5,
        };

        assert_eq!(spec.estimated_effort(), 5);
        assert!(spec.is_suitable_for_ai());
        assert!(spec.to_prompt().contains("Test interface"));
    }

    #[tokio::test]
    async fn test_smart_assignment_strategy() {
        let agents = vec![AvailableAgent {
            name: "CodeGen AI".to_string(),
            capabilities: vec!["rust".to_string(), "testing".to_string()],
            max_complexity: 8,
            available: true,
        }];

        let developers = vec![AvailableDeveloper {
            name: "Jane Developer".to_string(),
            email: "jane@example.com".to_string(),
            skills: vec!["rust".to_string()],
            availability: 0.8,
        }];

        let strategy = SmartAssignmentStrategy::new(agents, developers);

        let work_item = WorkItem {
            id: uuid::Uuid::new_v4(),
            work_item_type: WorkItemType::CreateTests,
            claim_id: uuid::Uuid::new_v4(),
            title: "Test work item".to_string(),
            description: "A test".to_string(),
            status: WorkItemStatus::Pending,
            created_at: chrono::Utc::now(),
            assignee: None,
            estimated_effort: 3, // Low effort - should go to AI
            required_skills: vec!["testing".to_string()],
            specification: serde_json::Value::Null,
        };

        let assignment = strategy.assign(&work_item).await.unwrap();
        
        match assignment.assignee {
            Assignee::AiAgent { agent_type, .. } => {
                assert_eq!(agent_type, "CodeGen AI");
            }
            _ => panic!("Expected AI assignment for low-effort task"),
        }
    }
}