use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Unique identifier for artifacts, claims, and relationships
pub type Id = uuid::Uuid;

/// Timestamp for tracking when artifacts and analysis were created
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Confidence score between 0.0 and 1.0
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Confidence(f64);

impl Confidence {
    pub fn new(value: f64) -> Result<Self, InvalidConfidence> {
        if (0.0..=1.0).contains(&value) {
            Ok(Confidence(value))
        } else {
            Err(InvalidConfidence(value))
        }
    }
    
    pub fn value(&self) -> f64 {
        self.0
    }
}

#[derive(Error, Debug)]
#[error("Confidence must be between 0.0 and 1.0, got {0}")]
pub struct InvalidConfidence(f64);

/// Location where an artifact exists - could be file path, URL, commit hash, etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Location {
    File { path: String, line_range: Option<(u32, u32)> },
    Url(String),
    Commit { hash: String, file_path: Option<String> },
    Ticket { system: String, id: String },
    Memory { session_id: String, interaction_id: String },
}

impl Location {
    pub fn display(&self) -> String {
        match self {
            Location::File { path, line_range: Some((start, end)) } => {
                format!("{}:{}-{}", path, start, end)
            }
            Location::File { path, line_range: None } => path.clone(),
            Location::Url(url) => url.clone(),
            Location::Commit { hash, file_path: Some(path) } => {
                format!("{}:{}", &hash[..8], path)
            }
            Location::Commit { hash, file_path: None } => hash[..8].to_string(),
            Location::Ticket { system, id } => format!("{}:{}", system, id),
            Location::Memory { session_id, interaction_id } => {
                format!("session:{}:{}", &session_id[..8], interaction_id)
            }
        }
    }
}

/// Types of artifacts that can be analyzed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactType {
    /// Source code files
    Code,
    /// Test files and test cases
    Test,
    /// Documentation files (markdown, rst, etc.)
    Documentation,
    /// Git commit messages and metadata
    Commit,
    /// Issue tracking tickets (JIRA, GitHub issues, etc.)
    Ticket,
    /// Code review comments
    Comment,
    /// Technical specifications and requirements
    Specification,
    /// Meeting notes or design discussions
    Discussion,
}

/// A piece of content that can be analyzed for claims
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    pub id: Id,
    pub artifact_type: ArtifactType,
    pub content: String,
    pub location: Location,
    pub created_at: Timestamp,
    pub author: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Types of claims that can be extracted from artifacts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaimType {
    /// Claims about what the system does or should do
    Functional,
    /// Claims about performance characteristics
    Performance,
    /// Claims about security properties
    Security,
    /// Claims about user-facing behavior
    Behavior,
    /// Claims about code structure or architecture
    Structure,
    /// Claims about business requirements
    Requirement,
    /// Claims about testing coverage or validation
    Testing,
}

/// A statement extracted from an artifact that makes a claim about the system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    pub id: Id,
    pub artifact_id: Id,
    /// The actual claim being made (e.g., "This function handles user authentication")
    pub statement: String,
    /// How confident we are that this claim is actually being made
    pub extraction_confidence: Confidence,
    pub claim_type: ClaimType,
    /// The specific part of the artifact this claim was extracted from
    pub source_excerpt: String,
    pub extracted_at: Timestamp,
}

/// Types of relationships between artifacts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Code implements a requirement or specification
    Implements,
    /// Tests verify behavior or functionality
    Tests,
    /// Documentation explains or describes
    Documents,
    /// Artifact references or mentions another
    References,
    /// Artifacts make contradictory claims
    Contradicts,
    /// Artifact supersedes or replaces another
    Supersedes,
    /// Artifacts are part of the same feature or epic
    RelatedTo,
}

/// Semantic relationship between two artifacts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Id,
    pub source_artifact_id: Id,
    pub target_artifact_id: Id,
    pub relationship_type: RelationshipType,
    pub confidence: Confidence,
    pub discovered_at: Timestamp,
    pub metadata: HashMap<String, String>,
}

/// How well evidence supports a claim
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alignment {
    pub id: Id,
    pub claim_id: Id,
    pub evidence_artifact_id: Id,
    /// Score from 0.0 to 1.0 indicating alignment strength
    pub alignment_score: Confidence,
    /// LLM's explanation of the alignment assessment
    pub explanation: String,
    pub checked_at: Timestamp,
}

/// A detected gap or misalignment in the system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GapType {
    /// Claim has no supporting evidence
    NoEvidence,
    /// Claim has only weak supporting evidence
    WeakEvidence,
    /// Contradictory claims exist
    Contradiction,
    /// Implementation exists but no tests
    Untested,
    /// Implementation exists but not documented
    Undocumented,
    /// Requirement exists but no implementation
    Unimplemented,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gap {
    pub id: Id,
    pub gap_type: GapType,
    pub severity: Severity,
    pub primary_claim_id: Id,
    pub related_artifact_ids: Vec<Id>,
    pub description: String,
    pub detected_at: Timestamp,
}

/// Analysis results for project health metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectHealth {
    pub total_claims: usize,
    pub fully_supported_claims: usize,
    pub partially_supported_claims: usize,
    pub unsupported_claims: usize,
    pub average_alignment_score: f64,
    pub gaps_by_severity: HashMap<Severity, usize>,
    pub coverage_metrics: CoverageMetrics,
    pub analyzed_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageMetrics {
    pub code_files_with_tests: f64,
    pub tests_with_documentation: f64,
    pub tickets_with_implementation: f64,
    pub commits_matching_changes: f64,
}

/// Configuration for narrative analysis and claim extraction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Minimum confidence threshold for extracted claims
    pub min_claim_confidence: Confidence,
    /// Minimum confidence threshold for relationships
    pub min_relationship_confidence: Confidence,
    /// Minimum alignment score to consider evidence as supporting
    pub min_alignment_threshold: Confidence,
    /// Whether to analyze historical versions of artifacts
    pub include_historical: bool,
    /// Types of artifacts to analyze
    pub artifact_types: Vec<ArtifactType>,
    /// Custom prompts for different analysis types
    pub custom_prompts: HashMap<String, String>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            min_claim_confidence: Confidence::new(0.6).unwrap(),
            min_relationship_confidence: Confidence::new(0.5).unwrap(),
            min_alignment_threshold: Confidence::new(0.5).unwrap(),
            include_historical: false,
            artifact_types: vec![
                ArtifactType::Code,
                ArtifactType::Test,
                ArtifactType::Documentation,
                ArtifactType::Ticket,
                ArtifactType::Commit,
            ],
            custom_prompts: HashMap::new(),
        }
    }
}