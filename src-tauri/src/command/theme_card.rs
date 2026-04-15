use std::sync::atomic::Ordering;

use tauri::State;

use crate::command::error::IpcError;
use crate::command::state::AppState;
use crate::domain::error::DomainError;
use crate::domain::theme_card::{CreateThemeCardInput, ThemeCard, ThemeCardRepository};

#[tauri::command]
#[specta::specta]
pub async fn create_theme_card(
    state: State<'_, AppState>,
    input: CreateThemeCardInput,
) -> Result<ThemeCard, IpcError> {
    if !state.ready.load(Ordering::Acquire) {
        return Err(IpcError::app_not_ready());
    }
    input.validate()?;
    state
        .theme_card_repo
        .create(input)
        .await
        .map_err(IpcError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn list_theme_cards(state: State<'_, AppState>) -> Result<Vec<ThemeCard>, IpcError> {
    if !state.ready.load(Ordering::Acquire) {
        return Err(IpcError::app_not_ready());
    }
    let mut cards = state.theme_card_repo.list().await.map_err(IpcError::from)?;
    cards.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(cards)
}

#[tauri::command]
#[specta::specta]
pub async fn get_theme_card(
    state: State<'_, AppState>,
    theme_card_id: String,
) -> Result<ThemeCard, IpcError> {
    if !state.ready.load(Ordering::Acquire) {
        return Err(IpcError::app_not_ready());
    }
    state
        .theme_card_repo
        .get_by_id(theme_card_id.as_str())
        .await
        .map_err(IpcError::from)?
        .ok_or_else(|| {
            IpcError::from(DomainError::ThemeCardNotFound {
                id: theme_card_id.clone(),
            })
        })
}
