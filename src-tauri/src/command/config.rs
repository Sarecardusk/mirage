use std::sync::atomic::Ordering;

use tauri::State;

use crate::command::error::IpcError;
use crate::command::state::AppState;
use crate::domain::llm::LlmConfig;

#[tauri::command]
#[specta::specta]
pub async fn get_llm_config(state: State<'_, AppState>) -> Result<LlmConfig, IpcError> {
    if !state.ready.load(Ordering::Acquire) {
        return Err(IpcError::app_not_ready());
    }
    state
        .app_config_repo
        .get_llm_config()
        .await
        .map_err(IpcError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn set_llm_config(
    state: State<'_, AppState>,
    input: LlmConfig,
) -> Result<LlmConfig, IpcError> {
    if !state.ready.load(Ordering::Acquire) {
        return Err(IpcError::app_not_ready());
    }
    input.validate()?;

    let config = LlmConfig {
        endpoint: input.endpoint.trim().to_string(),
        api_key: input.api_key.trim().to_string(),
        model: input.model.trim().to_string(),
    };

    state
        .app_config_repo
        .set_llm_config(&config)
        .await
        .map_err(IpcError::from)?;

    Ok(config)
}
