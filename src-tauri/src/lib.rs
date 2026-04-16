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
            command::config::set_llm_config,
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

    // 先把 invoke handler 取出来，避免 `builder` 进入 setup 闭包后无法再用。
    let invoke_handler = builder.invoke_handler();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(invoke_handler)
        .setup(move |app| {
            // 把 Specta 导出的事件类型一并挂进 TypeScript 绑定里。
            builder.mount_events(app);

            // 确定持久化数据库所在目录。
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app_data_dir");

            let db_path = app_data_dir.join("db");
            let app_handle = app.handle().clone();

            // 启动阶段的整套初始化都放到 Tauri 的异步运行时里执行。
            // 这里可以安全使用 block_on，因为 setup() 发生在事件循环启动前，
            // 不会把主循环卡死。
            tauri::async_runtime::block_on(async move {
                // 第一步：连接 SurrealDB，这里使用 SurrealKV 文件存储。
                let db = Arc::new(
                    infra::database::Database::connect_file(&db_path)
                        .await
                        .expect("failed to connect to SurrealDB"),
                );

                // 第二步：执行尚未应用的迁移；重复启动时也可以安全调用。
                infra::migration::run(&db)
                    .await
                    .expect("database migration failed — startup aborted");

                // 第三步：组装 AppState，让各个仓储共享同一个数据库句柄。
                let state = AppState::new(Arc::clone(&db));

                // 第四步：首次安装时补齐默认应用配置。
                state
                    .app_config_repo
                    .seed_defaults()
                    .await
                    .expect("failed to seed app config defaults");

                // 第五步：把状态注入到 Tauri，后续命令才能拿到它。
                app_handle.manage(state);

                // 第六步：标记应用已就绪，业务命令从这一刻开始放行。
                let managed: tauri::State<AppState> = app_handle.state();
                managed.ready.store(true, Ordering::Release);

                // 第七步：广播 AppReady，通知前端结束启动中的占位状态。
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
