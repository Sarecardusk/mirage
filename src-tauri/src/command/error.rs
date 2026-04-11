use serde::Serialize;
use specta::Type;

use crate::domain::error::DomainError;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    pub category: ErrorCategory,
    pub error_code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[allow(dead_code)]
pub enum ErrorCategory {
    Domain,
    Infra,
    Security,
    Gateway,
}

impl IpcError {
    pub fn gateway(message: impl Into<String>, retryable: bool) -> Self {
        Self {
            category: ErrorCategory::Gateway,
            error_code: "GATEWAY_UPSTREAM_ERROR".to_string(),
            message: message.into(),
            retryable,
        }
    }
}

impl From<DomainError> for IpcError {
    fn from(error: DomainError) -> Self {
        match error {
            DomainError::ValidationFailed { field } => Self {
                category: ErrorCategory::Domain,
                error_code: "VALIDATION_FAILED".to_string(),
                message: format!("validation failed: {field}"),
                retryable: false,
            },
            DomainError::ThemeCardNotFound { id } => Self {
                category: ErrorCategory::Domain,
                error_code: "ENTITY_NOT_FOUND".to_string(),
                message: format!("theme card not found: {id}"),
                retryable: false,
            },
            DomainError::SessionNotFound { id } => Self {
                category: ErrorCategory::Domain,
                error_code: "ENTITY_NOT_FOUND".to_string(),
                message: format!("session not found: {id}"),
                retryable: false,
            },
        }
    }
}
