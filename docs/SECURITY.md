# Mirage Security Boundaries

> Last Updated: 2026-04-17

本文件定义安全模型的底线约束。该约束优先级高于“易用性降级”。

## 安全前提

- 当前版本默认单机本地运行，不引入云账号中心。
- 若未来引入同步，必须先满足设备配对与端到端加密前提。
- 未配对设备必须被硬拒绝，不允许“只读同步”例外路径（HR-01）。

## 密钥与敏感信息策略

- 敏感信息存放于本地加密文件。
- MVP 主密钥当前使用机器本地随机密钥文件（`vault.key`），该方案仅为过渡实现（`DEBT-004`）。
- 在 `v1.0.0` 稳定阶段前，必须完成替换为“用户口令派生主密钥”，不得继续以 `vault.key` 作为最终主密钥方案。
- 口令主密钥必须作为统一上游密钥来源，供本地 Vault 与未来多端同步端到端加密复用。
- 口令/密钥错误不得触发明文 fallback 或降级读取（HR-02）。

## Vault（MVP）

- 存储位置：`app_data_dir/vault.key`（32-byte 主密钥）与 `app_data_dir/vault.enc`（AES-256-GCM 密文）。
- 持久化策略：`tmp -> write -> sync_all -> rename` 原子落盘，避免崩溃时产生半写文件。
- 数据策略：SurrealDB 仅保存 `api_key_ref`，不保存 LLM API Key 明文。
- 异常策略：
  - `VAULT_DECRYPT_FAILED`：密文篡改或密钥不匹配，硬拒绝。
  - `VAULT_KEY_MISSING`：有密文但缺密钥，硬拒绝。
  - `VAULT_CORRUPTED`：文件结构损坏，不做静默恢复。
- Windows 现状：MVP 不额外强化 ACL，当前依赖 `app_data_dir` 的用户级隔离；ACL 收紧作为后续独立计划处理。

## 敏感载荷入口

- 配对参数、口令相关输入、签名相关输入必须进入 Rust `validator` 校验（MH-08）。
- 校验失败必须返回可追踪错误码，不得吞错。

## 失败处理原则

- 未配对设备：硬拒绝（HR-01）。
- 口令错误：硬拒绝（HR-02）。
- 签名校验失败：硬拒绝（HR-03）。
- 不允许以“功能可用”为由绕过以上拒绝策略。

## Tauri Capability 与 CSP 管理

### Capability 最小权限原则

- 权限声明集中在 `src-tauri/capabilities/default.json`。
- 当前仅授权 `core:default` 与 `opener:default`。
- 新增 Tauri 插件时，必须同步在 `capabilities/default.json` 添加对应权限条目。
- 不预授权：不提前添加未使用的权限，按需逐项开放。

### CSP 策略

- 当前 `tauri.conf.json` 中 `"csp": null`（开发阶段，方便调试）。
- 正式发布前须配置 CSP：仅允许自身来源（`'self'`）加必要的外部 API 域名。
- CSP 配置变更需在 commit message 中说明理由。
- **切换时机**：在首次创建 GitHub Release tag 前，必须通过 ExecPlan 完成 CSP 配置。创建 Release tag 的 PR 审查清单须包含 CSP 非 null 验证。

## 同步相关边界（v1）

- 禁止引入可执行同步流程。
- `SyncTransport`、`SyncOperationLog`、`ConflictResolver` 仅允许接口占位。
