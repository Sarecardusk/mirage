# Tech Debt Tracker

本表仅记录与“高重构成本边界”相关的技术债。

| ID       | Debt                                     | Impacted Boundary            | Current Risk | Suggested Direction                                                                                                   | Status   |
| -------- | ---------------------------------------- | ---------------------------- | ------------ | --------------------------------------------------------------------------------------------------------------------- | -------- |
| DEBT-001 | `specta-zod` 自动链路不可用              | IPC 类型一致性               | 中           | 继续维持 Rust + 手写 Zod 的双轨校验，等待生态稳定                                                                     | Open     |
| DEBT-002 | Sync 仅有保留接口无执行路径              | Sync-Ready 边界清晰度        | 低           | 在未来需求确认后补独立设计文档，不提前实现                                                                            | Open     |
| DEBT-003 | `LlmConfig.api_key` 明文存储于 SurrealDB | 安全模型（SECURITY.md）      | 中           | 已通过 Vault（ring）迁移为 `api_key_ref` + 本地密文存储；argon2 在口令派生特性落地时引入                              | Resolved |
| DEBT-004 | MVP 仍以 `vault.key` 作为主密钥入口      | 主密钥路线与多端 E2EE 一致性 | 高           | 在 `v1.0.0` 稳定阶段前完成替换：改为“用户口令派生主密钥”，并作为本地 Vault 与未来多端同步端到端加密的统一上游密钥来源 | Open     |

> Release Gate: `DEBT-004` 在创建 `v1.0.0` Release tag 前必须为 `Resolved`。
