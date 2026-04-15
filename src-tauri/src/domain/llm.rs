use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::error::DomainError;

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
        }
    }
}

impl LlmConfig {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.endpoint.trim().is_empty() {
            return Err(DomainError::ValidationFailed {
                field: "endpoint".to_string(),
            });
        }
        if self.api_key.trim().is_empty() {
            return Err(DomainError::ValidationFailed {
                field: "apiKey".to_string(),
            });
        }
        if self.model.trim().is_empty() {
            return Err(DomainError::ValidationFailed {
                field: "model".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::LlmConfig;
    use crate::domain::error::DomainError;

    fn valid_config() -> LlmConfig {
        LlmConfig {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: "sk-test".to_string(),
            model: "gpt-4o-mini".to_string(),
        }
    }

    #[test]
    fn validates_valid_config() {
        assert!(valid_config().validate().is_ok());
    }

    #[test]
    fn rejects_empty_endpoint() {
        let mut config = valid_config();
        config.endpoint = "  ".to_string();
        assert!(
            matches!(config.validate(), Err(DomainError::ValidationFailed { field }) if field == "endpoint")
        );
    }

    #[test]
    fn rejects_empty_api_key() {
        let mut config = valid_config();
        config.api_key = "".to_string();
        assert!(
            matches!(config.validate(), Err(DomainError::ValidationFailed { field }) if field == "apiKey")
        );
    }

    #[test]
    fn rejects_empty_model() {
        let mut config = valid_config();
        config.model = "   ".to_string();
        assert!(
            matches!(config.validate(), Err(DomainError::ValidationFailed { field }) if field == "model")
        );
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LlmStreamEvent {
    TokenChunk {
        text: String,
    },
    #[serde(rename_all = "camelCase")]
    Completion {
        full_text: String,
    },
    #[serde(rename_all = "camelCase")]
    Error {
        error_code: String,
        message: String,
        retryable: bool,
    },
}
