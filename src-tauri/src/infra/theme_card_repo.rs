use std::collections::HashMap;

use chrono::Utc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::theme_card::{CreateThemeCardInput, ThemeCard, ThemeCardRepository};

pub struct MemoryThemeCardRepo {
    theme_cards: Mutex<HashMap<String, ThemeCard>>,
}

impl MemoryThemeCardRepo {
    pub fn new() -> Self {
        Self {
            theme_cards: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MemoryThemeCardRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeCardRepository for MemoryThemeCardRepo {
    async fn create(&self, input: CreateThemeCardInput) -> Result<ThemeCard, DomainError> {
        let now = Utc::now().to_rfc3339();
        let theme_card = ThemeCard {
            id: Uuid::new_v4().to_string(),
            schema_version: 1,
            name: input.name.trim().to_string(),
            system_prompt: input.system_prompt.trim().to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        let mut cards = self.theme_cards.lock().await;
        cards.insert(theme_card.id.clone(), theme_card.clone());

        Ok(theme_card)
    }

    async fn list(&self) -> Result<Vec<ThemeCard>, DomainError> {
        let cards = self.theme_cards.lock().await;
        Ok(cards.values().cloned().collect())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<ThemeCard>, DomainError> {
        let cards = self.theme_cards.lock().await;
        Ok(cards.get(id).cloned())
    }
}
