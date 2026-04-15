use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::theme_card::{CreateThemeCardInput, ThemeCard, ThemeCardRepository};
use crate::infra::database::Database;

pub struct SurrealThemeCardRepo {
    db: Arc<Database>,
}

impl SurrealThemeCardRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl ThemeCardRepository for SurrealThemeCardRepo {
    async fn create(&self, input: CreateThemeCardInput) -> Result<ThemeCard, DomainError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // 这里用 serde_json::Value 作为 `content`，才能满足 SurrealDB 的类型约束。
        self.db
            .inner()
            .create::<Option<serde_json::Value>>(("theme_card", id.as_str()))
            .content(serde_json::json!({
                "schema_version": 1,
                "name": input.name.trim(),
                "system_prompt": input.system_prompt.trim(),
                "created_at": &now,
                "updated_at": &now,
            }))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .ok_or_else(|| DomainError::StorageFailed {
                message: "create theme_card returned no record".to_string(),
            })?;

        // 这些字段都来自当前输入与本地生成值，直接回组装结果即可。
        Ok(ThemeCard {
            id,
            schema_version: 1,
            name: input.name.trim().to_string(),
            system_prompt: input.system_prompt.trim().to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    async fn list(&self) -> Result<Vec<ThemeCard>, DomainError> {
        // `record::id(id)` 只取主键本体，不会把表名前缀一并带回来。
        let rows: Vec<serde_json::Value> = self
            .db
            .inner()
            .query("SELECT record::id(id) AS id, schema_version, name, system_prompt, created_at, updated_at FROM theme_card")
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .take(0)
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        rows.into_iter()
            .map(json_to_theme_card)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<ThemeCard>, DomainError> {
        let rows: Vec<serde_json::Value> = self
            .db
            .inner()
            .query("SELECT record::id(id) AS id, schema_version, name, system_prompt, created_at, updated_at FROM type::record('theme_card', $id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .take(0)
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        match rows.into_iter().next() {
            Some(row) => json_to_theme_card(row).map(Some),
            None => Ok(None),
        }
    }
}

// ── JSON 转领域对象 ───────────────────────────────────────────────────────────

fn json_to_theme_card(val: serde_json::Value) -> Result<ThemeCard, DomainError> {
    let missing = |field: &str| DomainError::StorageFailed {
        message: format!("theme_card record missing field '{field}'"),
    };

    Ok(ThemeCard {
        id: val["id"].as_str().ok_or_else(|| missing("id"))?.to_string(),
        schema_version: val["schema_version"]
            .as_u64()
            .ok_or_else(|| missing("schema_version"))? as u32,
        name: val["name"]
            .as_str()
            .ok_or_else(|| missing("name"))?
            .to_string(),
        system_prompt: val["system_prompt"]
            .as_str()
            .ok_or_else(|| missing("system_prompt"))?
            .to_string(),
        created_at: val["created_at"]
            .as_str()
            .ok_or_else(|| missing("created_at"))?
            .to_string(),
        updated_at: val["updated_at"]
            .as_str()
            .ok_or_else(|| missing("updated_at"))?
            .to_string(),
    })
}

// ── 测试 ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::domain::theme_card::{CreateThemeCardInput, ThemeCardRepository};
    use crate::infra::database::Database;
    use crate::infra::migration;
    use crate::infra::surreal_theme_card_repo::SurrealThemeCardRepo;

    async fn make_repo() -> SurrealThemeCardRepo {
        let db = Arc::new(Database::connect_memory().await.unwrap());
        migration::run(&db).await.unwrap();
        SurrealThemeCardRepo::new(db)
    }

    #[tokio::test]
    async fn create_and_get_theme_card() {
        let repo = make_repo().await;

        let card = repo
            .create(CreateThemeCardInput {
                name: "Detective".to_string(),
                system_prompt: "Stay in character".to_string(),
            })
            .await
            .unwrap();

        assert!(!card.id.is_empty());
        assert_eq!(card.name, "Detective");
        assert_eq!(card.schema_version, 1);

        let fetched = repo.get_by_id(&card.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, card.id);
        assert_eq!(fetched.name, "Detective");
    }

    #[tokio::test]
    async fn list_returns_all_cards() {
        let repo = make_repo().await;

        repo.create(CreateThemeCardInput {
            name: "A".to_string(),
            system_prompt: "P".to_string(),
        })
        .await
        .unwrap();

        repo.create(CreateThemeCardInput {
            name: "B".to_string(),
            system_prompt: "Q".to_string(),
        })
        .await
        .unwrap();

        let cards = repo.list().await.unwrap();
        assert_eq!(cards.len(), 2);
    }

    #[tokio::test]
    async fn get_by_id_returns_none_for_unknown_id() {
        let repo = make_repo().await;
        let result = repo.get_by_id("nonexistent").await.unwrap();
        assert!(result.is_none());
    }
}
