use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::llm::{LlmConfig, DEFAULT_LLM_API_KEY_REF};
use crate::infra::database::Database;

/// `app_config` 表中那条唯一 LLM 配置记录所使用的固定主键。
const LLM_RECORD_KEY: &str = "llm";

pub struct AppConfigRepo {
    db: Arc<Database>,
}

impl AppConfigRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 读取已落盘的 `LlmConfig`；如果记录还不存在，就退回默认配置。
    pub async fn get_llm_config(&self) -> Result<LlmConfig, DomainError> {
        let rows: Vec<serde_json::Value> = self
            .db
            .inner()
            .query("SELECT endpoint, api_key_ref, model, temperature, max_tokens, top_p, frequency_penalty, presence_penalty FROM type::record('app_config', $key)")
            .bind(("key", LLM_RECORD_KEY))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .take(0)
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        match rows.into_iter().next() {
            Some(val) => {
                let defaults = LlmConfig::default();
                Ok(LlmConfig {
                    endpoint: val["endpoint"]
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or(defaults.endpoint),
                    api_key_ref: val["api_key_ref"]
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or(defaults.api_key_ref),
                    model: val["model"]
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or(defaults.model),
                    temperature: val["temperature"].as_f64(),
                    max_tokens: val["max_tokens"]
                        .as_i64()
                        .and_then(|v| u32::try_from(v).ok()),
                    top_p: val["top_p"].as_f64(),
                    frequency_penalty: val["frequency_penalty"].as_f64(),
                    presence_penalty: val["presence_penalty"].as_f64(),
                })
            }
            None => Ok(LlmConfig::default()),
        }
    }

    /// 写入新的 `LlmConfig`，直接覆盖旧值。
    /// 这里使用 `UPDATE`，因此记录不存在时也会顺手补建出来。
    pub async fn set_llm_config(&self, config: &LlmConfig) -> Result<(), DomainError> {
        self.db
            .inner()
            .query("UPDATE type::record('app_config', $key) CONTENT { endpoint: $endpoint, api_key_ref: $api_key_ref, model: $model, temperature: $temperature, max_tokens: $max_tokens, top_p: $top_p, frequency_penalty: $frequency_penalty, presence_penalty: $presence_penalty }")
            .bind(("key", LLM_RECORD_KEY))
            .bind(("endpoint", config.endpoint.clone()))
            .bind(("api_key_ref", config.api_key_ref.clone()))
            .bind(("model", config.model.clone()))
            .bind(("temperature", config.temperature))
            .bind(("max_tokens", config.max_tokens))
            .bind(("top_p", config.top_p))
            .bind(("frequency_penalty", config.frequency_penalty))
            .bind(("presence_penalty", config.presence_penalty))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// 首次启动且配置尚未落盘时，补一份默认 `LlmConfig`。
    pub async fn seed_defaults(&self) -> Result<(), DomainError> {
        let rows: Vec<serde_json::Value> = self
            .db
            .inner()
            .query("SELECT endpoint FROM type::record('app_config', $key)")
            .bind(("key", LLM_RECORD_KEY))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .take(0)
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        if rows.is_empty() {
            let defaults = LlmConfig::default();
            self.db
                .inner()
                .create::<Option<serde_json::Value>>(("app_config", LLM_RECORD_KEY))
                .content(serde_json::json!({
                    "endpoint": defaults.endpoint,
                    "api_key_ref": DEFAULT_LLM_API_KEY_REF,
                    "model": defaults.model,
                }))
                .await
                .map_err(|e| DomainError::StorageFailed {
                    message: e.to_string(),
                })?;
        }

        Ok(())
    }
}

// ── 测试 ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use crate::domain::llm::{LlmConfig, DEFAULT_LLM_API_KEY_REF};
    use crate::infra::app_config_repo::AppConfigRepo;
    use crate::infra::database::Database;
    use crate::infra::migration;
    use crate::infra::vault::{MachineLocalKeyProvider, Vault};

    fn make_vault() -> Arc<Vault> {
        let vault_dir =
            std::env::temp_dir().join(format!("mirage-vault-app-config-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&vault_dir).unwrap();
        Arc::new(Vault::open(&vault_dir, &MachineLocalKeyProvider).unwrap())
    }

    async fn make_repo() -> AppConfigRepo {
        let db = Arc::new(Database::connect_memory().await.unwrap());
        let vault = make_vault();
        migration::run(&db, &vault).await.unwrap();
        let repo = AppConfigRepo::new(db);
        repo.seed_defaults().await.unwrap();
        repo
    }

    #[tokio::test]
    async fn get_returns_defaults_after_seed() {
        let repo = make_repo().await;
        let config = repo.get_llm_config().await.unwrap();
        assert_eq!(config.endpoint, "https://api.deepseek.com");
        assert_eq!(config.model, "deepseek-chat");
        assert_eq!(config.api_key_ref, DEFAULT_LLM_API_KEY_REF);
    }

    #[tokio::test]
    async fn set_and_get_roundtrip() {
        let repo = make_repo().await;

        let config = LlmConfig {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key_ref: DEFAULT_LLM_API_KEY_REF.to_string(),
            model: "gpt-4o".to_string(),
            temperature: None,
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        };
        repo.set_llm_config(&config).await.unwrap();

        let fetched = repo.get_llm_config().await.unwrap();
        assert_eq!(fetched.api_key_ref, DEFAULT_LLM_API_KEY_REF);
        assert_eq!(fetched.model, "gpt-4o");
    }

    #[tokio::test]
    async fn generation_params_roundtrip() {
        let repo = make_repo().await;

        let config = LlmConfig {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key_ref: DEFAULT_LLM_API_KEY_REF.to_string(),
            model: "deepseek-chat".to_string(),
            temperature: Some(0.8),
            max_tokens: Some(256),
            top_p: Some(0.95),
            frequency_penalty: Some(-0.5),
            presence_penalty: Some(1.0),
        };
        repo.set_llm_config(&config).await.unwrap();

        let fetched = repo.get_llm_config().await.unwrap();
        assert_eq!(fetched.temperature, Some(0.8));
        assert_eq!(fetched.max_tokens, Some(256));
        assert_eq!(fetched.top_p, Some(0.95));
        assert_eq!(fetched.frequency_penalty, Some(-0.5));
        assert_eq!(fetched.presence_penalty, Some(1.0));
    }

    #[tokio::test]
    async fn generation_params_null_roundtrip() {
        let repo = make_repo().await;

        let config = LlmConfig {
            endpoint: "https://api.deepseek.com".to_string(),
            api_key_ref: DEFAULT_LLM_API_KEY_REF.to_string(),
            model: "deepseek-chat".to_string(),
            temperature: None,
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        };
        repo.set_llm_config(&config).await.unwrap();

        let fetched = repo.get_llm_config().await.unwrap();
        assert_eq!(fetched.temperature, None);
        assert_eq!(fetched.max_tokens, None);
        assert_eq!(fetched.top_p, None);
        assert_eq!(fetched.frequency_penalty, None);
        assert_eq!(fetched.presence_penalty, None);
    }
}
