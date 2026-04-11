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

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetLlmConfigInput {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
}

impl SetLlmConfigInput {
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
    use super::SetLlmConfigInput;
    use crate::domain::error::DomainError;

    fn valid_input() -> SetLlmConfigInput {
        SetLlmConfigInput {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: "sk-test".to_string(),
            model: "gpt-4o-mini".to_string(),
        }
    }

    #[test]
    fn validates_valid_input() {
        assert!(valid_input().validate().is_ok());
    }

    #[test]
    fn rejects_empty_endpoint() {
        let mut input = valid_input();
        input.endpoint = "  ".to_string();
        assert!(
            matches!(input.validate(), Err(DomainError::ValidationFailed { field }) if field == "endpoint")
        );
    }

    #[test]
    fn rejects_empty_api_key() {
        let mut input = valid_input();
        input.api_key = "".to_string();
        assert!(
            matches!(input.validate(), Err(DomainError::ValidationFailed { field }) if field == "apiKey")
        );
    }

    #[test]
    fn rejects_empty_model() {
        let mut input = valid_input();
        input.model = "   ".to_string();
        assert!(
            matches!(input.validate(), Err(DomainError::ValidationFailed { field }) if field == "model")
        );
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LlmStreamEvent {
    TokenChunk { text: String },
    Completion { full_text: String },
    Error {
        error_code: String,
        message: String,
        retryable: bool,
    },
}
