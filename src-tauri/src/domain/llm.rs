use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::error::DomainError;

pub const DEFAULT_LLM_API_KEY_REF: &str = "llm_api_key";

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    pub endpoint: String,
    pub api_key_ref: String,
    pub model: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetLlmConfigInput {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ListLlmModelsInput {
    pub endpoint: String,
    pub api_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TestLlmConnectionInput {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            // DeepSeek 官方推荐 base_url 为 https://api.deepseek.com（也兼容 /v1）。
            endpoint: "https://api.deepseek.com".to_string(),
            api_key_ref: DEFAULT_LLM_API_KEY_REF.to_string(),
            model: "deepseek-chat".to_string(),
            temperature: None,
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        }
    }
}

fn validate_non_empty(value: &str, field: &str) -> Result<(), DomainError> {
    if value.trim().is_empty() {
        return Err(DomainError::ValidationFailed {
            field: field.to_string(),
        });
    }
    Ok(())
}

fn validate_f64_range(
    value: Option<f64>,
    field: &str,
    min: f64,
    max: f64,
) -> Result<(), DomainError> {
    if let Some(v) = value {
        if !(min..=max).contains(&v) {
            return Err(DomainError::ValidationFailed {
                field: field.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_generation_params(
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    top_p: Option<f64>,
    frequency_penalty: Option<f64>,
    presence_penalty: Option<f64>,
) -> Result<(), DomainError> {
    validate_f64_range(temperature, "temperature", 0.0, 2.0)?;
    validate_f64_range(top_p, "topP", 0.0, 1.0)?;
    validate_f64_range(frequency_penalty, "frequencyPenalty", -2.0, 2.0)?;
    validate_f64_range(presence_penalty, "presencePenalty", -2.0, 2.0)?;
    if let Some(mt) = max_tokens {
        if mt == 0 {
            return Err(DomainError::ValidationFailed {
                field: "maxTokens".to_string(),
            });
        }
    }
    Ok(())
}

impl SetLlmConfigInput {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_non_empty(self.endpoint.as_str(), "endpoint")?;
        validate_non_empty(self.api_key.as_str(), "apiKey")?;
        validate_non_empty(self.model.as_str(), "model")?;
        validate_generation_params(
            self.temperature,
            self.max_tokens,
            self.top_p,
            self.frequency_penalty,
            self.presence_penalty,
        )
    }
}

impl ListLlmModelsInput {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_non_empty(self.endpoint.as_str(), "endpoint")?;
        validate_non_empty(self.api_key.as_str(), "apiKey")
    }
}

impl TestLlmConnectionInput {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_non_empty(self.endpoint.as_str(), "endpoint")?;
        validate_non_empty(self.api_key.as_str(), "apiKey")?;
        validate_non_empty(self.model.as_str(), "model")
    }
}

#[cfg(test)]
mod tests {
    use super::{ListLlmModelsInput, SetLlmConfigInput, TestLlmConnectionInput};
    use crate::domain::error::DomainError;

    fn valid_set_input() -> SetLlmConfigInput {
        SetLlmConfigInput {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key: "sk-test".to_string(),
            model: "deepseek-chat".to_string(),
            temperature: None,
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        }
    }

    #[test]
    fn set_input_accepts_generation_params_within_range() {
        let mid = SetLlmConfigInput {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key: "sk-test".to_string(),
            model: "deepseek-chat".to_string(),
            temperature: Some(1.0),
            max_tokens: Some(512),
            top_p: Some(0.9),
            frequency_penalty: Some(-1.5),
            presence_penalty: Some(1.5),
        };
        assert!(mid.validate().is_ok());

        let boundary = SetLlmConfigInput {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key: "sk-test".to_string(),
            model: "deepseek-chat".to_string(),
            temperature: Some(0.0),
            max_tokens: Some(1),
            top_p: Some(0.0),
            frequency_penalty: Some(-2.0),
            presence_penalty: Some(2.0),
        };
        assert!(boundary.validate().is_ok());

        let boundary_max = SetLlmConfigInput {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key: "sk-test".to_string(),
            model: "deepseek-chat".to_string(),
            temperature: Some(2.0),
            max_tokens: Some(2048),
            top_p: Some(1.0),
            frequency_penalty: Some(2.0),
            presence_penalty: Some(-2.0),
        };
        assert!(boundary_max.validate().is_ok());
    }

    #[test]
    fn set_input_rejects_temperature_above_max() {
        let config = SetLlmConfigInput {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key: "sk-test".to_string(),
            model: "deepseek-chat".to_string(),
            temperature: Some(2.1),
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        };
        assert!(
            matches!(config.validate(), Err(DomainError::ValidationFailed { field }) if field == "temperature")
        );
    }

    #[test]
    fn set_input_rejects_max_tokens_zero() {
        let config = SetLlmConfigInput {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key: "sk-test".to_string(),
            model: "deepseek-chat".to_string(),
            temperature: None,
            max_tokens: Some(0),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        };
        assert!(
            matches!(config.validate(), Err(DomainError::ValidationFailed { field }) if field == "maxTokens")
        );
    }

    #[test]
    fn set_input_rejects_empty_api_key() {
        let mut input = valid_set_input();
        input.api_key = " ".to_string();
        assert!(
            matches!(input.validate(), Err(DomainError::ValidationFailed { field }) if field == "apiKey")
        );
    }

    #[test]
    fn list_models_input_requires_endpoint_and_api_key() {
        let valid = ListLlmModelsInput {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key: "sk-test".to_string(),
        };
        assert!(valid.validate().is_ok());

        let missing_endpoint = ListLlmModelsInput {
            endpoint: " ".to_string(),
            ..valid.clone()
        };
        assert!(
            matches!(missing_endpoint.validate(), Err(DomainError::ValidationFailed { field }) if field == "endpoint")
        );

        let missing_api_key = ListLlmModelsInput {
            api_key: "".to_string(),
            ..valid
        };
        assert!(
            matches!(missing_api_key.validate(), Err(DomainError::ValidationFailed { field }) if field == "apiKey")
        );
    }

    #[test]
    fn test_connection_input_requires_model() {
        let valid = TestLlmConnectionInput {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key: "sk-test".to_string(),
            model: "deepseek-chat".to_string(),
        };
        assert!(valid.validate().is_ok());

        let missing_model = TestLlmConnectionInput {
            model: " ".to_string(),
            ..valid
        };
        assert!(
            matches!(missing_model.validate(), Err(DomainError::ValidationFailed { field }) if field == "model")
        );
    }

    #[test]
    fn test_connection_input_requires_endpoint_and_api_key() {
        let missing_endpoint = TestLlmConnectionInput {
            endpoint: " ".to_string(),
            api_key: "sk-test".to_string(),
            model: "deepseek-chat".to_string(),
        };
        assert!(
            matches!(missing_endpoint.validate(), Err(DomainError::ValidationFailed { field }) if field == "endpoint")
        );

        let missing_api_key = TestLlmConnectionInput {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key: "".to_string(),
            model: "deepseek-chat".to_string(),
        };
        assert!(
            matches!(missing_api_key.validate(), Err(DomainError::ValidationFailed { field }) if field == "apiKey")
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
