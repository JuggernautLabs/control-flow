//! Semantic Alignment Tracking System (SATS) Core Library
//! 
//! This library provides the foundational types, traits, and interfaces for the SATS system,
//! which tracks semantic relationships between project artifacts and measures alignment
//! between requirements, code, tests, and documentation.

pub mod types;
pub mod analysis;
pub mod alignment;
pub mod storage;
pub mod claude_impl;

// Re-export commonly used types for convenience
pub use types::{
    Artifact, ArtifactType, Claim, ClaimType, Relationship, RelationshipType,
    Alignment, Gap, GapType, Severity, ProjectHealth, AnalysisConfig,
    Confidence, Location, Id, Timestamp, CoverageMetrics,
};

pub use analysis::{
    ClaimExtractor, RelationshipDiscoverer, AlignmentChecker, GapAnalyzer,
    AnalysisError, ClaimExtractionResult, RelationshipDiscoveryResult, AlignmentResult,
    ContextualClaimExtractor, ExtractionContext, ClaimExtractionStrategy,
};

pub use alignment::{
    AlignmentScore, AlignmentDimensions, EvidencePoint, EvidenceType,
    RelationshipGraph, RelationshipChain, MultiEvidenceAlignmentChecker,
    RelationshipEvolutionTracker, RelationshipDriftAnalysis,
};

pub use storage::{
    SatsStorage, ArtifactIngester, StorageError, IngestionError,
    QueryFilter, QueryResult, IngestionResult, StorageConfig,
    ExternalSource, ExternalSourceConfig, FileTypeDetector,
    StorageCache, CachedStorage,
};

pub use claude_impl::{
    ClaudeClaimExtractor, ClaudeAlignmentChecker,
};

/// Version of the SATS core library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type commonly used throughout the SATS system
pub type SatsResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Common trait bounds for types that can be used in SATS operations
pub trait SatsEntity: Send + Sync + Clone + std::fmt::Debug {}

impl SatsEntity for Artifact {}
impl SatsEntity for Claim {}
impl SatsEntity for Relationship {}
impl SatsEntity for Alignment {}
impl SatsEntity for Gap {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_confidence_creation() {
        assert!(Confidence::new(0.5).is_ok());
        assert!(Confidence::new(1.0).is_ok());
        assert!(Confidence::new(0.0).is_ok());
        assert!(Confidence::new(1.1).is_err());
        assert!(Confidence::new(-0.1).is_err());
    }

    #[test]
    fn test_location_display() {
        let file_location = Location::File {
            path: "src/main.rs".to_string(),
            line_range: Some((10, 20)),
        };
        assert_eq!(file_location.display(), "src/main.rs:10-20");

        let commit_location = Location::Commit {
            hash: "abcdef1234567890".to_string(),
            file_path: Some("README.md".to_string()),
        };
        assert_eq!(commit_location.display(), "abcdef12:README.md");
    }

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();
        assert_eq!(config.min_claim_confidence.value(), 0.6);
        assert_eq!(config.min_relationship_confidence.value(), 0.5);
        assert_eq!(config.min_alignment_threshold.value(), 0.5);
        assert!(!config.include_historical);
        assert!(config.artifact_types.contains(&ArtifactType::Code));
        assert!(config.artifact_types.contains(&ArtifactType::Test));
    }

    #[test]
    fn test_artifact_creation() {
        let artifact = Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type: ArtifactType::Code,
            content: "fn main() { println!(\"Hello, world!\"); }".to_string(),
            location: Location::File {
                path: "src/main.rs".to_string(),
                line_range: None,
            },
            created_at: chrono::Utc::now(),
            author: Some("developer@example.com".to_string()),
            metadata: HashMap::new(),
        };

        assert_eq!(artifact.artifact_type, ArtifactType::Code);
        assert!(artifact.content.contains("Hello, world!"));
    }
}