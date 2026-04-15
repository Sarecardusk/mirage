use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::infra::app_config_repo::AppConfigRepo;
use crate::infra::database::Database;
use crate::infra::surreal_session_repo::SurrealSessionRepo;
use crate::infra::surreal_theme_card_repo::SurrealThemeCardRepo;

/// 通过 `tauri::Manager::manage` 注入的全局共享状态。
///
/// 这里的字段要么包在 `Arc` 里，要么天然满足 `Send + Sync`，
/// 这样 Tauri 才能把它们安全分发给各个异步命令处理器。
pub struct AppState {
    /// 启动流程全部完成后才会置为 true，包括数据库连接、迁移执行、
    /// `AppReady` 事件发出等步骤。
    /// 业务命令会先检查这个标记；若仍为 false，就返回 `APP_NOT_READY`。
    pub ready: AtomicBool,

    pub theme_card_repo: SurrealThemeCardRepo,
    pub session_repo: SurrealSessionRepo,
    pub app_config_repo: AppConfigRepo,
}

impl AppState {
    /// 基于已经连通且完成迁移的 `Database` 构造应用状态。
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            ready: AtomicBool::new(false),
            theme_card_repo: SurrealThemeCardRepo::new(Arc::clone(&db)),
            session_repo: SurrealSessionRepo::new(Arc::clone(&db)),
            app_config_repo: AppConfigRepo::new(Arc::clone(&db)),
        }
    }
}
