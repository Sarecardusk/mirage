# DEBT-003: Vault 安全存储

> Last Updated: 2026-04-17

## 目标

- 消除 `app_config:llm.api_key` 明文落盘。
- 新增本地 Vault（`vault.key` + `vault.enc`）管理 LLM API Key。
- IPC 契约切换为 `apiKeyRef` 读侧 + `SetLlmConfigInput` 写侧。
- 迁移 v4 一次性搬运旧明文并清空数据库字段。

## 落地结果（本次）

- Rust 新增 `infra/vault.rs`：
  - `VaultKeyProvider` trait
  - `MachineLocalKeyProvider`
  - `Vault`（AES-256-GCM + 原子落盘）
  - 单测覆盖 roundtrip / tamper / key missing / 首次初始化 / 并发 set
- `domain/llm.rs`：
  - `LlmConfig.api_key` -> `api_key_ref`
  - 新增 `SetLlmConfigInput` 与 `LlmConfigRecord`
- `infra/migration.rs`：
  - `run(db, vault)` 新签名
  - 新增 v4 与 Rust hook（SQL + hook 成功后才写 `_migration`）
  - 搬运逻辑幂等可重试
- `command/config.rs`：
  - 新增 `get_llm_api_key`
  - `set_llm_config` 改为 `SetLlmConfigInput` 并写入 Vault
- `command/llm.rs`：
  - 生成前从 Vault 取明文 `api_key`
- `command/error.rs`：
  - 新增 `From<VaultError> for IpcError`
  - 错误码：`VAULT_DECRYPT_FAILED` / `VAULT_KEY_MISSING` / `VAULT_WRITE_FAILED` / `VAULT_CORRUPTED`
- `command/state.rs` / `lib.rs`：
  - `AppState` 持有 `Arc<Vault>`
  - 启动顺序收紧为 `connect -> vault -> migration -> state -> seed_defaults -> manage`
- 前端对齐：
  - `bindings.ts` 更新：`apiKeyRef`、`SetLlmConfigInput`、`getLlmApiKey`
  - `schemas/config.ts` 新增 `SetLlmConfigInputSchema`
  - `services/config.ts` 新增 `getLlmApiKey()`
  - `SettingsView.vue` 改为并行加载 `getLlmConfig + getLlmApiKey`
- 文档对齐：
  - `docs/COMPATIBILITY.md`
  - `docs/SECURITY.md`
  - `ARCHITECTURE.md`
  - `CHANGELOG.md`
  - `AGENTS.md`
  - `docs/exec-plans/tech-debt-tracker.md`

## 待最终确认

- 全量验证命令：`pnpm check && pnpm lint && pnpm test`
- 手动路径回归：首次启动 / 旧数据迁移 / vault.enc 篡改 / vault.key 丢失
