use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum StoryGenerationError {
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Node not found: {node_id}")]
    NodeNotFound { node_id: Uuid },

    #[error("Choice not found: {choice_id}")]
    ChoiceNotFound { choice_id: Uuid },

    #[error("Graph validation failed: {issues:?}")]
    GraphValidationFailed { issues: Vec<String> },

    #[error("AI service error: {message}")]
    AiServiceError { message: String },

    #[error("Coherence validation failed: {score} below threshold {threshold}")]
    CoherenceValidationFailed { score: f32, threshold: f32 },

    #[error("Maximum graph depth exceeded: {current_depth} > {max_depth}")]
    MaxDepthExceeded { current_depth: u32, max_depth: u32 },

    #[error("Too many choices: {choice_count} > {max_choices}")]
    TooManyChoices { choice_count: u32, max_choices: u32 },

    #[error("Circular dependency detected in nodes: {node_ids:?}")]
    CircularDependency { node_ids: Vec<Uuid> },

    #[error("Question validation failed: {field} - {message}")]
    QuestionValidationError { field: String, message: String },

    #[error("Context insufficient for generation: missing {missing_fields:?}")]
    InsufficientContext { missing_fields: Vec<String> },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Timeout occurred during operation: {operation}")]
    Timeout { operation: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },
}

impl From<serde_json::Error> for StoryGenerationError {
    fn from(err: serde_json::Error) -> Self {
        StoryGenerationError::SerializationError {
            message: err.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, StoryGenerationError>;