use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StoryGraph {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub nodes: HashMap<Uuid, StoryNode>,
    pub edges: Vec<StoryEdge>,
    pub root_node_id: Option<Uuid>,
    pub current_node_id: Option<Uuid>,
    pub metadata: StoryMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StoryNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub situation: String,
    pub choices: Vec<Choice>,
    pub state: NodeState,
    pub complexity_score: Option<f32>,
    pub metadata: NodeMetadata,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Choice {
    pub id: Uuid,
    pub description: String,
    pub target_node_id: Option<Uuid>,
    pub weight: f32,
    pub feasibility_score: Option<f32>,
    pub consequences: Vec<String>,
    pub metadata: ChoiceMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StoryEdge {
    pub id: Uuid,
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub choice_id: Uuid,
    pub traversal_count: u32,
    pub metadata: EdgeMetadata,
}


#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum NodeType {
    Start,
    Decision,
    Action,
    Outcome,
    Question,
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum NodeState {
    Unvisited,
    Current,
    Visited,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StoryMetadata {
    pub owner_id: Option<Uuid>,
    pub is_public: bool,
    pub tags: Vec<String>,
    pub project_constraints: ProjectConstraints,
    pub analytics: StoryAnalytics,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NodeMetadata {
    pub generated_by_ai: bool,
    pub validation_status: ValidationStatus,
    pub user_annotations: Vec<String>,
    pub technical_details: Vec<TechnicalDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChoiceMetadata {
    pub confidence_score: Option<f32>,
    pub risk_level: RiskLevel,
    pub time_estimate: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EdgeMetadata {
    pub decision_timestamp: Option<DateTime<Utc>>,
    pub user_feedback: Option<String>,
    pub success_probability: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectConstraints {
    pub timeline: Option<String>,
    pub team_size: Option<u32>,
    pub experience_level: ExperienceLevel,
    pub budget: Option<String>,
    pub technical_constraints: Vec<String>,
    pub business_constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StoryAnalytics {
    pub total_nodes: u32,
    pub completion_percentage: f32,
    pub average_complexity: f32,
    pub decision_points: u32,
    pub estimated_timeline: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TechnicalDetail {
    pub category: String,
    pub description: String,
    pub importance: ImportanceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ValidationStatus {
    Valid,
    Warning,
    Error,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ImportanceLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StoryContext {
    pub previous_choices: Vec<Uuid>,
    pub user_responses: Vec<UserResponse>,
    pub current_constraints: ProjectConstraints,
    pub session_metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UserResponse {
    pub question_id: Uuid,
    #[ts(type = "any")]
    pub response_data: serde_json::Value,
    pub confidence: Option<f32>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionMetadata {
    pub session_id: Uuid,
    pub user_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CoherenceReport {
    pub is_coherent: bool,
    pub confidence_score: f32,
    pub issues: Vec<CoherenceIssue>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CoherenceIssue {
    pub issue_type: CoherenceIssueType,
    pub description: String,
    pub severity: IssueSeverity,
    pub affected_nodes: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum CoherenceIssueType {
    LogicalInconsistency,
    DependencyConflict,
    TimelineConflict,
    ResourceConflict,
    NarrativeInconsistency,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Question {
    pub id: Uuid,
    pub question_type: QuestionType,
    pub prompt: String,
    pub options: Option<Vec<QuestionOption>>,
    pub validation_rules: ValidationRules,
    pub context: QuestionContext,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QuestionOption {
    pub id: String,
    pub label: String,
    #[ts(type = "any")]
    pub value: serde_json::Value,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ValidationRules {
    pub required: bool,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    pub pattern: Option<String>,
    pub custom_validator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QuestionContext {
    pub uncertainty_areas: Vec<String>,
    pub missing_context: Vec<String>,
    pub conflicting_choices: Vec<Uuid>,
    pub priority_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum QuestionType {
    MultipleChoice,
    FreeText,
    NumericRange,
    Boolean,
    TechnicalSelection,
    PriorityRanking,
    ResourceAllocation,
    RiskAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UncertaintyContext {
    pub uncertainty_level: f32,
    pub missing_context: Vec<String>,
    pub conflicting_choices: Vec<Choice>,
    pub ambiguous_requirements: Vec<String>,
}

impl Default for StoryGraph {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            description: None,
            nodes: HashMap::new(),
            edges: Vec::new(),
            root_node_id: None,
            current_node_id: None,
            metadata: StoryMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl Default for StoryMetadata {
    fn default() -> Self {
        Self {
            owner_id: None,
            is_public: false,
            tags: Vec::new(),
            project_constraints: ProjectConstraints::default(),
            analytics: StoryAnalytics::default(),
        }
    }
}

impl Default for ProjectConstraints {
    fn default() -> Self {
        Self {
            timeline: None,
            team_size: None,
            experience_level: ExperienceLevel::Intermediate,
            budget: None,
            technical_constraints: Vec::new(),
            business_constraints: Vec::new(),
        }
    }
}

impl Default for StoryAnalytics {
    fn default() -> Self {
        Self {
            total_nodes: 0,
            completion_percentage: 0.0,
            average_complexity: 0.0,
            decision_points: 0,
            estimated_timeline: None,
        }
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            required: false,
            min_length: None,
            max_length: None,
            pattern: None,
            custom_validator: None,
        }
    }
}