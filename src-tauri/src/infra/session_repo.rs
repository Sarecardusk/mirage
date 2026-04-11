use std::collections::HashMap;

use chrono::Utc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::session::{ChatRole, Message, Session, SessionRepository};

pub struct MemorySessionRepo {
    sessions: Mutex<HashMap<String, Session>>,
    messages: Mutex<HashMap<String, Vec<Message>>>,
}

impl MemorySessionRepo {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            messages: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MemorySessionRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionRepository for MemorySessionRepo {
    async fn create_session(&self, theme_card_id: &str) -> Result<Session, DomainError> {
        let now = Utc::now().to_rfc3339();
        let session = Session {
            id: Uuid::new_v4().to_string(),
            theme_card_id: theme_card_id.to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        {
            let mut sessions = self.sessions.lock().await;
            sessions.insert(session.id.clone(), session.clone());
        }

        let mut messages = self.messages.lock().await;
        messages.insert(session.id.clone(), Vec::new());

        Ok(session)
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<Session>, DomainError> {
        let sessions = self.sessions.lock().await;
        Ok(sessions.get(session_id).cloned())
    }

    async fn list_messages(&self, session_id: &str) -> Result<Vec<Message>, DomainError> {
        let sessions = self.sessions.lock().await;
        if !sessions.contains_key(session_id) {
            return Err(DomainError::SessionNotFound {
                id: session_id.to_string(),
            });
        }
        drop(sessions);

        let messages = self.messages.lock().await;
        Ok(messages.get(session_id).cloned().unwrap_or_default())
    }

    async fn append_message(
        &self,
        session_id: &str,
        role: ChatRole,
        content: String,
    ) -> Result<Message, DomainError> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| DomainError::SessionNotFound {
                id: session_id.to_string(),
            })?;
        session.updated_at = Utc::now().to_rfc3339();
        drop(sessions);

        let message = Message {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            created_at: Utc::now().to_rfc3339(),
        };

        let mut messages = self.messages.lock().await;
        messages
            .entry(session_id.to_string())
            .or_default()
            .push(message.clone());

        Ok(message)
    }
}
