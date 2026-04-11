mod command;
mod domain;
mod gateway;
mod infra;

use command::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
        command::config::get_llm_config,
        command::config::set_llm_config,
        command::llm::invoke_llm_generation,
        command::session::append_message,
        command::session::create_session,
        command::session::list_messages,
        command::theme_card::create_theme_card,
        command::theme_card::get_theme_card,
        command::theme_card::list_theme_cards,
    ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/types/bindings.ts",
        )
        .expect("Failed to export Specta types");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
