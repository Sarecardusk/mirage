use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation failed: {field}")]
    ValidationFailed { field: String },

    #[error("theme card not found: {id}")]
    ThemeCardNotFound { id: String },

    #[error("session not found: {id}")]
    SessionNotFound { id: String },

    /// 存储层失败，例如 SurrealDB 的读写异常。
    ///
    /// 领域仓储接口统一返回 `DomainError`，
    /// 这样 `command` 层就不需要直接依赖 `infra` 层的具体错误类型。
    /// 这个分支专门用来跨层传递底层存储异常。
    #[error("storage failed: {message}")]
    StorageFailed { message: String },
}
