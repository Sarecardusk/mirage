use serde::Serialize;
use specta::Type;

use crate::domain::error::DomainError;
use crate::infra::vault::VaultError;

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

impl From<VaultError> for IpcError {
    fn from(error: VaultError) -> Self {
        match error {
            VaultError::KeyMissing => Self {
                category: ErrorCategory::Security,
                error_code: "VAULT_KEY_MISSING".to_string(),
                message: "vault key file is missing while encrypted data exists".to_string(),
                retryable: false,
            },
            VaultError::DecryptFailed => Self {
                category: ErrorCategory::Security,
                error_code: "VAULT_DECRYPT_FAILED".to_string(),
                message: "vault decrypt failed".to_string(),
                retryable: false,
            },
            VaultError::WriteFailed(inner) => Self {
                category: ErrorCategory::Infra,
                error_code: "VAULT_WRITE_FAILED".to_string(),
                message: inner.to_string(),
                retryable: true,
            },
            VaultError::Corrupted(message) => Self {
                category: ErrorCategory::Infra,
                error_code: "VAULT_CORRUPTED".to_string(),
                message: message.to_string(),
                retryable: false,
            },
            VaultError::RandomFailed => Self {
                category: ErrorCategory::Infra,
                error_code: "VAULT_WRITE_FAILED".to_string(),
                message: "vault random source failed".to_string(),
                retryable: true,
            },
            VaultError::CryptoFailed(message) => Self {
                category: ErrorCategory::Infra,
                error_code: "VAULT_WRITE_FAILED".to_string(),
                message: format!("vault crypto failed: {message}"),
                retryable: false,
            },
            VaultError::Serde(inner) => Self {
                category: ErrorCategory::Infra,
                error_code: "VAULT_CORRUPTED".to_string(),
                message: inner.to_string(),
                retryable: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorCategory, IpcError};
    use crate::infra::vault::VaultError;

    #[test]
    fn maps_security_vault_errors() {
        let missing = IpcError::from(VaultError::KeyMissing);
        assert!(matches!(missing.category, ErrorCategory::Security));
        assert_eq!(missing.error_code, "VAULT_KEY_MISSING");
        assert!(!missing.retryable);

        let decrypt = IpcError::from(VaultError::DecryptFailed);
        assert!(matches!(decrypt.category, ErrorCategory::Security));
        assert_eq!(decrypt.error_code, "VAULT_DECRYPT_FAILED");
        assert!(!decrypt.retryable);
    }

    #[test]
    fn maps_infra_vault_errors() {
        let write = IpcError::from(VaultError::WriteFailed(std::io::Error::other("disk full")));
        assert!(matches!(write.category, ErrorCategory::Infra));
        assert_eq!(write.error_code, "VAULT_WRITE_FAILED");
        assert!(write.retryable);

        let corrupted = IpcError::from(VaultError::Corrupted("bad nonce"));
        assert!(matches!(corrupted.category, ErrorCategory::Infra));
        assert_eq!(corrupted.error_code, "VAULT_CORRUPTED");
        assert!(!corrupted.retryable);

        let random = IpcError::from(VaultError::RandomFailed);
        assert!(matches!(random.category, ErrorCategory::Infra));
        assert_eq!(random.error_code, "VAULT_WRITE_FAILED");
        assert!(random.retryable);

        let crypto = IpcError::from(VaultError::CryptoFailed("encryption failed"));
        assert!(matches!(crypto.category, ErrorCategory::Infra));
        assert_eq!(crypto.error_code, "VAULT_WRITE_FAILED");
        assert!(!crypto.retryable);
        assert_eq!(crypto.message, "vault crypto failed: encryption failed");
    }
}
