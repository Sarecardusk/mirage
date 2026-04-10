# Mirage Compatibility Policy

> Last Updated: 2026-04-05

本文件定义 Mirage 的兼容性语义，防止“看似小改动，实际破坏公开契约”的情况进入主线。

## 兼容性覆盖面

以下内容都视为公开契约：

- Tauri IPC 命令名与参数结构。
- `RuntimeEvent` 事件名与 payload 结构。
- `invoke_llm_generation` 的流式事件类型与终止语义。
- 结构化错误包络与错误码。
- `ThemeCard` / `Session` / 应用级配置的持久化 schema。
- Rust 导出的 TS 类型与手写 Zod schema 的对应关系。

## 变更分类

### 非破坏性变更

- 新增可选字段，且旧调用方可忽略。
- 新增错误码，但不改变已有错误码语义。
- 新增事件，但不改变已有事件名与既有 payload 字段语义。
- 在不改变既有输入输出语义的前提下补充日志字段或内部诊断字段。

### 破坏性变更

- 重命名、删除、拆分、合并已有命令名、事件名、错误码。
- 将可选字段改为必填，或改变字段类型、枚举值、默认行为。
- 修改流式事件终止语义，使既有调用方无法按原逻辑判断完成或失败。
- 修改持久化结构但未提供显式迁移。
- 修改 Zod / TS / Rust 对齐关系，导致“编译能过但运行时不兼容”。

## 版本化规则

- `ThemeCard` 的版本化由 `schemaVersion` 驱动。
- 应用级存储结构的版本化由 `app_schema_version` 驱动。
- 任何持久化 breaking change 都必须附带显式迁移与回放样本。
- v1 阶段的公开命令、错误码、事件名默认视为追加式演进；一旦进入实现（首次合并到 main 分支），不允许无计划重命名。

## 状态标记

为避免“目标态文档”与“当前实现”混淆，公开契约在评审时必须显式区分：

- `Planned`：约束已锁定，但仓库尚未有实现与测试证据。
- `Implemented`：已有实现，并至少有一项自动化或启动期证据。

当前仓库默认状态：

- `Theme Card` / `Session` / `Memory Retrieval Layer` / `LLM Gateway` 相关契约：`Planned`
- Tauri 基础工程链、capability、CSP 开发态、命令脚手架：`Implemented`

## 变更流程

- 改动公开契约前，先更新权威文档，再改实现。
- 破坏性变更必须附带：
  - ExecPlan
  - 迁移策略
  - 兼容性说明
  - 回归用例更新
- 非破坏性变更也必须更新对应测试或验收清单，避免“声明追加，证据缺失”。

## 公开契约注册表（v1 最小集合）

以下注册表列出 v1 范围内所有公开契约条目。任何新增、重命名或删除都必须先更新此表，再改实现。

### UseCaseCommand

| 命令名                  | 方向      | 参数摘要                                       | 返回摘要           | 状态    | Owner     |
| ----------------------- | --------- | ---------------------------------------------- | ------------------ | ------- | --------- |
| `create_theme_card`     | UI → Rust | `CreateThemeCardInput`                         | `ThemeCard`        | Planned | Rust Core |
| `update_theme_card`     | UI → Rust | `UpdateThemeCardInput`（含 `theme_card_id`）   | `ThemeCard`        | Planned | Rust Core |
| `create_session`        | UI → Rust | `CreateSessionInput`（含 `theme_card_id`）     | `Session`          | Planned | Rust Core |
| `switch_session`        | UI → Rust | `session_id`                                   | `Session`          | Planned | Rust Core |
| `run_memory_retrieval`  | UI → Rust | `session_id` + `theme_card_id` + query context | 检索结果           | Planned | Rust Core |
| `invoke_llm_generation` | UI → Rust | `session_id` + Channel + generation params     | 流式事件（见下表） | Planned | Rust Core |

### ResourceCrudCommand

| 命令名              | 方向      | 说明                   | 状态    | Owner     |
| ------------------- | --------- | ---------------------- | ------- | --------- |
| `get_app_config`    | UI → Rust | 读取应用级配置         | Planned | Rust Core |
| `update_app_config` | UI → Rust | 写入应用级非敏感配置   | Planned | Rust Core |
| `list_theme_cards`  | UI → Rust | 列出所有 Theme Card    | Planned | Rust Core |
| `get_theme_card`    | UI → Rust | 按 ID 读取单张卡片     | Planned | Rust Core |
| `delete_theme_card` | UI → Rust | 删除卡片（含引用清理） | Planned | Rust Core |
| `list_sessions`     | UI → Rust | 列出指定卡片的 Session | Planned | Rust Core |
| `delete_session`    | UI → Rust | 删除 Session           | Planned | Rust Core |

### 流式事件类型（Channel API）

| 事件类型     | payload 关键字段                                           | 终止语义 | 稳定性 |
| ------------ | ---------------------------------------------------------- | -------- | ------ |
| `TokenChunk` | `text: string`                                             | 非终止   | 稳定   |
| `Completion` | `full_text: string`, `usage?: Usage`                       | 终止     | 稳定   |
| `Error`      | `error_code: string`, `message: string`, `retryable: bool` | 终止     | 稳定   |

### RuntimeEvent（Tauri Event System）

| 事件名     | payload 摘要    | 方向      | 状态    | Owner     |
| ---------- | --------------- | --------- | ------- | --------- |
| `AppReady` | `{ timestamp }` | Rust → UI | Planned | Rust Core |

> 后续新增 RuntimeEvent（如后台进度、告警）时，必须同步更新此表。

### 错误码注册表

| error_code               | 类别            | retryable | 说明                         | 状态    |
| ------------------------ | --------------- | --------- | ---------------------------- | ------- |
| `APP_NOT_READY`          | `InfraError`    | true      | AppReady 前调用业务命令      | Planned |
| `ENTITY_NOT_FOUND`       | `DomainError`   | false     | 实体不存在                   | Planned |
| `VALIDATION_FAILED`      | `DomainError`   | false     | 输入校验失败                 | Planned |
| `MIGRATION_FAILED`       | `DomainError`   | false     | 迁移执行失败                 | Planned |
| `VERSION_MISMATCH`       | `DomainError`   | false     | schemaVersion 不匹配         | Planned |
| `STORAGE_READ_FAILED`    | `InfraError`    | true      | SurrealDB 读取失败           | Planned |
| `STORAGE_WRITE_FAILED`   | `InfraError`    | true      | SurrealDB 写入失败           | Planned |
| `ASSET_WRITE_FAILED`     | `InfraError`    | true      | 资产文件写入失败             | Planned |
| `DANGLING_REFERENCE`     | `InfraError`    | false     | 悬挂引用检测                 | Planned |
| `COMMAND_TIMEOUT`        | `InfraError`    | true      | 非流式命令超时               | Planned |
| `CLIENT_DISCONNECTED`    | `InfraError`    | false     | 前端断连                     | Planned |
| `SHUTDOWN_ABORT`         | `InfraError`    | false     | 应用关闭中断                 | Planned |
| `UNPAIRED_DEVICE`        | `SecurityError` | false     | 未配对设备                   | Planned |
| `INVALID_PASSPHRASE`     | `SecurityError` | false     | 口令错误                     | Planned |
| `SIGNATURE_FAILED`       | `SecurityError` | false     | 签名校验失败                 | Planned |
| `GATEWAY_TIMEOUT`        | `GatewayError`  | true      | 上游供应商超时               | Planned |
| `GATEWAY_UPSTREAM_ERROR` | `GatewayError`  | true      | 上游返回可重试错误           | Planned |
| `GATEWAY_FATAL_ERROR`    | `GatewayError`  | false     | 上游返回不可重试错误         | Planned |
| `NORMALIZATION_FAILED`   | `GatewayError`  | false     | 响应归一化失败               | Planned |
| `RETRIEVAL_DEGRADED`     | `GatewayError`  | false     | 检索降级（可继续无检索路径） | Planned |
| `GENERATION_CANCELLED`   | `DomainError`   | false     | 用户主动取消生成             | Planned |
| `GENERATION_CONFLICT`    | `DomainError`   | false     | 同 Session 已有活跃生成流    | Planned |

> 新增错误码时必须同步更新此表与 Rust/TS 两侧实现（TD-01）。状态从 `Planned` 变为 `Implemented` 时一并更新。

### 错误码添加清单

新增错误码时，按以下步骤操作：

1. **分类**：使用 `DESIGN.md` 错误分类决策树确定 `category` 和 `retryable` 值。
2. **注册**：在上方错误码注册表中添加一行，填写全部字段（error_code、类别、retryable、说明、状态）。命名使用 `UPPER_SNAKE_CASE`。
3. **同步实现**：
   - Rust：在对应层的 `thiserror` enum 中添加变体，确保 `command/error.rs` 的 `From` 实现覆盖新变体。
   - TypeScript：在 `src/schemas/error.ts` 的错误码常量或 Zod schema 中添加对应值。
   - 测试：在 command 层测试中添加触发该错误码的用例。

> 新增错误码不需要 ExecPlan（见 `AGENTS.md` 边界案例判定表）。修改已有错误码的语义或 `retryable` 值才需要。

## 错误码与事件名稳定性规则

- **命名规范**：错误码使用 `UPPER_SNAKE_CASE`；事件名使用 `PascalCase`；命令名使用 `snake_case`。
- **一旦 Implemented，不可无计划重命名**：首次合并到 main 后，该名称即视为稳定公开面。重命名属于破坏性变更，需走 ExecPlan。
- **只增不删**：v1 阶段错误码和事件类型只做追加。废弃需标记 `Deprecated` 并保留至少一个大版本。
- **payload 字段追加安全**：新增可选字段是非破坏性变更，但删除字段或改变类型是破坏性变更。

## 样本与证据

- 持久化 schema 的兼容性通过迁移 fixture 与回放测试证明。
- 错误包络兼容性通过命令层集成测试证明。
- 流式事件兼容性通过事件序列测试证明。
- 前端类型契约兼容性通过 Zod schema 测试与服务层测试证明。
