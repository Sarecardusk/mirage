use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation failed: {field}")]
    ValidationFailed { field: String },

    #[error("theme card not found: {id}")]
    ThemeCardNotFound { id: String },

    #[error("session not found: {id}")]
    SessionNotFound { id: String },
}
