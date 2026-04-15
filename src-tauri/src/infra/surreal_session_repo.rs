use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::session::{ChatRole, Message, Session, SessionRepository};
use crate::infra::database::Database;

// ChatRole 与字符串互转

fn role_to_str(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::System => "system",
    }
}

fn str_to_role(s: &str) -> ChatRole {
    match s {
        "assistant" => ChatRole::Assistant,
        "system" => ChatRole::System,
        _ => ChatRole::User,
    }
}

// 存储实现

pub struct SurrealSessionRepo {
    db: Arc<Database>,
}

impl SurrealSessionRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl SessionRepository for SurrealSessionRepo {
    async fn create_session(&self, theme_card_id: &str) -> Result<Session, DomainError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.db
            .inner()
            .create::<Option<serde_json::Value>>(("session", id.as_str()))
            .content(serde_json::json!({
                "theme_card_id": theme_card_id,
                "created_at": &now,
                "updated_at": &now,
            }))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .ok_or_else(|| DomainError::StorageFailed {
                message: "create_session returned no record".to_string(),
            })?;

        Ok(Session {
            id,
            theme_card_id: theme_card_id.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<Session>, DomainError> {
        let rows: Vec<serde_json::Value> = self
            .db
            .inner()
            .query("SELECT record::id(id) AS id, theme_card_id, created_at, updated_at FROM type::record('session', $id)")
            .bind(("id", session_id.to_string()))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .take(0)
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        match rows.into_iter().next() {
            Some(row) => json_to_session(row).map(Some),
            None => Ok(None),
        }
    }

    async fn list_messages(&self, session_id: &str) -> Result<Vec<Message>, DomainError> {
        // 先确认 session 存在，再决定是否继续查消息。
        let session = self.get_session(session_id).await?;
        if session.is_none() {
            return Err(DomainError::SessionNotFound {
                id: session_id.to_string(),
            });
        }

        let rows: Vec<serde_json::Value> = self
            .db
            .inner()
            .query("SELECT record::id(id) AS id, session_id, role, content, created_at FROM message WHERE session_id = $sid ORDER BY created_at ASC")
            .bind(("sid", session_id.to_string()))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .take(0)
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        rows.into_iter()
            .map(json_to_message)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn append_message(
        &self,
        session_id: &str,
        role: ChatRole,
        content: String,
    ) -> Result<Message, DomainError> {
        if self.get_session(session_id).await?.is_none() {
            return Err(DomainError::SessionNotFound {
                id: session_id.to_string(),
            });
        }

        let now = Utc::now().to_rfc3339();

        self.db
            .inner()
            .query("UPDATE type::record('session', $sid) SET updated_at = $now")
            .bind(("sid", session_id.to_string()))
            .bind(("now", now.clone()))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?;

        let msg_id = Uuid::new_v4().to_string();
        self.db
            .inner()
            .create::<Option<serde_json::Value>>(("message", msg_id.as_str()))
            .content(serde_json::json!({
                "session_id": session_id.to_string(),
                "role": role_to_str(&role),
                "content": content.clone(),
                "created_at": now.clone(),
            }))
            .await
            .map_err(|e| DomainError::StorageFailed {
                message: e.to_string(),
            })?
            .ok_or_else(|| DomainError::StorageFailed {
                message: "append_message returned no record".to_string(),
            })?;

        Ok(Message {
            id: msg_id,
            role,
            content,
            created_at: now,
        })
    }
}

// ── JSON 转领域对象 ───────────────────────────────────────────────────────────

fn json_to_session(val: serde_json::Value) -> Result<Session, DomainError> {
    let missing = |field: &str| DomainError::StorageFailed {
        message: format!("session record missing field '{field}'"),
    };

    Ok(Session {
        id: val["id"].as_str().ok_or_else(|| missing("id"))?.to_string(),
        theme_card_id: val["theme_card_id"]
            .as_str()
            .ok_or_else(|| missing("theme_card_id"))?
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

fn json_to_message(val: serde_json::Value) -> Result<Message, DomainError> {
    let missing = |field: &str| DomainError::StorageFailed {
        message: format!("message record missing field '{field}'"),
    };

    let role_str = val["role"].as_str().ok_or_else(|| missing("role"))?;

    Ok(Message {
        id: val["id"].as_str().ok_or_else(|| missing("id"))?.to_string(),
        role: str_to_role(role_str),
        content: val["content"]
            .as_str()
            .ok_or_else(|| missing("content"))?
            .to_string(),
        created_at: val["created_at"]
            .as_str()
            .ok_or_else(|| missing("created_at"))?
            .to_string(),
    })
}

// ── 测试 ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::domain::session::{ChatRole, SessionRepository};
    use crate::infra::database::Database;
    use crate::infra::migration;
    use crate::infra::surreal_session_repo::SurrealSessionRepo;

    async fn make_repo() -> SurrealSessionRepo {
        let db = Arc::new(Database::connect_memory().await.unwrap());
        migration::run(&db).await.unwrap();
        SurrealSessionRepo::new(db)
    }

    #[tokio::test]
    async fn create_and_get_session() {
        let repo = make_repo().await;
        let session = repo.create_session("card-1").await.unwrap();

        assert!(!session.id.is_empty());
        assert_eq!(session.theme_card_id, "card-1");

        let fetched = repo.get_session(&session.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, session.id);
    }

    #[tokio::test]
    async fn append_and_list_messages() {
        let repo = make_repo().await;
        let session = repo.create_session("card-1").await.unwrap();

        repo.append_message(&session.id, ChatRole::User, "Hello".to_string())
            .await
            .unwrap();

        repo.append_message(&session.id, ChatRole::Assistant, "Hi there".to_string())
            .await
            .unwrap();

        let messages = repo.list_messages(&session.id).await.unwrap();
        assert_eq!(messages.len(), 2);
        assert!(matches!(messages[0].role, ChatRole::User));
        assert!(matches!(messages[1].role, ChatRole::Assistant));
    }

    #[tokio::test]
    async fn list_messages_fails_for_unknown_session() {
        let repo = make_repo().await;
        let result = repo.list_messages("no-such-session").await;
        assert!(result.is_err());
    }
}
