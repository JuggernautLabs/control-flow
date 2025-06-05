use crate::types::*;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Sophisticated alignment scoring that considers multiple dimensions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignmentScore {
    /// Overall alignment score (0.0 to 1.0)
    pub overall: Confidence,
    /// Breakdown by different alignment dimensions
    pub dimensions: AlignmentDimensions,
    /// Explanation of the scoring rationale
    pub explanation: String,
    /// Key evidence points that support or contradict the claim
    pub evidence_points: Vec<EvidencePoint>,
}

/// Different dimensions of alignment between claims and evidence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignmentDimensions {
    /// Direct semantic alignment (does evidence directly support claim?)
    pub semantic_alignment: Confidence,
    /// Functional alignment (does evidence implement claimed functionality?)
    pub functional_alignment: Confidence,
    /// Behavioral alignment (does evidence exhibit claimed behavior?)
    pub behavioral_alignment: Confidence,
    /// Structural alignment (does evidence follow claimed structure?)
    pub structural_alignment: Confidence,
    /// Temporal alignment (is evidence current with claim?)
    pub temporal_alignment: Confidence,
}

/// A specific piece of evidence that supports or contradicts a claim
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidencePoint {
    /// The specific excerpt from the evidence artifact
    pub excerpt: String,
    /// Whether this supports or contradicts the claim
    pub evidence_type: EvidenceType,
    /// Strength of this evidence point
    pub strength: Confidence,
    /// Location within the evidence artifact
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Evidence directly supports the claim
    Supporting,
    /// Evidence partially supports the claim
    PartiallySupporting,
    /// Evidence contradicts the claim
    Contradicting,
    /// Evidence is neutral/irrelevant to the claim
    Neutral,
    /// Evidence suggests the claim is outdated
    Outdated,
}

/// Model for tracking relationships between artifacts over time
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationshipGraph {
    /// All relationships in the system
    pub relationships: Vec<Relationship>,
    /// Cached relationship lookup by artifact ID
    pub artifact_relationships: HashMap<Id, Vec<Id>>,
    /// Transitive relationship chains (e.g., requirement -> code -> test)
    pub relationship_chains: Vec<RelationshipChain>,
    /// Metrics about the relationship graph
    pub graph_metrics: GraphMetrics,
}

/// A chain of relationships showing traceability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationshipChain {
    /// Ordered list of artifact IDs in the chain
    pub artifact_chain: Vec<Id>,
    /// Relationship types between each pair
    pub relationship_types: Vec<RelationshipType>,
    /// Overall confidence in this chain
    pub chain_confidence: Confidence,
    /// Description of what this chain represents
    pub description: String,
}

/// Metrics about the relationship graph structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphMetrics {
    /// Total number of artifacts
    pub total_artifacts: usize,
    /// Total number of relationships
    pub total_relationships: usize,
    /// Number of isolated artifacts (no relationships)
    pub isolated_artifacts: usize,
    /// Average number of relationships per artifact
    pub avg_relationships_per_artifact: f64,
    /// Relationship type distribution
    pub relationship_type_counts: HashMap<RelationshipType, usize>,
    /// Longest relationship chain
    pub max_chain_length: usize,
}

/// Advanced alignment checker that considers multiple evidence sources
pub struct MultiEvidenceAlignmentChecker {
    /// Minimum number of evidence sources required for high confidence
    pub min_evidence_sources: usize,
    /// Weight given to different types of evidence
    pub evidence_weights: HashMap<ArtifactType, f64>,
    /// Threshold for considering evidence as supporting
    pub support_threshold: Confidence,
}

impl Default for MultiEvidenceAlignmentChecker {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert(ArtifactType::Test, 1.0);           // Tests are strong evidence
        weights.insert(ArtifactType::Code, 0.9);           // Code is direct evidence
        weights.insert(ArtifactType::Documentation, 0.7);  // Docs can be outdated
        weights.insert(ArtifactType::Commit, 0.6);         // Commits show intent
        weights.insert(ArtifactType::Comment, 0.5);        // Comments can be stale
        
        Self {
            min_evidence_sources: 2,
            evidence_weights: weights,
            support_threshold: Confidence::new(0.5).unwrap(),
        }
    }
}

impl MultiEvidenceAlignmentChecker {
    /// Calculate alignment considering multiple evidence sources
    pub fn calculate_multi_evidence_alignment(
        &self,
        claim: &Claim,
        evidence_artifacts: &[Artifact],
        individual_alignments: &[Alignment],
    ) -> AlignmentScore {
        let mut total_weighted_score = 0.0;
        let mut total_weight = 0.0;
        let mut evidence_points = Vec::new();
        
        for alignment in individual_alignments {
            if let Some(artifact) = evidence_artifacts.iter()
                .find(|a| a.id == alignment.evidence_artifact_id) {
                
                let weight = self.evidence_weights.get(&artifact.artifact_type)
                    .copied().unwrap_or(0.5);
                
                total_weighted_score += alignment.alignment_score.value() * weight;
                total_weight += weight;
                
                // Extract evidence points from the alignment
                evidence_points.push(EvidencePoint {
                    excerpt: alignment.explanation.chars().take(200).collect(),
                    evidence_type: self.classify_evidence_type(alignment.alignment_score),
                    strength: alignment.alignment_score,
                    location: Some(artifact.location.display()),
                });
            }
        }
        
        let overall_score = if total_weight > 0.0 {
            total_weighted_score / total_weight
        } else {
            0.0
        };
        
        let confidence_adjustment = self.calculate_confidence_adjustment(
            evidence_artifacts.len(),
            individual_alignments.len()
        );
        
        let adjusted_score = overall_score * confidence_adjustment;
        
        AlignmentScore {
            overall: Confidence::new(adjusted_score.clamp(0.0, 1.0)).unwrap(),
            dimensions: self.calculate_alignment_dimensions(individual_alignments, evidence_artifacts),
            explanation: self.generate_alignment_explanation(claim, &evidence_points, adjusted_score),
            evidence_points,
        }
    }
    
    fn classify_evidence_type(&self, score: Confidence) -> EvidenceType {
        match score.value() {
            s if s >= 0.8 => EvidenceType::Supporting,
            s if s >= 0.5 => EvidenceType::PartiallySupporting,
            s if s >= 0.3 => EvidenceType::Neutral,
            _ => EvidenceType::Contradicting,
        }
    }
    
    fn calculate_confidence_adjustment(&self, total_evidence: usize, aligned_evidence: usize) -> f64 {
        if total_evidence == 0 {
            return 0.0;
        }
        
        // Boost confidence if we have multiple evidence sources
        let evidence_diversity_boost = if total_evidence >= self.min_evidence_sources {
            1.0 + (total_evidence as f64 - self.min_evidence_sources as f64) * 0.05
        } else {
            0.8 // Penalize if we have too few evidence sources
        };
        
        // Consider the ratio of aligned evidence
        let alignment_ratio = aligned_evidence as f64 / total_evidence as f64;
        
        (evidence_diversity_boost * alignment_ratio).min(1.2) // Cap the boost
    }
    
    fn calculate_alignment_dimensions(
        &self,
        alignments: &[Alignment],
        _evidence_artifacts: &[Artifact],
    ) -> AlignmentDimensions {
        // This is simplified - in practice, you'd analyze the content more deeply
        let avg_score = alignments.iter()
            .map(|a| a.alignment_score.value())
            .sum::<f64>() / alignments.len().max(1) as f64;
        
        let base_confidence = Confidence::new(avg_score).unwrap();
        
        AlignmentDimensions {
            semantic_alignment: base_confidence,
            functional_alignment: base_confidence,
            behavioral_alignment: base_confidence,
            structural_alignment: base_confidence,
            temporal_alignment: base_confidence,
        }
    }
    
    fn generate_alignment_explanation(
        &self,
        claim: &Claim,
        evidence_points: &[EvidencePoint],
        final_score: f64,
    ) -> String {
        let supporting_count = evidence_points.iter()
            .filter(|ep| matches!(ep.evidence_type, EvidenceType::Supporting | EvidenceType::PartiallySupporting))
            .count();
        
        let contradicting_count = evidence_points.iter()
            .filter(|ep| matches!(ep.evidence_type, EvidenceType::Contradicting))
            .count();
        
        format!(
            "Claim '{}' has {} supporting evidence points and {} contradictory points. \
            Overall alignment score: {:.2}. {}",
            claim.statement.chars().take(50).collect::<String>(),
            supporting_count,
            contradicting_count,
            final_score,
            if final_score >= 0.7 {
                "Strong evidence support."
            } else if final_score >= 0.4 {
                "Moderate evidence support."
            } else {
                "Weak or contradictory evidence."
            }
        )
    }
}

/// Tracker for relationship evolution over time
pub struct RelationshipEvolutionTracker {
    /// Historical snapshots of relationships
    snapshots: Vec<RelationshipSnapshot>,
    /// Configuration for tracking
    config: EvolutionConfig,
}

#[derive(Debug, Clone)]
pub struct RelationshipSnapshot {
    pub timestamp: Timestamp,
    pub relationships: Vec<Relationship>,
    pub trigger: SnapshotTrigger,
}

#[derive(Debug, Clone)]
pub enum SnapshotTrigger {
    /// Regular periodic snapshot
    Periodic,
    /// Triggered by a new commit
    Commit(String),
    /// Triggered by artifact changes
    ArtifactChange(Id),
    /// Manual snapshot
    Manual,
}

#[derive(Debug, Clone)]
pub struct EvolutionConfig {
    /// How often to take periodic snapshots
    pub snapshot_frequency: chrono::Duration,
    /// Maximum number of snapshots to keep
    pub max_snapshots: usize,
    /// Whether to track relationship strength changes
    pub track_strength_changes: bool,
}

impl RelationshipEvolutionTracker {
    pub fn new(config: EvolutionConfig) -> Self {
        Self {
            snapshots: Vec::new(),
            config,
        }
    }
    
    /// Add a new snapshot of relationships
    pub fn add_snapshot(
        &mut self,
        relationships: Vec<Relationship>,
        trigger: SnapshotTrigger,
    ) {
        let snapshot = RelationshipSnapshot {
            timestamp: chrono::Utc::now(),
            relationships,
            trigger,
        };
        
        self.snapshots.push(snapshot);
        
        // Trim old snapshots if needed
        if self.snapshots.len() > self.config.max_snapshots {
            self.snapshots.remove(0);
        }
    }
    
    /// Analyze how relationships have changed over time
    pub fn analyze_relationship_drift(&self) -> RelationshipDriftAnalysis {
        if self.snapshots.len() < 2 {
            return RelationshipDriftAnalysis::default();
        }
        
        let latest = self.snapshots.last().unwrap();
        let previous = &self.snapshots[self.snapshots.len() - 2];
        
        let mut added_relationships = Vec::new();
        let mut removed_relationships = Vec::new();
        let mut strength_changes = Vec::new();
        
        let previous_ids: HashSet<_> = previous.relationships.iter()
            .map(|r| (r.source_artifact_id, r.target_artifact_id, &r.relationship_type))
            .collect();
        
        let latest_map: HashMap<_, _> = latest.relationships.iter()
            .map(|r| ((r.source_artifact_id, r.target_artifact_id, &r.relationship_type), r))
            .collect();
        
        // Find added relationships
        for relationship in &latest.relationships {
            let key = (relationship.source_artifact_id, relationship.target_artifact_id, &relationship.relationship_type);
            if !previous_ids.contains(&key) {
                added_relationships.push(relationship.clone());
            }
        }
        
        // Find removed relationships and strength changes
        for prev_relationship in &previous.relationships {
            let key = (prev_relationship.source_artifact_id, prev_relationship.target_artifact_id, &prev_relationship.relationship_type);
            if let Some(current_relationship) = latest_map.get(&key) {
                // Check for strength changes
                if self.config.track_strength_changes {
                    let strength_diff = current_relationship.confidence.value() - prev_relationship.confidence.value();
                    if strength_diff.abs() > 0.1 {
                        strength_changes.push(StrengthChange {
                            relationship: (*current_relationship).clone(),
                            previous_strength: prev_relationship.confidence,
                            current_strength: current_relationship.confidence,
                            change: strength_diff,
                        });
                    }
                }
            } else {
                removed_relationships.push(prev_relationship.clone());
            }
        }
        
        RelationshipDriftAnalysis {
            added_relationships,
            removed_relationships,
            strength_changes,
            analysis_period: latest.timestamp - previous.timestamp,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RelationshipDriftAnalysis {
    pub added_relationships: Vec<Relationship>,
    pub removed_relationships: Vec<Relationship>,
    pub strength_changes: Vec<StrengthChange>,
    pub analysis_period: chrono::Duration,
}

#[derive(Debug, Clone)]
pub struct StrengthChange {
    pub relationship: Relationship,
    pub previous_strength: Confidence,
    pub current_strength: Confidence,
    pub change: f64,
}