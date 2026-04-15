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

    /// 启动流程尚未结束时，业务命令统一返回这个错误
    ///（也就是 `AppState::ready == false` 的阶段）。
    pub fn app_not_ready() -> Self {
        Self {
            category: ErrorCategory::Infra,
            error_code: "APP_NOT_READY".to_string(),
            message: "application is still starting up".to_string(),
            retryable: true,
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
            DomainError::StorageFailed { message } => Self {
                category: ErrorCategory::Infra,
                error_code: "STORAGE_WRITE_FAILED".to_string(),
                message,
                retryable: true,
            },
        }
    }
}
