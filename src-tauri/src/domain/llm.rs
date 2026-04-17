use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::error::DomainError;

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
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
            api_key: String::new(),
            model: "deepseek-chat".to_string(),
            temperature: None,
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        }
    }
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
        validate_f64_range(self.temperature, "temperature", 0.0, 2.0)?;
        validate_f64_range(self.top_p, "topP", 0.0, 1.0)?;
        validate_f64_range(self.frequency_penalty, "frequencyPenalty", -2.0, 2.0)?;
        validate_f64_range(self.presence_penalty, "presencePenalty", -2.0, 2.0)?;
        if let Some(mt) = self.max_tokens {
            if mt == 0 {
                return Err(DomainError::ValidationFailed {
                    field: "maxTokens".to_string(),
                });
            }
        }
        Ok(())
    }
}

impl ListLlmModelsInput {
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
        Ok(())
    }
}

impl TestLlmConnectionInput {
    pub fn validate(&self) -> Result<(), DomainError> {
        ListLlmModelsInput {
            endpoint: self.endpoint.clone(),
            api_key: self.api_key.clone(),
        }
        .validate()?;

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
    use super::{ListLlmModelsInput, LlmConfig, TestLlmConnectionInput};
    use crate::domain::error::DomainError;

    fn valid_config() -> LlmConfig {
        LlmConfig {
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

    #[test]
    fn accepts_generation_params_within_range() {
        let mid = LlmConfig {
            temperature: Some(1.0),
            max_tokens: Some(512),
            top_p: Some(0.9),
            frequency_penalty: Some(-1.5),
            presence_penalty: Some(1.5),
            ..valid_config()
        };
        assert!(mid.validate().is_ok());

        // 闭区间端点也必须通过
        let boundary = LlmConfig {
            temperature: Some(0.0),
            max_tokens: Some(1),
            top_p: Some(0.0),
            frequency_penalty: Some(-2.0),
            presence_penalty: Some(2.0),
            ..valid_config()
        };
        assert!(boundary.validate().is_ok());

        let boundary_max = LlmConfig {
            temperature: Some(2.0),
            top_p: Some(1.0),
            frequency_penalty: Some(2.0),
            presence_penalty: Some(-2.0),
            ..valid_config()
        };
        assert!(boundary_max.validate().is_ok());
    }

    #[test]
    fn rejects_temperature_above_max() {
        let config = LlmConfig {
            temperature: Some(2.1),
            ..valid_config()
        };
        assert!(
            matches!(config.validate(), Err(DomainError::ValidationFailed { field }) if field == "temperature")
        );
    }

    #[test]
    fn rejects_temperature_below_min() {
        let config = LlmConfig {
            temperature: Some(-0.1),
            ..valid_config()
        };
        assert!(
            matches!(config.validate(), Err(DomainError::ValidationFailed { field }) if field == "temperature")
        );
    }

    #[test]
    fn rejects_max_tokens_zero() {
        let config = LlmConfig {
            max_tokens: Some(0),
            ..valid_config()
        };
        assert!(
            matches!(config.validate(), Err(DomainError::ValidationFailed { field }) if field == "maxTokens")
        );
    }

    #[test]
    fn rejects_top_p_above_max() {
        let config = LlmConfig {
            top_p: Some(1.1),
            ..valid_config()
        };
        assert!(
            matches!(config.validate(), Err(DomainError::ValidationFailed { field }) if field == "topP")
        );
    }

    #[test]
    fn rejects_frequency_penalty_out_of_range() {
        let config = LlmConfig {
            frequency_penalty: Some(2.5),
            ..valid_config()
        };
        assert!(
            matches!(config.validate(), Err(DomainError::ValidationFailed { field }) if field == "frequencyPenalty")
        );
    }

    #[test]
    fn rejects_presence_penalty_out_of_range() {
        let config = LlmConfig {
            presence_penalty: Some(-3.0),
            ..valid_config()
        };
        assert!(
            matches!(config.validate(), Err(DomainError::ValidationFailed { field }) if field == "presencePenalty")
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
