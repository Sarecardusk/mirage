use tauri::State;

use crate::command::error::IpcError;
use crate::command::state::AppState;
use crate::domain::llm::{LlmConfig, SetLlmConfigInput};

#[tauri::command]
#[specta::specta]
pub async fn get_llm_config(state: State<'_, AppState>) -> Result<LlmConfig, IpcError> {
    let config = state.llm_config.read().await.clone();
    Ok(config)
}

#[tauri::command]
#[specta::specta]
pub async fn set_llm_config(
    state: State<'_, AppState>,
    input: SetLlmConfigInput,
) -> Result<LlmConfig, IpcError> {
    input.validate()?;

    let config = LlmConfig {
        endpoint: input.endpoint.trim().to_string(),
        api_key: input.api_key.trim().to_string(),
        model: input.model.trim().to_string(),
    };

    let mut config_guard = state.llm_config.write().await;
    *config_guard = config.clone();

    Ok(config)
}
