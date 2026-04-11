use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::error::DomainError;

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ThemeCard {
    pub id: String,
    pub schema_version: u32,
    pub name: String,
    pub system_prompt: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateThemeCardInput {
    pub name: String,
    pub system_prompt: String,
}

impl CreateThemeCardInput {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() {
            return Err(DomainError::ValidationFailed {
                field: "name".to_string(),
            });
        }
        if self.system_prompt.trim().is_empty() {
            return Err(DomainError::ValidationFailed {
                field: "systemPrompt".to_string(),
            });
        }
        Ok(())
    }
}

pub trait ThemeCardRepository: Send + Sync {
    async fn create(&self, input: CreateThemeCardInput) -> Result<ThemeCard, DomainError>;
    async fn list(&self) -> Result<Vec<ThemeCard>, DomainError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<ThemeCard>, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::CreateThemeCardInput;
    use crate::domain::error::DomainError;

    #[test]
    fn validate_accepts_non_empty_fields() {
        let input = CreateThemeCardInput {
            name: "Detective".to_string(),
            system_prompt: "Stay in character".to_string(),
        };

        let result = input.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn validate_rejects_empty_name() {
        let input = CreateThemeCardInput {
            name: "   ".to_string(),
            system_prompt: "Stay in character".to_string(),
        };

        let result = input.validate();
        assert!(matches!(result, Err(DomainError::ValidationFailed { field }) if field == "name"));
    }

    #[test]
    fn validate_rejects_empty_system_prompt() {
        let input = CreateThemeCardInput {
            name: "Detective".to_string(),
            system_prompt: "".to_string(),
        };

        let result = input.validate();
        assert!(matches!(result, Err(DomainError::ValidationFailed { field }) if field == "systemPrompt"));
    }
}
