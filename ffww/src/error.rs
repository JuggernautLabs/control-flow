
#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("Query resolver error: {0}")]
    QueryError(#[from] client_implementations::error::QueryResolverError),

    #[error("claude client error: {0}")]
    ClaudeError(#[from] client_implementations::error::AIError),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
