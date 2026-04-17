use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::theme_card::{
    CreateThemeCardInput, ThemeCard, ThemeCardRepository, UpdateThemeCardInput,
};
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

        // 用 serde_json::Value 作为 content 满足 SurrealDB 的类型约束；
        // RETURN 子句确保返回与 list/get_by_id 格式一致的字段（record::id 去除表名前缀）。
        let rows: Vec<serde_json::Value> = self
            .db
            .inner()
            .query("CREATE type::record('theme_card', $id) CONTENT { schema_version: 1, name: $name, system_prompt: $system_prompt, created_at: $created_at, updated_at: $updated_at } RETURN record::id(id) AS id, schema_version, name, system_prompt, created_at, updated_at")
            .bind(("id", id.clone()))
            .bind(("name", input.name.trim().to_string()))
            .bind(("system_prompt", input.system_prompt.trim().to_string()))
            .bind(("created_at", now.clone()))
            .bind(("updated_at", now.clone()))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .take(0)
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        rows.into_iter()
            .next()
            .ok_or_else(|| DomainError::StorageFailed {
                message: "create theme_card returned no record".to_string(),
            })
            .and_then(json_to_theme_card)
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

    async fn update(&self, input: UpdateThemeCardInput) -> Result<ThemeCard, DomainError> {
        let now = Utc::now().to_rfc3339();

        // RETURN 子句与 create/list/get_by_id 保持一致的字段格式；
        // 若记录不存在，UPDATE 返回空数组，直接映射为 ThemeCardNotFound。
        let rows: Vec<serde_json::Value> = self
            .db
            .inner()
            .query("UPDATE type::record('theme_card', $id) SET name = $name, system_prompt = $system_prompt, updated_at = $updated_at RETURN record::id(id) AS id, schema_version, name, system_prompt, created_at, updated_at")
            .bind(("id", input.theme_card_id.clone()))
            .bind(("name", input.name.trim().to_string()))
            .bind(("system_prompt", input.system_prompt.trim().to_string()))
            .bind(("updated_at", now))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .take(0)
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        rows.into_iter()
            .next()
            .ok_or_else(|| DomainError::ThemeCardNotFound {
                id: input.theme_card_id.clone(),
            })
            .and_then(json_to_theme_card)
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        // 先确认卡片存在，不存在则快速失败
        if self.get_by_id(id).await?.is_none() {
            return Err(DomainError::ThemeCardNotFound { id: id.to_string() });
        }

        // 第一步：删除该 Theme Card 下所有 Session 的消息（先删子级）
        self.db
            .inner()
            .query("DELETE message WHERE session_id IN (SELECT VALUE record::id(id) FROM session WHERE theme_card_id = $card_id)")
            .bind(("card_id", id.to_string()))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        // 第二步：删除所有关联 Session
        self.db
            .inner()
            .query("DELETE session WHERE theme_card_id = $card_id")
            .bind(("card_id", id.to_string()))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        // 第三步：删除 Theme Card 本身
        self.db
            .inner()
            .query("DELETE type::record('theme_card', $id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        Ok(())
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

    use uuid::Uuid;

    use crate::domain::error::DomainError;
    use crate::domain::session::SessionRepository;
    use crate::domain::theme_card::{
        CreateThemeCardInput, ThemeCardRepository, UpdateThemeCardInput,
    };
    use crate::infra::database::Database;
    use crate::infra::migration;
    use crate::infra::surreal_session_repo::SurrealSessionRepo;
    use crate::infra::surreal_theme_card_repo::SurrealThemeCardRepo;
    use crate::infra::vault::{MachineLocalKeyProvider, Vault};

    fn make_vault() -> Arc<Vault> {
        let vault_dir =
            std::env::temp_dir().join(format!("mirage-vault-theme-card-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&vault_dir).unwrap();
        Arc::new(Vault::open(&vault_dir, &MachineLocalKeyProvider).unwrap())
    }

    async fn make_repo() -> SurrealThemeCardRepo {
        let db = Arc::new(Database::connect_memory().await.unwrap());
        let vault = make_vault();
        migration::run(&db, &vault).await.unwrap();
        SurrealThemeCardRepo::new(db)
    }

    // 同时返回 theme card repo 和 session repo（共享同一个 db），用于级联删除测试
    async fn make_repos() -> (SurrealThemeCardRepo, SurrealSessionRepo) {
        let db = Arc::new(Database::connect_memory().await.unwrap());
        let vault = make_vault();
        migration::run(&db, &vault).await.unwrap();
        let tc_repo = SurrealThemeCardRepo::new(Arc::clone(&db));
        let s_repo = SurrealSessionRepo::new(Arc::clone(&db));
        (tc_repo, s_repo)
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

    #[tokio::test]
    async fn update_changes_name_and_system_prompt() {
        let repo = make_repo().await;
        let card = repo
            .create(CreateThemeCardInput {
                name: "Original".to_string(),
                system_prompt: "Old prompt".to_string(),
            })
            .await
            .unwrap();

        let original_updated_at = card.updated_at.clone();

        let updated = repo
            .update(UpdateThemeCardInput {
                theme_card_id: card.id.clone(),
                name: "Renamed".to_string(),
                system_prompt: "New prompt".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(updated.id, card.id);
        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.system_prompt, "New prompt");
        // updated_at 应已更新（可能在同一秒内执行，因此用 >= 比较）
        assert!(updated.updated_at >= original_updated_at);
        // immutable fields unchanged
        assert_eq!(updated.created_at, card.created_at);
        assert_eq!(updated.schema_version, card.schema_version);
    }

    #[tokio::test]
    async fn update_returns_error_for_unknown_id() {
        let repo = make_repo().await;
        let result = repo
            .update(UpdateThemeCardInput {
                theme_card_id: "no-such-card".to_string(),
                name: "Name".to_string(),
                system_prompt: "Prompt".to_string(),
            })
            .await;
        assert!(matches!(result, Err(DomainError::ThemeCardNotFound { .. })));
    }

    #[tokio::test]
    async fn delete_removes_theme_card() {
        let repo = make_repo().await;
        let card = repo
            .create(CreateThemeCardInput {
                name: "Temp".to_string(),
                system_prompt: "Prompt".to_string(),
            })
            .await
            .unwrap();

        repo.delete(&card.id).await.unwrap();

        assert!(repo.get_by_id(&card.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn delete_cascades_sessions_and_messages() {
        let (tc_repo, s_repo) = make_repos().await;
        let card = tc_repo
            .create(CreateThemeCardInput {
                name: "Card".to_string(),
                system_prompt: "Prompt".to_string(),
            })
            .await
            .unwrap();

        // 建立 session 和消息
        let session = s_repo.create_session(&card.id).await.unwrap();
        s_repo
            .append_message(
                &session.id,
                crate::domain::session::ChatRole::User,
                "Hi".to_string(),
            )
            .await
            .unwrap();

        // 删除 Theme Card，应级联清理 session 和 message
        tc_repo.delete(&card.id).await.unwrap();

        // session 不再存在
        assert!(s_repo.get_session(&session.id).await.unwrap().is_none());
        // 查询已删 session 的消息应失败（list_messages 会检查 session 存在性）
        let msg_result = s_repo.list_messages(&session.id).await;
        assert!(msg_result.is_err());
    }

    #[tokio::test]
    async fn delete_returns_error_for_unknown_id() {
        let repo = make_repo().await;
        let result = repo.delete("no-such-card").await;
        assert!(matches!(result, Err(DomainError::ThemeCardNotFound { .. })));
    }
}
