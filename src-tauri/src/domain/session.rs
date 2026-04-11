use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::error::DomainError;

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub theme_card_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionInput {
    pub theme_card_id: String,
}

impl CreateSessionInput {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.theme_card_id.trim().is_empty() {
            return Err(DomainError::ValidationFailed {
                field: "themeCardId".to_string(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub role: ChatRole,
    pub content: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppendMessageInput {
    pub session_id: String,
    pub role: ChatRole,
    pub content: String,
}

impl AppendMessageInput {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.session_id.trim().is_empty() {
            return Err(DomainError::ValidationFailed {
                field: "sessionId".to_string(),
            });
        }
        if self.content.trim().is_empty() {
            return Err(DomainError::ValidationFailed {
                field: "content".to_string(),
            });
        }
        Ok(())
    }
}

pub trait SessionRepository: Send + Sync {
    async fn create_session(&self, theme_card_id: &str) -> Result<Session, DomainError>;
    async fn get_session(&self, session_id: &str) -> Result<Option<Session>, DomainError>;
    async fn list_messages(&self, session_id: &str) -> Result<Vec<Message>, DomainError>;
    async fn append_message(
        &self,
        session_id: &str,
        role: ChatRole,
        content: String,
    ) -> Result<Message, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::{AppendMessageInput, ChatRole, CreateSessionInput};
    use crate::domain::error::DomainError;

    #[test]
    fn create_session_validates_non_empty_theme_card_id() {
        let input = CreateSessionInput {
            theme_card_id: "card-1".to_string(),
        };
        assert!(input.validate().is_ok());
    }

    #[test]
    fn create_session_rejects_empty_theme_card_id() {
        let input = CreateSessionInput {
            theme_card_id: "   ".to_string(),
        };
        assert!(
            matches!(input.validate(), Err(DomainError::ValidationFailed { field }) if field == "themeCardId")
        );
    }

    #[test]
    fn append_message_validates_valid_input() {
        let input = AppendMessageInput {
            session_id: "session-1".to_string(),
            role: ChatRole::User,
            content: "Hello".to_string(),
        };
        assert!(input.validate().is_ok());
    }

    #[test]
    fn append_message_rejects_empty_session_id() {
        let input = AppendMessageInput {
            session_id: "".to_string(),
            role: ChatRole::User,
            content: "Hello".to_string(),
        };
        assert!(
            matches!(input.validate(), Err(DomainError::ValidationFailed { field }) if field == "sessionId")
        );
    }

    #[test]
    fn append_message_rejects_empty_content() {
        let input = AppendMessageInput {
            session_id: "session-1".to_string(),
            role: ChatRole::User,
            content: "   ".to_string(),
        };
        assert!(
            matches!(input.validate(), Err(DomainError::ValidationFailed { field }) if field == "content")
        );
    }
}
