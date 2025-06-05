use crate::types::*;
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Error types for storage operations
#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("Connection failed: {0}")]
    Connection(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    #[error("Storage backend error: {0}")]
    Backend(String),
}

/// Query filters for retrieving artifacts and analysis results
#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    /// Filter by artifact types
    pub artifact_types: Option<Vec<ArtifactType>>,
    /// Filter by date range
    pub date_range: Option<(Timestamp, Timestamp)>,
    /// Filter by author
    pub author: Option<String>,
    /// Filter by location pattern (e.g., "src/*.rs")
    pub location_pattern: Option<String>,
    /// Filter by confidence threshold
    pub min_confidence: Option<Confidence>,
    /// Filter by metadata key-value pairs
    pub metadata_filters: HashMap<String, String>,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Results from storage queries with metadata
#[derive(Debug, Clone)]
pub struct QueryResult<T> {
    pub items: Vec<T>,
    pub total_count: usize,
    pub query_time_ms: u64,
    pub has_more: bool,
}

/// Core storage interface for all SATS data
#[async_trait]
pub trait SatsStorage: Send + Sync {
    // Artifact operations
    async fn store_artifact(&self, artifact: &Artifact) -> Result<(), StorageError>;
    async fn get_artifact(&self, id: &Id) -> Result<Option<Artifact>, StorageError>;
    async fn query_artifacts(&self, filter: &QueryFilter) -> Result<QueryResult<Artifact>, StorageError>;
    async fn update_artifact(&self, artifact: &Artifact) -> Result<(), StorageError>;
    async fn delete_artifact(&self, id: &Id) -> Result<bool, StorageError>;
    
    // Claim operations
    async fn store_claim(&self, claim: &Claim) -> Result<(), StorageError>;
    async fn get_claim(&self, id: &Id) -> Result<Option<Claim>, StorageError>;
    async fn get_claims_for_artifact(&self, artifact_id: &Id) -> Result<Vec<Claim>, StorageError>;
    async fn query_claims(&self, filter: &QueryFilter) -> Result<QueryResult<Claim>, StorageError>;
    async fn delete_claim(&self, id: &Id) -> Result<bool, StorageError>;
    
    // Relationship operations
    async fn store_relationship(&self, relationship: &Relationship) -> Result<(), StorageError>;
    async fn get_relationship(&self, id: &Id) -> Result<Option<Relationship>, StorageError>;
    async fn get_relationships_for_artifact(&self, artifact_id: &Id) -> Result<Vec<Relationship>, StorageError>;
    async fn query_relationships(&self, filter: &QueryFilter) -> Result<QueryResult<Relationship>, StorageError>;
    async fn delete_relationship(&self, id: &Id) -> Result<bool, StorageError>;
    
    // Alignment operations
    async fn store_alignment(&self, alignment: &Alignment) -> Result<(), StorageError>;
    async fn get_alignment(&self, id: &Id) -> Result<Option<Alignment>, StorageError>;
    async fn get_alignments_for_claim(&self, claim_id: &Id) -> Result<Vec<Alignment>, StorageError>;
    async fn query_alignments(&self, filter: &QueryFilter) -> Result<QueryResult<Alignment>, StorageError>;
    async fn delete_alignment(&self, id: &Id) -> Result<bool, StorageError>;
    
    // Gap operations
    async fn store_gap(&self, gap: &Gap) -> Result<(), StorageError>;
    async fn get_gap(&self, id: &Id) -> Result<Option<Gap>, StorageError>;
    async fn query_gaps(&self, filter: &QueryFilter) -> Result<QueryResult<Gap>, StorageError>;
    async fn delete_gap(&self, id: &Id) -> Result<bool, StorageError>;
    
    // Batch operations for performance
    async fn store_artifacts_batch(&self, artifacts: &[Artifact]) -> Result<(), StorageError>;
    async fn store_claims_batch(&self, claims: &[Claim]) -> Result<(), StorageError>;
    async fn store_relationships_batch(&self, relationships: &[Relationship]) -> Result<(), StorageError>;
    async fn store_alignments_batch(&self, alignments: &[Alignment]) -> Result<(), StorageError>;
    
    // Analytics operations
    async fn get_project_health(&self, filter: &QueryFilter) -> Result<ProjectHealth, StorageError>;
    async fn get_artifact_stats(&self) -> Result<ArtifactStats, StorageError>;
}

/// Statistics about stored artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactStats {
    pub total_artifacts: usize,
    pub artifacts_by_type: HashMap<ArtifactType, usize>,
    pub total_claims: usize,
    pub claims_by_type: HashMap<ClaimType, usize>,
    pub total_relationships: usize,
    pub relationships_by_type: HashMap<RelationshipType, usize>,
    pub total_alignments: usize,
    pub average_alignment_score: f64,
    pub total_gaps: usize,
    pub gaps_by_severity: HashMap<Severity, usize>,
}

/// Configuration for different storage backends
#[derive(Debug, Clone)]
pub enum StorageConfig {
    /// In-memory storage for testing and development
    InMemory,
    /// Local SQLite database
    Sqlite { path: String },
    /// PostgreSQL database
    Postgres { connection_string: String },
    /// MongoDB database
    Mongo { connection_string: String, database: String },
    /// Custom storage backend
    Custom { config: HashMap<String, String> },
}

/// Artifact ingestion pipeline that processes different sources
#[async_trait]
pub trait ArtifactIngester: Send + Sync {
    /// Ingest a single artifact and return the stored artifact with ID
    async fn ingest_artifact(
        &self,
        content: String,
        artifact_type: ArtifactType,
        location: Location,
        metadata: HashMap<String, String>,
    ) -> Result<Artifact, IngestionError>;
    
    /// Ingest artifacts from a git repository
    async fn ingest_from_git(
        &self,
        repo_path: &str,
        branch: Option<String>,
        file_patterns: &[String],
    ) -> Result<IngestionResult, IngestionError>;
    
    /// Ingest artifacts from a directory tree
    async fn ingest_from_directory(
        &self,
        directory: &str,
        recursive: bool,
        file_patterns: &[String],
    ) -> Result<IngestionResult, IngestionError>;
    
    /// Ingest from external sources (JIRA, GitHub issues, etc.)
    async fn ingest_from_external(
        &self,
        source: ExternalSource,
        config: ExternalSourceConfig,
    ) -> Result<IngestionResult, IngestionError>;
}

/// Error types for ingestion operations
#[derive(thiserror::Error, Debug)]
pub enum IngestionError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Analysis error: {0}")]
    Analysis(#[from] AnalysisError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Git error: {0}")]
    Git(String),
    #[error("External source error: {0}")]
    ExternalSource(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Results from ingestion operations
#[derive(Debug, Clone)]
pub struct IngestionResult {
    pub artifacts_ingested: usize,
    pub claims_extracted: usize,
    pub relationships_discovered: usize,
    pub errors: Vec<String>,
    pub processing_time_ms: u64,
    pub ingested_artifact_ids: Vec<Id>,
}

/// External sources for artifact ingestion
#[derive(Debug, Clone)]
pub enum ExternalSource {
    /// JIRA tickets
    Jira { server_url: String },
    /// GitHub issues and PRs
    GitHub { owner: String, repo: String },
    /// Slack conversations
    Slack { workspace: String },
    /// Confluence pages
    Confluence { base_url: String },
    /// Custom external source
    Custom { source_type: String },
}

/// Configuration for external source ingestion
#[derive(Debug, Clone)]
pub struct ExternalSourceConfig {
    /// Authentication credentials
    pub credentials: HashMap<String, String>,
    /// Query or filter parameters
    pub query_params: HashMap<String, String>,
    /// Date range for ingestion
    pub date_range: Option<(Timestamp, Timestamp)>,
    /// Whether to include historical data
    pub include_historical: bool,
}

/// File type detection for automatic artifact type assignment
pub struct FileTypeDetector {
    /// Mapping from file extensions to artifact types
    extension_mappings: HashMap<String, ArtifactType>,
    /// Patterns for detecting test files
    test_patterns: Vec<String>,
    /// Patterns for detecting documentation
    doc_patterns: Vec<String>,
}

impl Default for FileTypeDetector {
    fn default() -> Self {
        let mut extension_mappings = HashMap::new();
        
        // Code files
        extension_mappings.insert("rs".to_string(), ArtifactType::Code);
        extension_mappings.insert("py".to_string(), ArtifactType::Code);
        extension_mappings.insert("js".to_string(), ArtifactType::Code);
        extension_mappings.insert("ts".to_string(), ArtifactType::Code);
        extension_mappings.insert("java".to_string(), ArtifactType::Code);
        extension_mappings.insert("cpp".to_string(), ArtifactType::Code);
        extension_mappings.insert("c".to_string(), ArtifactType::Code);
        extension_mappings.insert("go".to_string(), ArtifactType::Code);
        
        // Documentation files
        extension_mappings.insert("md".to_string(), ArtifactType::Documentation);
        extension_mappings.insert("rst".to_string(), ArtifactType::Documentation);
        extension_mappings.insert("txt".to_string(), ArtifactType::Documentation);
        extension_mappings.insert("adoc".to_string(), ArtifactType::Documentation);
        
        Self {
            extension_mappings,
            test_patterns: vec![
                "test_*.rs".to_string(),
                "*_test.py".to_string(),
                "*.test.js".to_string(),
                "*.spec.ts".to_string(),
                "test/*.rs".to_string(),
                "tests/*.py".to_string(),
                "__tests__/*.js".to_string(),
            ],
            doc_patterns: vec![
                "README.*".to_string(),
                "CHANGELOG.*".to_string(),
                "docs/*".to_string(),
                "documentation/*".to_string(),
                "*.api.md".to_string(),
            ],
        }
    }
}

impl FileTypeDetector {
    /// Detect artifact type from file path and content
    pub fn detect_type(&self, file_path: &str, content: &str) -> ArtifactType {
        // Check test patterns first
        for pattern in &self.test_patterns {
            if self.matches_pattern(file_path, pattern) {
                return ArtifactType::Test;
            }
        }
        
        // Check doc patterns
        for pattern in &self.doc_patterns {
            if self.matches_pattern(file_path, pattern) {
                return ArtifactType::Documentation;
            }
        }
        
        // Check by extension
        if let Some(extension) = std::path::Path::new(file_path).extension() {
            if let Some(ext_str) = extension.to_str() {
                if let Some(artifact_type) = self.extension_mappings.get(ext_str) {
                    return artifact_type.clone();
                }
            }
        }
        
        // Content-based detection as fallback
        self.detect_from_content(content)
    }
    
    fn matches_pattern(&self, file_path: &str, pattern: &str) -> bool {
        // Simple pattern matching - in practice you'd use a proper glob library
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                file_path.starts_with(parts[0]) && file_path.ends_with(parts[1])
            } else {
                false
            }
        } else {
            file_path == pattern
        }
    }
    
    fn detect_from_content(&self, content: &str) -> ArtifactType {
        let content_lower = content.to_lowercase();
        
        // Look for test indicators
        if content_lower.contains("test") || 
           content_lower.contains("assert") ||
           content_lower.contains("expect") ||
           content_lower.contains("should") {
            return ArtifactType::Test;
        }
        
        // Look for documentation indicators
        if content_lower.contains("# ") || // Markdown headers
           content_lower.contains("## ") ||
           content_lower.contains("api") ||
           content_lower.contains("documentation") {
            return ArtifactType::Documentation;
        }
        
        // Default to code
        ArtifactType::Code
    }
}

/// Caching layer for storage operations
#[async_trait]
pub trait StorageCache: Send + Sync {
    /// Get cached item
    async fn get<T>(&self, key: &str) -> Result<Option<T>, StorageError>
    where
        T: for<'de> Deserialize<'de> + Send;
    
    /// Store item in cache
    async fn set<T>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<(), StorageError>
    where
        T: Serialize + Send + Sync;
    
    /// Remove item from cache
    async fn remove(&self, key: &str) -> Result<bool, StorageError>;
    
    /// Clear all cached items
    async fn clear(&self) -> Result<(), StorageError>;
}

/// Storage backend that combines primary storage with caching
pub struct CachedStorage<S: SatsStorage, C: StorageCache> {
    storage: S,
    cache: C,
    cache_ttl_seconds: u64,
}

impl<S: SatsStorage, C: StorageCache> CachedStorage<S, C> {
    pub fn new(storage: S, cache: C, cache_ttl_seconds: u64) -> Self {
        Self {
            storage,
            cache,
            cache_ttl_seconds,
        }
    }
    
    async fn cache_key_for_artifact(&self, id: &Id) -> String {
        format!("artifact:{}", id)
    }
    
    async fn cache_key_for_query(&self, filter: &QueryFilter) -> String {
        // In practice, you'd hash the filter parameters
        format!("query:{:?}", filter)
    }
}

#[async_trait]
impl<S: SatsStorage, C: StorageCache> SatsStorage for CachedStorage<S, C> {
    async fn store_artifact(&self, artifact: &Artifact) -> Result<(), StorageError> {
        let result = self.storage.store_artifact(artifact).await;
        if result.is_ok() {
            let cache_key = self.cache_key_for_artifact(&artifact.id).await;
            let _ = self.cache.set(&cache_key, artifact, Some(self.cache_ttl_seconds)).await;
        }
        result
    }
    
    async fn get_artifact(&self, id: &Id) -> Result<Option<Artifact>, StorageError> {
        let cache_key = self.cache_key_for_artifact(id).await;
        
        // Try cache first
        if let Ok(Some(cached)) = self.cache.get::<Artifact>(&cache_key).await {
            return Ok(Some(cached));
        }
        
        // Fall back to storage
        let result = self.storage.get_artifact(id).await;
        if let Ok(Some(ref artifact)) = result {
            let _ = self.cache.set(&cache_key, artifact, Some(self.cache_ttl_seconds)).await;
        }
        
        result
    }
    
    // Implement other methods by delegating to storage and updating cache...
    // For brevity, showing just the pattern here
    
    async fn query_artifacts(&self, filter: &QueryFilter) -> Result<QueryResult<Artifact>, StorageError> {
        self.storage.query_artifacts(filter).await
    }
    
    async fn update_artifact(&self, artifact: &Artifact) -> Result<(), StorageError> {
        let result = self.storage.update_artifact(artifact).await;
        if result.is_ok() {
            let cache_key = self.cache_key_for_artifact(&artifact.id).await;
            let _ = self.cache.remove(&cache_key).await; // Invalidate cache
        }
        result
    }
    
    async fn delete_artifact(&self, id: &Id) -> Result<bool, StorageError> {
        let result = self.storage.delete_artifact(id).await;
        if result.is_ok() {
            let cache_key = self.cache_key_for_artifact(id).await;
            let _ = self.cache.remove(&cache_key).await;
        }
        result
    }
    
    // ... implement remaining methods with similar cache-then-storage pattern
    async fn store_claim(&self, claim: &Claim) -> Result<(), StorageError> { self.storage.store_claim(claim).await }
    async fn get_claim(&self, id: &Id) -> Result<Option<Claim>, StorageError> { self.storage.get_claim(id).await }
    async fn get_claims_for_artifact(&self, artifact_id: &Id) -> Result<Vec<Claim>, StorageError> { self.storage.get_claims_for_artifact(artifact_id).await }
    async fn query_claims(&self, filter: &QueryFilter) -> Result<QueryResult<Claim>, StorageError> { self.storage.query_claims(filter).await }
    async fn delete_claim(&self, id: &Id) -> Result<bool, StorageError> { self.storage.delete_claim(id).await }
    async fn store_relationship(&self, relationship: &Relationship) -> Result<(), StorageError> { self.storage.store_relationship(relationship).await }
    async fn get_relationship(&self, id: &Id) -> Result<Option<Relationship>, StorageError> { self.storage.get_relationship(id).await }
    async fn get_relationships_for_artifact(&self, artifact_id: &Id) -> Result<Vec<Relationship>, StorageError> { self.storage.get_relationships_for_artifact(artifact_id).await }
    async fn query_relationships(&self, filter: &QueryFilter) -> Result<QueryResult<Relationship>, StorageError> { self.storage.query_relationships(filter).await }
    async fn delete_relationship(&self, id: &Id) -> Result<bool, StorageError> { self.storage.delete_relationship(id).await }
    async fn store_alignment(&self, alignment: &Alignment) -> Result<(), StorageError> { self.storage.store_alignment(alignment).await }
    async fn get_alignment(&self, id: &Id) -> Result<Option<Alignment>, StorageError> { self.storage.get_alignment(id).await }
    async fn get_alignments_for_claim(&self, claim_id: &Id) -> Result<Vec<Alignment>, StorageError> { self.storage.get_alignments_for_claim(claim_id).await }
    async fn query_alignments(&self, filter: &QueryFilter) -> Result<QueryResult<Alignment>, StorageError> { self.storage.query_alignments(filter).await }
    async fn delete_alignment(&self, id: &Id) -> Result<bool, StorageError> { self.storage.delete_alignment(id).await }
    async fn store_gap(&self, gap: &Gap) -> Result<(), StorageError> { self.storage.store_gap(gap).await }
    async fn get_gap(&self, id: &Id) -> Result<Option<Gap>, StorageError> { self.storage.get_gap(id).await }
    async fn query_gaps(&self, filter: &QueryFilter) -> Result<QueryResult<Gap>, StorageError> { self.storage.query_gaps(filter).await }
    async fn delete_gap(&self, id: &Id) -> Result<bool, StorageError> { self.storage.delete_gap(id).await }
    async fn store_artifacts_batch(&self, artifacts: &[Artifact]) -> Result<(), StorageError> { self.storage.store_artifacts_batch(artifacts).await }
    async fn store_claims_batch(&self, claims: &[Claim]) -> Result<(), StorageError> { self.storage.store_claims_batch(claims).await }
    async fn store_relationships_batch(&self, relationships: &[Relationship]) -> Result<(), StorageError> { self.storage.store_relationships_batch(relationships).await }
    async fn store_alignments_batch(&self, alignments: &[Alignment]) -> Result<(), StorageError> { self.storage.store_alignments_batch(alignments).await }
    async fn get_project_health(&self, filter: &QueryFilter) -> Result<ProjectHealth, StorageError> { self.storage.get_project_health(filter).await }
    async fn get_artifact_stats(&self) -> Result<ArtifactStats, StorageError> { self.storage.get_artifact_stats().await }
}