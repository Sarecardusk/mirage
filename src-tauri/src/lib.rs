mod command;
mod domain;
mod gateway;
mod infra;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::{Emitter, Manager};

use command::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder =
        tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
            command::config::get_llm_config,
            command::config::get_llm_api_key,
            command::config::set_llm_config,
            command::config::list_llm_models,
            command::config::test_llm_connection,
            command::llm::invoke_llm_generation,
            command::session::append_message,
            command::session::create_session,
            command::session::delete_session,
            command::session::list_messages,
            command::session::list_sessions,
            command::session::touch_session,
            command::theme_card::create_theme_card,
            command::theme_card::delete_theme_card,
            command::theme_card::get_theme_card,
            command::theme_card::list_theme_cards,
            command::theme_card::update_theme_card,
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/types/bindings.ts",
        )
        .expect("Failed to export Specta types");

    // `invoke_handler` 需要在进入 setup 闭包前构建，避免移动后无法复用 builder。
    let invoke_handler = builder.invoke_handler();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(invoke_handler)
        .setup(move |app| {
            builder.mount_events(app);

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app_data_dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app_data_dir");

            let db_path = app_data_dir.join("db");
            let app_handle = app.handle().clone();

            // setup() 发生在事件循环启动前，此处 block_on 不会阻塞 UI 事件循环。
            // 启动不变量：DB/Vault/迁移/默认配置都成功后才发布 AppState 并标记 ready。
            tauri::async_runtime::block_on(async move {
                let db = Arc::new(
                    infra::database::Database::connect_file(&db_path)
                        .await
                        .expect("failed to connect to SurrealDB"),
                );

                let vault = Arc::new(
                    infra::vault::Vault::open(
                        &app_data_dir,
                        &infra::vault::MachineLocalKeyProvider,
                    )
                    .expect("failed to open vault"),
                );

                infra::migration::run(&db, &vault)
                    .await
                    .expect("database migration failed — startup aborted");

                let state = AppState::new(Arc::clone(&db), Arc::clone(&vault));

                state
                    .app_config_repo
                    .seed_defaults()
                    .await
                    .expect("failed to seed app config defaults");

                app_handle.manage(state);

                let managed: tauri::State<AppState> = app_handle.state();
                managed.ready.store(true, Ordering::Release);

                app_handle
                    .emit(
                        "AppReady",
                        serde_json::json!({ "timestamp": chrono::Utc::now().to_rfc3339() }),
                    )
                    .expect("failed to emit AppReady");

                tracing::info!("startup sequence complete — AppReady emitted");
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
