use tauri::ipc::Channel;
use tauri::State;

use crate::command::error::IpcError;
use crate::command::state::AppState;
use crate::domain::error::DomainError;
use crate::domain::llm::LlmStreamEvent;
use crate::domain::session::{ChatRole, SessionRepository};
use crate::domain::theme_card::ThemeCardRepository;
use crate::gateway;

#[tauri::command]
#[specta::specta]
pub async fn invoke_llm_generation(
    state: State<'_, AppState>,
    session_id: String,
    theme_card_id: String,
    channel: Channel<LlmStreamEvent>,
) -> Result<(), IpcError> {
    if session_id.trim().is_empty() {
        return Err(IpcError::from(DomainError::ValidationFailed {
            field: "sessionId".to_string(),
        }));
    }
    if theme_card_id.trim().is_empty() {
        return Err(IpcError::from(DomainError::ValidationFailed {
            field: "themeCardId".to_string(),
        }));
    }

    let theme_card = state
        .theme_card_repo
        .get_by_id(theme_card_id.as_str())
        .await
        .map_err(IpcError::from)?
        .ok_or_else(|| {
            IpcError::from(DomainError::ThemeCardNotFound {
                id: theme_card_id.clone(),
            })
        })?;

    let session = state
        .session_repo
        .get_session(session_id.as_str())
        .await
        .map_err(IpcError::from)?
        .ok_or_else(|| {
            IpcError::from(DomainError::SessionNotFound {
                id: session_id.clone(),
            })
        })?;
    if session.theme_card_id != theme_card.id {
        return Err(IpcError::from(DomainError::ValidationFailed {
            field: "themeCardId".to_string(),
        }));
    }

    let config = state.llm_config.read().await.clone();
    if config.api_key.trim().is_empty() {
        return Err(IpcError::from(DomainError::ValidationFailed {
            field: "apiKey".to_string(),
        }));
    }

    let history = state
        .session_repo
        .list_messages(session_id.as_str())
        .await
        .map_err(IpcError::from)?;

    match gateway::llm::stream_chat_completion(
        &config,
        theme_card.name.as_str(),
        theme_card.system_prompt.as_str(),
        &history,
        &channel,
    )
    .await
    {
        Ok(full_text) => {
            state
                .session_repo
                .append_message(session_id.as_str(), ChatRole::Assistant, full_text.clone())
                .await
                .map_err(IpcError::from)?;

            channel
                .send(LlmStreamEvent::Completion { full_text })
                .map_err(|error| IpcError::gateway(error.to_string(), false))?;

            Ok(())
        }
        Err(error) => {
            let message = format!("llm generation failed: {error}");
            let _ = channel.send(LlmStreamEvent::Error {
                error_code: "GATEWAY_UPSTREAM_ERROR".to_string(),
                message: message.clone(),
                retryable: true,
            });
            Err(IpcError::gateway(message, true))
        }
    }
}
