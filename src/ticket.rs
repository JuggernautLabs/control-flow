use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub terms_needing_refinement: Vec<String>,
    #[serde(rename = "openQuestions")]
    pub open_questions: Vec<String>,
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