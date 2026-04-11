use tauri::State;

use crate::command::error::IpcError;
use crate::command::state::AppState;
use crate::domain::error::DomainError;
use crate::domain::session::{
    AppendMessageInput, CreateSessionInput, Message, Session, SessionRepository,
};
use crate::domain::theme_card::ThemeCardRepository;

#[tauri::command]
#[specta::specta]
pub async fn create_session(
    state: State<'_, AppState>,
    input: CreateSessionInput,
) -> Result<Session, IpcError> {
    input.validate()?;

    let maybe_theme_card = state
        .theme_card_repo
        .get_by_id(input.theme_card_id.as_str())
        .await
        .map_err(IpcError::from)?;
    if maybe_theme_card.is_none() {
        return Err(IpcError::from(DomainError::ThemeCardNotFound {
            id: input.theme_card_id.clone(),
        }));
    }

    state
        .session_repo
        .create_session(input.theme_card_id.as_str())
        .await
        .map_err(IpcError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn list_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<Message>, IpcError> {
    if session_id.trim().is_empty() {
        return Err(IpcError::from(DomainError::ValidationFailed {
            field: "sessionId".to_string(),
        }));
    }

    state
        .session_repo
        .list_messages(session_id.as_str())
        .await
        .map_err(IpcError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn append_message(
    state: State<'_, AppState>,
    input: AppendMessageInput,
) -> Result<Message, IpcError> {
    input.validate()?;

    state
        .session_repo
        .append_message(
            input.session_id.as_str(),
            input.role.clone(),
            input.content.trim().to_string(),
        )
        .await
        .map_err(IpcError::from)
}
