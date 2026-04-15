use chrono::Utc;

use crate::infra::database::Database;
struct Migration {
    version: u32,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_schema",
        sql: "
        DEFINE TABLE theme_card SCHEMAFULL;
        DEFINE FIELD schema_version ON theme_card TYPE int;
        DEFINE FIELD name            ON theme_card TYPE string;
        DEFINE FIELD system_prompt   ON theme_card TYPE string;
        DEFINE FIELD created_at      ON theme_card TYPE string;
        DEFINE FIELD updated_at      ON theme_card TYPE string;

        DEFINE TABLE session SCHEMAFULL;
        DEFINE FIELD theme_card_id ON session TYPE string;
        DEFINE FIELD created_at    ON session TYPE string;
        DEFINE FIELD updated_at    ON session TYPE string;

        DEFINE TABLE message SCHEMAFULL;
        DEFINE FIELD session_id ON message TYPE string;
        DEFINE FIELD role       ON message TYPE string;
        DEFINE FIELD content    ON message TYPE string;
        DEFINE FIELD created_at ON message TYPE string;
        DEFINE INDEX idx_message_session ON message FIELDS session_id;

        DEFINE TABLE app_config SCHEMAFULL;
        DEFINE FIELD endpoint ON app_config TYPE option<string>;
        DEFINE FIELD api_key  ON app_config TYPE option<string>;
        DEFINE FIELD model    ON app_config TYPE option<string>;

        DEFINE TABLE _migration SCHEMAFULL;
        DEFINE FIELD version    ON _migration TYPE int;
        DEFINE FIELD name       ON _migration TYPE string;
        DEFINE FIELD applied_at ON _migration TYPE string;
    ",
    },
    Migration {
        version: 2,
        name: "session_last_opened_at",
        // option<string> 允许空值，兼容 v1 存量 session 记录（旧记录该字段返回 None）
        sql: "DEFINE FIELD last_opened_at ON session TYPE option<string>;",
    },
];

// ── 对外执行入口 ───────────────────────────────────────────────────────────────

/// 对 `db` 执行所有尚未落地的迁移。
///
/// 已写入 `_migration` 的版本会被自动跳过，因此应用每次启动时都可以放心调用。
pub async fn run(db: &Database) -> anyhow::Result<()> {
    let inner = db.inner();

    // 先读出当前已应用的最高版本；如果还没有迁移记录，就从 0 开始。
    // 全新数据库里 `_migration` 可能还不存在，这里统一按版本 0 处理。
    let applied_version: u32 = inner
        .query("SELECT VALUE version FROM _migration ORDER BY version DESC LIMIT 1")
        .await
        .ok()
        .and_then(|mut r| r.take::<Option<u32>>(0).ok().flatten())
        .unwrap_or(0);

    for migration in MIGRATIONS {
        if migration.version <= applied_version {
            continue;
        }

        tracing::info!(
            version = migration.version,
            name = migration.name,
            "applying migration"
        );

        inner.query(migration.sql).await?;

        // 写入迁移记录，后续启动时就不会重复执行这一版。
        inner
            .query("CREATE type::record('_migration', $version) CONTENT { version: $version, name: $name, applied_at: $applied_at }")
            .bind(("version", migration.version))
            .bind(("name", migration.name))
            .bind(("applied_at", Utc::now().to_rfc3339()))
            .await?;

        tracing::info!(version = migration.version, "migration applied");
    }

    Ok(())
}
