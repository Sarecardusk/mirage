use chrono::Utc;

use crate::domain::llm::DEFAULT_LLM_API_KEY_REF;
use crate::infra::database::Database;
use crate::infra::vault::Vault;

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
    Migration {
        version: 3,
        name: "llm_generation_params",
        // 新增生成参数字段；SCHEMAFULL 表中已有记录该字段自动为 NONE，无需回填
        sql: "
            DEFINE FIELD temperature       ON app_config TYPE option<float>;
            DEFINE FIELD max_tokens        ON app_config TYPE option<int>;
            DEFINE FIELD top_p             ON app_config TYPE option<float>;
            DEFINE FIELD frequency_penalty ON app_config TYPE option<float>;
            DEFINE FIELD presence_penalty  ON app_config TYPE option<float>;
        ",
    },
    Migration {
        version: 4,
        name: "llm_api_key_ref",
        sql: "DEFINE FIELD api_key_ref ON app_config TYPE option<string>;",
    },
];

/// 对 `db` 执行所有尚未落地的迁移。
///
/// 已写入 `_migration` 的版本会被自动跳过，因此应用每次启动时都可以放心调用。
pub async fn run(db: &Database, vault: &Vault) -> anyhow::Result<()> {
    let inner = db.inner();

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

        // 先执行 SQL 迁移，确保 schema/data 达到该版本要求。
        inner.query(migration.sql).await?;

        // 对需要 Rust 逻辑补偿的版本执行额外钩子（当前仅 v4）。
        if migration.version == 4 {
            migrate_api_key_to_vault(db, vault).await?;
        }

        // 仅当 SQL 与补偿逻辑都成功后，才记录迁移版本，避免脏成功。
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

async fn migrate_api_key_to_vault(db: &Database, vault: &Vault) -> anyhow::Result<()> {
    let mut response = db
        .inner()
        .query("SELECT api_key, api_key_ref FROM type::record('app_config', $key)")
        .bind(("key", "llm"))
        .await?;

    let rows: Vec<serde_json::Value> = response.take(0)?;
    let Some(row) = rows.into_iter().next() else {
        return Ok(());
    };

    let ref_missing = row["api_key_ref"]
        .as_str()
        .map(str::trim)
        .is_none_or(str::is_empty);

    if !ref_missing {
        return Ok(());
    }

    if let Some(plaintext) = row["api_key"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        vault.set(DEFAULT_LLM_API_KEY_REF, plaintext)?;
    }

    db.inner()
        .query("UPDATE type::record('app_config', $key) SET api_key_ref = $api_key_ref, api_key = NONE")
        .bind(("key", "llm"))
        .bind(("api_key_ref", DEFAULT_LLM_API_KEY_REF))
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use super::{run, MIGRATIONS};
    use crate::domain::llm::DEFAULT_LLM_API_KEY_REF;
    use crate::infra::database::Database;
    use crate::infra::vault::{MachineLocalKeyProvider, Vault};

    fn make_vault() -> Arc<Vault> {
        let vault_dir =
            std::env::temp_dir().join(format!("mirage-vault-migration-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&vault_dir).unwrap();
        Arc::new(Vault::open(&vault_dir, &MachineLocalKeyProvider).unwrap())
    }

    async fn setup_v3_data_with_plaintext_api_key(db: &Database, api_key: &str) {
        for migration in MIGRATIONS.iter().filter(|m| m.version <= 3) {
            db.inner().query(migration.sql).await.unwrap();
        }

        db.inner()
            .query("CREATE type::record('_migration', 3) CONTENT { version: 3, name: 'llm_generation_params', applied_at: time::now() }")
            .await
            .unwrap();

        db.inner()
            .query("CREATE type::record('app_config', 'llm') CONTENT { endpoint: 'https://api.deepseek.com', api_key: $api_key, model: 'deepseek-chat' }")
            .bind(("api_key", api_key.to_string()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn v4_moves_plaintext_api_key_to_vault_and_clears_db_field() {
        let db = Database::connect_memory().await.unwrap();
        let vault = make_vault();
        setup_v3_data_with_plaintext_api_key(&db, "sk-migrate").await;

        run(&db, &vault).await.unwrap();

        assert_eq!(
            vault.get(DEFAULT_LLM_API_KEY_REF).as_deref(),
            Some("sk-migrate")
        );

        let mut response = db
            .inner()
            .query("SELECT api_key, api_key_ref FROM type::record('app_config', 'llm')")
            .await
            .unwrap();
        let rows: Vec<serde_json::Value> = response.take(0).unwrap();
        let row = rows.into_iter().next().unwrap();

        assert_eq!(row["api_key_ref"].as_str(), Some(DEFAULT_LLM_API_KEY_REF));
        assert!(row["api_key"].is_null());
    }

    #[tokio::test]
    async fn v4_migration_is_idempotent() {
        let db = Database::connect_memory().await.unwrap();
        let vault = make_vault();
        setup_v3_data_with_plaintext_api_key(&db, "sk-idempotent").await;

        run(&db, &vault).await.unwrap();
        run(&db, &vault).await.unwrap();

        assert_eq!(
            vault.get(DEFAULT_LLM_API_KEY_REF).as_deref(),
            Some("sk-idempotent")
        );

        let mut response = db
            .inner()
            .query("SELECT VALUE version FROM _migration ORDER BY version DESC LIMIT 1")
            .await
            .unwrap();
        let current: Option<u32> = response.take(0).unwrap();
        assert_eq!(current, Some(4));
    }
}
