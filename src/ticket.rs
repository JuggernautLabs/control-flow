use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::fs;
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketDecomposition {
    #[serde(rename = "originalTicket")]
    pub original_ticket: OriginalTicket,
    #[serde(rename = "decomposedTicket")]
    pub decomposed_ticket: DecomposedTicket,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OriginalTicket {
    pub title: String,
    #[serde(rename = "rawInput")]
    pub raw_input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecomposedTicket {
    pub terms: HashMap<String, String>,
    #[serde(rename = "termsNeedingRefinement")]
    pub terms_needing_refinement: Vec<RefinementRequest>,
    #[serde(rename = "openQuestions")]
    pub open_questions: Vec<String>,
    #[serde(rename = "engineQuestions")]
    pub engine_questions: Vec<String>,
    #[serde(rename = "validationMethod")]
    pub validation_method: Vec<String>,
    #[serde(rename = "validationResults")]
    pub validation_results: ValidationResults,
    pub metadata: TicketMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResults {
    pub mime: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketMetadata {
    pub status: TicketStatus,
    pub priority: Priority,
    #[serde(rename = "estimatedComplexity")]
    pub estimated_complexity: Complexity,
    #[serde(rename = "processedAt")]
    pub processed_at: String,
    #[serde(rename = "engineVersion")]
    pub engine_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TicketStatus {
    #[serde(rename = "AWAITING_REFINEMENT")]
    AwaitingRefinement,
    #[serde(rename = "IN_PROGRESS")]
    InProgress,
    #[serde(rename = "UNDER_REVIEW")]
    UnderReview,
    #[serde(rename = "COMPLETE")]
    Complete,
    #[serde(rename = "BLOCKED")]
    Blocked,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Priority {
    #[serde(rename = "LOW")]
    Low,
    #[serde(rename = "MEDIUM")]
    Medium,
    #[serde(rename = "HIGH")]
    High,
    #[serde(rename = "CRITICAL")]
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Complexity {
    #[serde(rename = "LOW")]
    Low,
    #[serde(rename = "MEDIUM")]
    Medium,
    #[serde(rename = "MEDIUM_HIGH")]
    MediumHigh,
    #[serde(rename = "HIGH")]
    High,
    #[serde(rename = "VERY_HIGH")]
    VeryHigh,
}

#[derive(Debug, Serialize, Clone)]
pub struct RefinementRequest {
    pub term: String,
    pub context: String,
    pub reason: String,
    pub priority: RefinementPriority,
}

impl<'de> serde::Deserialize<'de> for RefinementRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct RefinementRequestVisitor;

        impl<'de> Visitor<'de> for RefinementRequestVisitor {
            type Value = RefinementRequest;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or a structured refinement request object")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Handle legacy string format - convert to structured format
                Ok(RefinementRequest {
                    term: value.to_string(),
                    context: "Legacy format - context not specified".to_string(),
                    reason: "Needs refinement (converted from legacy string format)".to_string(),
                    priority: RefinementPriority::Medium,
                })
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut term = None;
                let mut context = None;
                let mut reason = None;
                let mut priority = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "term" => {
                            if term.is_some() {
                                return Err(de::Error::duplicate_field("term"));
                            }
                            term = Some(map.next_value()?);
                        }
                        "context" => {
                            if context.is_some() {
                                return Err(de::Error::duplicate_field("context"));
                            }
                            context = Some(map.next_value()?);
                        }
                        "reason" => {
                            if reason.is_some() {
                                return Err(de::Error::duplicate_field("reason"));
                            }
                            reason = Some(map.next_value()?);
                        }
                        "priority" => {
                            if priority.is_some() {
                                return Err(de::Error::duplicate_field("priority"));
                            }
                            priority = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore unknown fields
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let term = term.ok_or_else(|| de::Error::missing_field("term"))?;
                let context = context.unwrap_or_else(|| "Context not specified".to_string());
                let reason = reason.unwrap_or_else(|| "Needs refinement".to_string());
                let priority = priority.unwrap_or(RefinementPriority::Medium);

                Ok(RefinementRequest {
                    term,
                    context,
                    reason,
                    priority,
                })
            }
        }

        deserializer.deserialize_any(RefinementRequestVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RefinementPriority {
    #[serde(rename = "LOW")]
    Low,
    #[serde(rename = "MEDIUM")]
    Medium,
    #[serde(rename = "HIGH")]
    High,
    #[serde(rename = "CRITICAL")]
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefinementContext {
    pub parent_ticket_id: TicketId,
    pub term_being_refined: String,
    pub original_context: String,
    pub additional_context: Vec<TicketId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub graph: TicketGraph,
    pub created_at: String,
    pub updated_at: String,
}

impl Project {
    pub fn new(name: String, description: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            name,
            description,
            graph: TicketGraph::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn add_ticket(&mut self, ticket: TicketDecomposition) -> TicketId {
        self.updated_at = chrono::Utc::now().to_rfc3339();
        self.graph.add_ticket(ticket)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let project: Project = serde_json::from_str(&content)?;
        Ok(project)
    }

    pub fn get_root_tickets(&self) -> Vec<&TicketId> {
        self.graph.get_root_tickets()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectManager {
    pub projects: HashMap<String, PathBuf>,
    pub workspace_dir: PathBuf,
}

impl ProjectManager {
    pub fn new<P: AsRef<Path>>(workspace_dir: P) -> Result<Self, Box<dyn std::error::Error>> {
        let workspace_dir = workspace_dir.as_ref().to_path_buf();
        fs::create_dir_all(&workspace_dir)?;
        
        Ok(Self {
            projects: HashMap::new(),
            workspace_dir,
        })
    }

    pub fn create_project(&mut self, name: String, description: String) -> Result<(), Box<dyn std::error::Error>> {
        if self.projects.contains_key(&name) {
            return Err(format!("Project '{}' already exists", name).into());
        }

        // Ensure workspace directory exists
        fs::create_dir_all(&self.workspace_dir).map_err(|e| {
            format!("Failed to create workspace directory '{}': {}", self.workspace_dir.display(), e)
        })?;

        let project = Project::new(name.clone(), description);
        
        // Sanitize filename by replacing spaces with underscores
        let safe_filename = name.replace(' ', "_").replace('/', "_");
        let project_path = self.workspace_dir.join(format!("{}.json", safe_filename));
        
        project.save_to_file(&project_path).map_err(|e| {
            format!("Failed to save project to '{}': {}", project_path.display(), e)
        })?;
        
        self.projects.insert(name, project_path);
        self.save_index().map_err(|e| {
            format!("Failed to save project index: {}", e)
        })?;
        Ok(())
    }

    pub fn load_project(&self, name: &str) -> Result<Project, Box<dyn std::error::Error>> {
        let path = self.projects.get(name)
            .ok_or_else(|| format!("Project '{}' not found", name))?;
        Project::load_from_file(path)
    }

    pub fn save_project(&self, project: &Project) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.projects.get(&project.name)
            .ok_or_else(|| format!("Project '{}' not found in index", project.name))?;
        project.save_to_file(path)
    }

    pub fn list_projects(&self) -> Vec<&String> {
        self.projects.keys().collect()
    }

    pub fn delete_project(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.projects.remove(name)
            .ok_or_else(|| format!("Project '{}' not found", name))?;
        
        if path.exists() {
            fs::remove_file(path)?;
        }
        self.save_index()?;
        Ok(())
    }

    fn save_index(&self) -> Result<(), Box<dyn std::error::Error>> {
        let index_path = self.workspace_dir.join("projects.json");
        let json = serde_json::to_string_pretty(&self.projects)?;
        fs::write(index_path, json)?;
        Ok(())
    }

    pub fn load_index<P: AsRef<Path>>(workspace_dir: P) -> Result<Self, Box<dyn std::error::Error>> {
        let workspace_dir = workspace_dir.as_ref().to_path_buf();
        let index_path = workspace_dir.join("projects.json");
        
        let projects = if index_path.exists() {
            let content = fs::read_to_string(index_path)?;
            serde_json::from_str(&content)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            projects,
            workspace_dir,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TicketId(String);

impl TicketId {
    pub fn generate(ticket: &TicketDecomposition) -> Self {
        let serialized = serde_json::to_string(ticket).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let result = hasher.finalize();
        let hash_string = format!("{:x}", result);
        TicketId(hash_string[..16].to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TicketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketNode {
    pub id: TicketId,
    pub ticket: TicketDecomposition,
    pub dependencies: HashSet<TicketId>,
    pub dependents: HashSet<TicketId>,
    pub created_at: String,
    pub updated_at: String,
}

impl TicketNode {
    pub fn new(ticket: TicketDecomposition) -> Self {
        let id = TicketId::generate(&ticket);
        let now = chrono::Utc::now().to_rfc3339();
        
        Self {
            id,
            ticket,
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn add_dependency(&mut self, dependency_id: TicketId) {
        self.dependencies.insert(dependency_id);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn add_dependent(&mut self, dependent_id: TicketId) {
        self.dependents.insert(dependent_id);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn remove_dependency(&mut self, dependency_id: &TicketId) {
        self.dependencies.remove(dependency_id);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn remove_dependent(&mut self, dependent_id: &TicketId) {
        self.dependents.remove(dependent_id);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketGraph {
    pub nodes: HashMap<TicketId, TicketNode>,
    pub created_at: String,
    pub updated_at: String,
}

impl TicketGraph {
    pub fn new() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            nodes: HashMap::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn add_ticket(&mut self, ticket: TicketDecomposition) -> TicketId {
        let node = TicketNode::new(ticket);
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        self.updated_at = chrono::Utc::now().to_rfc3339();
        id
    }

    pub fn get_ticket(&self, id: &TicketId) -> Option<&TicketNode> {
        self.nodes.get(id)
    }

    pub fn get_ticket_mut(&mut self, id: &TicketId) -> Option<&mut TicketNode> {
        self.nodes.get_mut(id)
    }

    pub fn add_dependency(&mut self, dependent_id: &TicketId, dependency_id: &TicketId) -> Result<(), String> {
        if !self.nodes.contains_key(dependent_id) {
            return Err(format!("Dependent ticket {} not found", dependent_id));
        }
        if !self.nodes.contains_key(dependency_id) {
            return Err(format!("Dependency ticket {} not found", dependency_id));
        }

        if self.would_create_cycle(dependent_id, dependency_id) {
            return Err("Adding this dependency would create a cycle".to_string());
        }

        let dependent_node = self.nodes.get_mut(dependent_id).unwrap();
        dependent_node.add_dependency(dependency_id.clone());

        let dependency_node = self.nodes.get_mut(dependency_id).unwrap();
        dependency_node.add_dependent(dependent_id.clone());

        self.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn remove_dependency(&mut self, dependent_id: &TicketId, dependency_id: &TicketId) {
        if let Some(dependent_node) = self.nodes.get_mut(dependent_id) {
            dependent_node.remove_dependency(dependency_id);
        }
        if let Some(dependency_node) = self.nodes.get_mut(dependency_id) {
            dependency_node.remove_dependent(dependent_id);
        }
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn get_dependencies(&self, id: &TicketId) -> Option<&HashSet<TicketId>> {
        self.nodes.get(id).map(|node| &node.dependencies)
    }

    pub fn get_dependents(&self, id: &TicketId) -> Option<&HashSet<TicketId>> {
        self.nodes.get(id).map(|node| &node.dependents)
    }

    pub fn get_root_tickets(&self) -> Vec<&TicketId> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.dependencies.is_empty())
            .map(|(id, _)| id)
            .collect()
    }

    pub fn get_leaf_tickets(&self) -> Vec<&TicketId> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.dependents.is_empty())
            .map(|(id, _)| id)
            .collect()
    }

    fn would_create_cycle(&self, dependent_id: &TicketId, dependency_id: &TicketId) -> bool {
        // Check if dependency_id can reach dependent_id through existing dependencies
        let mut visited = HashSet::new();
        self.has_path_to(dependency_id, dependent_id, &mut visited)
    }

    fn has_path_to(&self, from: &TicketId, to: &TicketId, visited: &mut HashSet<TicketId>) -> bool {
        if from == to {
            return true;
        }

        if visited.contains(from) {
            return false;
        }

        visited.insert(from.clone());

        if let Some(node) = self.nodes.get(from) {
            // Follow the dependency chain (things this node depends on)
            for dependency in &node.dependencies {
                if self.has_path_to(dependency, to, visited) {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for TicketGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_ticket() -> TicketDecomposition {
        TicketDecomposition {
            original_ticket: OriginalTicket {
                title: "Test Ticket".to_string(),
                raw_input: "Create a test ticket".to_string(),
            },
            decomposed_ticket: DecomposedTicket {
                terms: HashMap::new(),
                terms_needing_refinement: vec![],
                open_questions: vec![],
                engine_questions: vec![],
                validation_method: vec![],
                validation_results: ValidationResults {
                    mime: "text/plain".to_string(),
                    url: "placeholder".to_string(),
                },
                metadata: TicketMetadata {
                    status: TicketStatus::AwaitingRefinement,
                    priority: Priority::Medium,
                    estimated_complexity: Complexity::Low,
                    processed_at: "2024-01-01T00:00:00Z".to_string(), // Fixed timestamp for tests
                    engine_version: "1.0".to_string(),
                },
            },
        }
    }

    #[test]
    fn test_ticket_id_generation() {
        let ticket1 = create_test_ticket();
        let ticket2 = create_test_ticket();
        
        let id1 = TicketId::generate(&ticket1);
        let id2 = TicketId::generate(&ticket2);
        
        assert_eq!(id1, id2); // Same ticket content should generate same ID
        assert_eq!(id1.as_str().len(), 16); // Should be 16 characters
    }

    #[test]
    fn test_ticket_graph_operations() {
        let mut graph = TicketGraph::new();
        
        let ticket1 = create_test_ticket();
        let ticket2 = create_test_ticket();
        
        let id1 = graph.add_ticket(ticket1);
        let id2 = graph.add_ticket(ticket2);
        
        assert_eq!(graph.nodes.len(), 1); // Same content = same ID = only one ticket
        assert!(graph.get_ticket(&id1).is_some());
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_dependencies() {
        let mut graph = TicketGraph::new();
        
        let mut ticket1 = create_test_ticket();
        ticket1.original_ticket.title = "Ticket 1".to_string();
        
        let mut ticket2 = create_test_ticket();
        ticket2.original_ticket.title = "Ticket 2".to_string();
        
        let id1 = graph.add_ticket(ticket1);
        let id2 = graph.add_ticket(ticket2);
        
        // Add dependency: ticket2 depends on ticket1
        assert!(graph.add_dependency(&id2, &id1).is_ok());
        
        let deps = graph.get_dependencies(&id2).unwrap();
        assert!(deps.contains(&id1));
        
        let dependents = graph.get_dependents(&id1).unwrap();
        assert!(dependents.contains(&id2));
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = TicketGraph::new();
        
        let mut ticket1 = create_test_ticket();
        ticket1.original_ticket.title = "Ticket 1".to_string();
        
        let mut ticket2 = create_test_ticket();
        ticket2.original_ticket.title = "Ticket 2".to_string();
        
        let id1 = graph.add_ticket(ticket1);
        let id2 = graph.add_ticket(ticket2);
        
        // Create dependency: id2 -> id1
        assert!(graph.add_dependency(&id2, &id1).is_ok());
        
        // Try to create cycle: id1 -> id2 (should fail)
        assert!(graph.add_dependency(&id1, &id2).is_err());
    }

    #[test]
    fn test_project_creation() {
        let project = Project::new("Test Project".to_string(), "A test project".to_string());
        
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, "A test project");
        assert_eq!(project.graph.nodes.len(), 0);
    }

    #[test]
    fn test_project_manager() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ProjectManager::new(temp_dir.path()).unwrap();
        
        // Create project
        assert!(manager.create_project("test-project".to_string(), "Test description".to_string()).is_ok());
        
        // List projects
        let projects = manager.list_projects();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0], "test-project");
        
        // Load project
        let project = manager.load_project("test-project").unwrap();
        assert_eq!(project.name, "test-project");
        assert_eq!(project.description, "Test description");
        
        // Try to create duplicate project
        assert!(manager.create_project("test-project".to_string(), "Duplicate".to_string()).is_err());
        
        // Delete project
        assert!(manager.delete_project("test-project").is_ok());
        assert_eq!(manager.list_projects().len(), 0);
    }

    #[test]
    fn test_project_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ProjectManager::new(temp_dir.path()).unwrap();
        
        // Create and save project
        manager.create_project("persistent-project".to_string(), "Test persistence".to_string()).unwrap();
        
        // Create new manager and load index
        let manager2 = ProjectManager::load_index(temp_dir.path()).unwrap();
        let projects = manager2.list_projects();
        
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0], "persistent-project");
        
        // Load the project
        let project = manager2.load_project("persistent-project").unwrap();
        assert_eq!(project.name, "persistent-project");
        assert_eq!(project.description, "Test persistence");
    }

    #[test]
    fn test_refinement_request() {
        let refinement = RefinementRequest {
            term: "API".to_string(),
            context: "Web service context".to_string(),
            reason: "Need technical specification".to_string(),
            priority: RefinementPriority::High,
        };
        
        assert_eq!(refinement.term, "API");
        assert_eq!(refinement.context, "Web service context");
        matches!(refinement.priority, RefinementPriority::High);
    }

    #[test]
    fn test_refinement_request_deserialization_from_string() {
        // Test deserializing from a simple string (legacy format)
        let json_string = r#""user interface""#;
        let refinement: RefinementRequest = serde_json::from_str(json_string).unwrap();
        
        assert_eq!(refinement.term, "user interface");
        assert_eq!(refinement.context, "Legacy format - context not specified");
        assert_eq!(refinement.reason, "Needs refinement (converted from legacy string format)");
        matches!(refinement.priority, RefinementPriority::Medium);
    }

    #[test]
    fn test_refinement_request_deserialization_from_object() {
        // Test deserializing from structured object (new format)
        let json_obj = r#"{
            "term": "user-friendly",
            "context": "create a user-friendly interface",
            "reason": "ambiguous - could mean accessible, intuitive, minimalist, or mobile-responsive",
            "priority": "HIGH"
        }"#;
        let refinement: RefinementRequest = serde_json::from_str(json_obj).unwrap();
        
        assert_eq!(refinement.term, "user-friendly");
        assert_eq!(refinement.context, "create a user-friendly interface");
        assert_eq!(refinement.reason, "ambiguous - could mean accessible, intuitive, minimalist, or mobile-responsive");
        matches!(refinement.priority, RefinementPriority::High);
    }

    #[test]
    fn test_refinement_request_partial_object() {
        // Test with missing optional fields
        let json_obj = r#"{
            "term": "scalable"
        }"#;
        let refinement: RefinementRequest = serde_json::from_str(json_obj).unwrap();
        
        assert_eq!(refinement.term, "scalable");
        assert_eq!(refinement.context, "Context not specified");
        assert_eq!(refinement.reason, "Needs refinement");
        matches!(refinement.priority, RefinementPriority::Medium);
    }

    #[test]
    fn test_mixed_refinement_formats_in_array() {
        // Test an array with both string and object formats
        let json_array = r#"[
            "legacy string term",
            {
                "term": "structured term",
                "context": "in the requirements",
                "reason": "needs clarification",
                "priority": "LOW"
            }
        ]"#;
        
        let refinements: Vec<RefinementRequest> = serde_json::from_str(json_array).unwrap();
        assert_eq!(refinements.len(), 2);
        
        // First item (legacy string)
        assert_eq!(refinements[0].term, "legacy string term");
        assert_eq!(refinements[0].context, "Legacy format - context not specified");
        
        // Second item (structured)
        assert_eq!(refinements[1].term, "structured term");
        assert_eq!(refinements[1].context, "in the requirements");
        assert_eq!(refinements[1].reason, "needs clarification");
        matches!(refinements[1].priority, RefinementPriority::Low);
    }

    #[test]
    fn test_project_creation_with_spaces() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ProjectManager::new(temp_dir.path()).unwrap();
        
        // Create project with spaces in name
        assert!(manager.create_project("todo app".to_string(), "A todo application".to_string()).is_ok());
        
        // Verify it was created and can be loaded
        let projects = manager.list_projects();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0], "todo app");
        
        let project = manager.load_project("todo app").unwrap();
        assert_eq!(project.name, "todo app");
        assert_eq!(project.description, "A todo application");
        
        // Verify the file was created with sanitized filename
        let expected_path = temp_dir.path().join("todo_app.json");
        assert!(expected_path.exists());
    }
}