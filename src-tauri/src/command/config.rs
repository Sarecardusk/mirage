use std::sync::atomic::Ordering;

use tauri::State;

use crate::command::error::IpcError;
use crate::command::state::AppState;
use crate::domain::llm::{ListLlmModelsInput, LlmConfig, TestLlmConnectionInput};

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
        temperature: input.temperature,
        max_tokens: input.max_tokens,
        top_p: input.top_p,
        frequency_penalty: input.frequency_penalty,
        presence_penalty: input.presence_penalty,
    };

    state
        .app_config_repo
        .set_llm_config(&config)
        .await
        .map_err(IpcError::from)?;

    Ok(config)
}

/// 用表单中的实时 endpoint + api_key（无需先保存）查询该 endpoint 支持的模型列表。
#[tauri::command]
#[specta::specta]
pub async fn list_llm_models(
    state: State<'_, AppState>,
    input: ListLlmModelsInput,
) -> Result<Vec<String>, IpcError> {
    if !state.ready.load(Ordering::Acquire) {
        return Err(IpcError::app_not_ready());
    }
    input.validate().map_err(IpcError::from)?;
    crate::gateway::llm::list_models(&input)
        .await
        .map_err(|e| IpcError::gateway(e.to_string(), true))
}

/// 用表单中的实时配置发一次最小请求（max_tokens=1）验证连通性。
#[tauri::command]
#[specta::specta]
pub async fn test_llm_connection(
    state: State<'_, AppState>,
    input: TestLlmConnectionInput,
) -> Result<(), IpcError> {
    if !state.ready.load(Ordering::Acquire) {
        return Err(IpcError::app_not_ready());
    }
    input.validate().map_err(IpcError::from)?;
    crate::gateway::llm::test_connection(&input)
        .await
        .map_err(|e| IpcError::gateway(e.to_string(), true))
}
