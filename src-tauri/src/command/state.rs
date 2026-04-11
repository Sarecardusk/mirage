use tokio::sync::RwLock;

use crate::domain::llm::LlmConfig;
use crate::infra::theme_card_repo::MemoryThemeCardRepo;
use crate::infra::session_repo::MemorySessionRepo;

pub struct AppState {
    pub llm_config: RwLock<LlmConfig>,
    pub session_repo: MemorySessionRepo,
    pub theme_card_repo: MemoryThemeCardRepo,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            llm_config: RwLock::new(LlmConfig::default()),
            session_repo: MemorySessionRepo::new(),
            theme_card_repo: MemoryThemeCardRepo::new(),
        }
    }
}
