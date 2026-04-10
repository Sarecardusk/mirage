# Mirage Security Boundaries

> Last Updated: 2026-04-05

本文件定义安全模型的底线约束。该约束优先级高于“易用性降级”。

## 安全前提

- 当前版本默认单机本地运行，不引入云账号中心。
- 若未来引入同步，必须先满足设备配对与端到端加密前提。
- 未配对设备必须被硬拒绝，不允许”只读同步”例外路径（HR-01）。

## 密钥与敏感信息策略

- 敏感信息存放于本地加密文件。
- 主密钥由用户口令派生。
- 错误口令不得触发明文 fallback 或降级读取（HR-02）。

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
