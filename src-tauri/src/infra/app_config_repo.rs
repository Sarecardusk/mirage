use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::llm::LlmConfig;
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
            .query("SELECT endpoint, api_key, model FROM type::record('app_config', $key)")
            .bind(("key", LLM_RECORD_KEY))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .take(0)
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        Ok(rows
            .into_iter()
            .next()
            .map(json_to_config)
            .unwrap_or_default())
    }

    /// 写入新的 `LlmConfig`，直接覆盖旧值。
    /// 这里使用 `UPDATE`，因此记录不存在时也会顺手补建出来。
    pub async fn set_llm_config(&self, config: &LlmConfig) -> Result<(), DomainError> {
        self.db
            .inner()
            .query("UPDATE type::record('app_config', $key) CONTENT { endpoint: $endpoint, api_key: $api_key, model: $model }")
            .bind(("key", LLM_RECORD_KEY))
            .bind(("endpoint", config.endpoint.clone()))
            .bind(("api_key", config.api_key.clone()))
            .bind(("model", config.model.clone()))
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
                    "api_key": defaults.api_key,
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

fn json_to_config(val: serde_json::Value) -> LlmConfig {
    let defaults = LlmConfig::default();
    LlmConfig {
        endpoint: val["endpoint"]
            .as_str()
            .map(str::to_string)
            .unwrap_or(defaults.endpoint),
        api_key: val["api_key"]
            .as_str()
            .map(str::to_string)
            .unwrap_or(defaults.api_key),
        model: val["model"]
            .as_str()
            .map(str::to_string)
            .unwrap_or(defaults.model),
    }
}

// ── 测试 ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::infra::app_config_repo::AppConfigRepo;
    use crate::infra::database::Database;
    use crate::infra::migration;

    async fn make_repo() -> AppConfigRepo {
        let db = Arc::new(Database::connect_memory().await.unwrap());
        migration::run(&db).await.unwrap();
        let repo = AppConfigRepo::new(db);
        repo.seed_defaults().await.unwrap();
        repo
    }

    #[tokio::test]
    async fn get_returns_defaults_after_seed() {
        let repo = make_repo().await;
        let config = repo.get_llm_config().await.unwrap();
        assert_eq!(config.endpoint, "https://api.openai.com/v1");
    }

    #[tokio::test]
    async fn set_and_get_roundtrip() {
        let repo = make_repo().await;

        let mut config = repo.get_llm_config().await.unwrap();
        config.api_key = "sk-test-key".to_string();
        config.model = "gpt-4o".to_string();
        repo.set_llm_config(&config).await.unwrap();

        let fetched = repo.get_llm_config().await.unwrap();
        assert_eq!(fetched.api_key, "sk-test-key");
        assert_eq!(fetched.model, "gpt-4o");
    }
}
