# Mirage Harness Boundaries

> Last Updated: 2026-04-05

本文件定义 Mirage 作为一个 Harness 工程时的执行约束。
这里的 Harness，不是产品规格，也不是页面设计，而是“命令如何执行、错误如何收敛、事件如何流动、故障如何诊断、变更如何受控”的工程护栏。

## 目标

- 把产品边界文档转成可执行、可验证、可回归的工程约束。
- 让 `UseCaseCommand`、`ResourceCrudCommand`、`RuntimeEvent`、流式生成通道形成稳定的公开面。
- 让失败、取消、重试、并发、回放、诊断这些高返工成本问题在落地前先被约束。

## 非目标

- 不定义页面视觉与交互文案。
- 不替代 `DESIGN.md` / `RELIABILITY.md` / `SECURITY.md` 的领域、安全、数据约束。
- 不为供应商私有协议做直接暴露；Harness 只暴露 Mirage 自己的稳定语义。

## Harness 覆盖范围

Harness 的公开工程面固定为以下五类：

- 启动序列与 `AppReady` 之前的阻断行为。
- `UseCaseCommand` / `ResourceCrudCommand` 的输入、输出、错误包络。
- `invoke_llm_generation` 的流式事件序列。
- `RuntimeEvent` 的广播语义与生命周期。
- 迁移、日志、诊断、回放、兼容性与测试门禁。

任何会改变上述五类公开面的改动，都必须同步更新本文件以及 `COMPATIBILITY.md` / `TEST_STRATEGY.md` 中的对应条目。

## 命令执行语义

### 命令分类

- 读命令：无副作用查询，允许并发执行。
- 变更命令：会修改领域状态或持久化状态，必须受 per-entity mutex 保护（见下方"Per-entity Mutex 实现规范"）。
- 流式命令：当前仅 `invoke_llm_generation`，同时具备命令与事件双重语义。

### 默认规则

- Rust 是命令执行与 `retryable` 判断的唯一权威。
- 前端不得对变更命令做静默自动重试；是否重试必须基于结构化错误显式决策。
- 读命令仅在错误被标记为 `retryable` 时允许调用方重试。
- 同一 `session_id` 在同一时刻默认只允许一个活跃的 LLM 生成流；若未来放宽，需专项设计文档。

### 终止条件

- 每次命令调用都必须以“成功结果”或“结构化错误”结束，不允许悬空等待。
- 流式命令必须以 `Completion` 或 `Error` 其中之一终止，二者互斥。
- UI 侧断开流式消费后，Rust 侧应尽快取消上游请求与本地清理，不保留悬挂任务。

## Per-entity Mutex 实现规范

### 层级

- Mutex 持有与获取逻辑位于 `command/` 层（Tauri command 处理器内），不下沉到 `domain/` 或 `infra/`。
- 使用 `HashMap<EntityId, Arc<Mutex<()>>>` 模式，按需为每个实体 ID 创建锁。
- `EntityId` 根据命令语义取 `theme_card_id` 或 `session_id`。

### 超时策略

- Mutex 获取使用 `tokio::time::timeout`，默认超时 **10 秒**。
- 超时后返回 `COMMAND_TIMEOUT`（`InfraError`, `retryable: true`），不 panic。
- 超时事件记录 `tracing::warn` 日志，包含 `entity_id`、`command_name`、等待时长。

### 死锁规避

- 每个命令至多获取一把 entity mutex。禁止同一命令嵌套获取多个实体的 mutex。
- 需要跨实体操作时（如删除 Theme Card 连带 Sessions），通过顺序编排拆为多个单锁步骤，不持有多锁。
- 删除操作涉及活跃流时，先取消流（释放 session mutex），再获取实体 mutex 执行删除。

### 清理

- 命令完成（成功或错误）后必须释放 mutex（通过 Rust RAII `MutexGuard` drop 保证）。
- 不在 `HashMap` 中无限积累已不再活跃的实体锁；可定期清理无引用的条目，或使用 `DashMap` + weak reference 模式。

## 取消、超时与背压

- 取消是 Harness 一等语义，不能依赖”页面卸载后自然丢弃结果”这种隐式行为。
- 取消后的结果必须转成稳定错误码或稳定终止语义，不得透出供应商私有取消结构。
- 非流式命令若超过预期预算，应记录慢命令日志；是否向用户提示由调用场景决定。
- 流式通道不得使用无界内存积压；慢消费者必须进入可诊断的背压处理路径。

### Channel 背压参考值

| 参数           | 参考值 | 说明                                                                                                |
| -------------- | ------ | --------------------------------------------------------------------------------------------------- |
| Channel 缓冲区 | 128 条 | Tauri Channel 底层有界队列大小；超出后 `channel.send()` 进入背压等待                                |
| 慢消费者超时   | 30 秒  | 背压等待超过此阈值，视为消费者失联                                                                  |
| 超时后行为     | —      | 记录 `tracing::warn` 日志（含 `session_id`、已等待时长），关闭流并发出 `Error(CLIENT_DISCONNECTED)` |

- 上述为参考值，实现时可根据实际基准测试微调，但必须保持有界且有超时。
- 缓冲区大小与超时阈值若需变更，须在 commit message 中说明理由。

## 错误包络与诊断

- 所有跨 IPC 失败都必须收敛到 Mirage 自己的四类错误：`Domain` / `Infra` / `Security` / `Gateway`。
- `correlation_id` 由 Rust 侧生成并贯穿命令日志、事件日志与错误响应，前端只负责透传与展示调试信息。
- 原始供应商错误、原始敏感载荷、内部堆栈不得直接暴露给最终用户。

### 结构化错误最小字段集（MH-31）

错误响应必须包含以下字段，不得遗漏：

| 字段             | 类型    | 必需 | 说明                                                  |
| ---------------- | ------- | ---- | ----------------------------------------------------- |
| `category`       | string  | 是   | 四类之一：`Domain` / `Infra` / `Security` / `Gateway` |
| `error_code`     | string  | 是   | `UPPER_SNAKE_CASE`，如 `THEME_CARD_NOT_FOUND`         |
| `message`        | string  | 是   | 面向开发者的可读描述，不含敏感信息                    |
| `retryable`      | boolean | 是   | 调用方是否可安全重试                                  |
| `correlation_id` | string  | 是   | 本次调用的唯一关联标识，格式为 UUID v4                |

以下字段可选，在调试与诊断场景中推荐携带：

| 字段              | 类型        | 说明                                                    |
| ----------------- | ----------- | ------------------------------------------------------- |
| `detail`          | string/null | 补充上下文（如"字段 X 格式不合法"），不含堆栈或敏感信息 |
| `timestamp`       | string/null | ISO 8601 格式，Rust 侧生成                              |
| `source_location` | string/null | 仅 debug 构建包含，格式 `module::function:line`         |

前端 Zod schema 与 Rust `IpcError` struct 必须同时覆盖上述必需字段，使用 `satisfies z.ZodType<IpcError>` 保持对齐。

## 回放与证据

- 下列流程必须可通过 fixture、日志或集成测试回放：
  - 应用级迁移。
  - `ThemeCard` 实体级迁移。
  - 网关标准化失败。
  - 安全硬拒绝。
  - 同一卡片多 Session 隔离。
- 回放证据至少要能回答：调用了什么、作用于哪个实体、耗时多久、失败在哪一层、错误码是什么。

## 变更门禁

- 新增或修改公开命令、事件名、流式事件类型、错误码、持久化 schema 时，必须同步更新：
  - `HARNESS.md`
  - `COMPATIBILITY.md`
  - `TEST_STRATEGY.md`
- 若改动触及 HR / MH 级约束，还必须按 `PLANS.md` 和 `docs/exec-plans/` 走 ExecPlan。

## 流式事件投递语义

以下规则约束 `invoke_llm_generation` 通过 Tauri Channel 推送的事件序列：

### 事件顺序保证

- 同一次 `invoke_llm_generation` 调用产生的事件，必须按 Rust 侧 `channel.send()` 的调用顺序到达前端。
- 不允许乱序投递；Tauri Channel 底层保证 FIFO，实现层不得引入并发 send。

### 去重与幂等

- 同一流内不保证去重。如果 gateway 返回重复内容，Rust 侧应在归一化阶段过滤；但前端消费侧仍应容忍重复 `TokenChunk` 的防御处理。
- `Completion` 和 `Error` 事件在同一流内至多各出现一次，二者互斥。

### 终止后禁止再发

- 一旦发出 `Completion` 或 `Error`，该流即视为已终止。
- 终止后 Rust 侧禁止再向同一 Channel 发送任何事件。前端收到终止事件后应注销监听器。

### 晚注册监听器

- 不提供事件补发机制。前端必须在 `invoke` 之前完成 Channel 注册。
- 若监听器晚于首个 `TokenChunk` 注册，丢失的事件不可恢复。

### 取消语义

- 用户取消生成时，Rust 侧应尽快终止上游请求并发出 `Error` 事件，`error_code` 为 `GENERATION_CANCELLED`。
- 取消不使用单独的终止事件类型——统一走 `Error`，通过 `error_code` 区分。

## 命令幂等性与重复提交

### 幂等性分类

| 命令类型                | 幂等性   | 说明                                            |
| ----------------------- | -------- | ----------------------------------------------- |
| 读命令                  | 天然幂等 | 无副作用                                        |
| `create_theme_card`     | 非幂等   | 每次调用创建新实体；需前端防重复提交            |
| `update_theme_card`     | 条件幂等 | 相同 payload 重复提交结果一致，但不应依赖此特性 |
| `create_session`        | 非幂等   | 每次调用创建新 Session；需前端防重复提交        |
| `switch_session`        | 幂等     | 切换到相同 session_id 无副作用                  |
| `run_memory_retrieval`  | 幂等     | 相同输入产生相同检索结果                        |
| `invoke_llm_generation` | 非幂等   | 每次调用产生新的 LLM 交互                       |

### 防重复提交策略

- 前端对非幂等变更命令必须实现 UI 级防重复提交（如按钮 disable、debounce）。
- Rust 侧不引入 `client_request_id` 去重机制（v1 不需要）；防线在前端 UI 层。
- `invoke_llm_generation` 已有 per-session 单流约束（MH-26），天然阻止同 Session 重复提交。

## AppReady 后端门禁

- Rust 侧在 `AppReady` 发出前，所有业务命令（UseCaseCommand / ResourceCrudCommand）必须返回稳定错误码 `APP_NOT_READY`，类别为 `InfraError`，`retryable: true`。
- 此门禁在 Rust 命令处理层统一拦截，不依赖前端自觉。
- 前端的 loading 态是 UX 优化，不是安全边界；真正的阻断在后端。

## 取消、超时与断连分类矩阵

| 场景              | 触发方        | error_code             | 类别           | retryable | 记为失败 | 说明                         |
| ----------------- | ------------- | ---------------------- | -------------- | --------- | -------- | ---------------------------- |
| 用户主动取消      | 前端 → Rust   | `GENERATION_CANCELLED` | `DomainError`  | false     | 否       | 用户意图，非故障             |
| 前端断开/页面卸载 | Rust 检测断连 | `CLIENT_DISCONNECTED`  | `InfraError`   | false     | 是       | 记录日志，清理上游请求       |
| 上游供应商超时    | gateway 层    | `GATEWAY_TIMEOUT`      | `GatewayError` | true      | 是       | 前端可提示用户重试           |
| 本地命令超时      | command 层    | `COMMAND_TIMEOUT`      | `InfraError`   | true      | 是       | 非流式命令超过性能预算时触发 |
| 应用关闭/进程终止 | 系统信号      | `SHUTDOWN_ABORT`       | `InfraError`   | false     | 否       | 尽力清理，不保证事件到达     |

### 规则

- 所有取消/超时/断连场景必须最终收敛为上表中的 `error_code`，不得透出供应商私有取消结构。
- `retryable: true` 的场景，前端可向用户展示重试选项，但不得自动重试变更命令。
- 用户主动取消不计入错误统计和告警阈值。
- `CLIENT_DISCONNECTED` 和 `SHUTDOWN_ABORT` 场景下，Rust 侧应尽快取消上游请求，释放 mutex。

## 跨命令并发冲突矩阵

以下矩阵定义同一实体（同一 `theme_card_id` 或 `session_id`）上不同命令的并发行为：

| 正在执行 ↓ \ 新请求 →                 | 读命令 | 变更命令           | invoke_llm_generation | delete（实体） |
| ------------------------------------- | ------ | ------------------ | --------------------- | -------------- |
| 读命令                                | 允许   | 允许（mutex 排队） | 允许                  | 允许（排队）   |
| 变更命令（同实体）                    | 允许   | 排队（per-entity） | 排队                  | 排队           |
| `invoke_llm_generation`（同 Session） | 允许   | 排队               | 拒绝（MH-26）         | 必须先取消流   |
| delete（同实体）                      | 允许   | 排队               | 必须先取消流          | 排队           |

### 冲突规则

- **生成中不可删除**：删除 Theme Card 或 Session 时，若该实体下存在活跃 LLM 生成流，命令层必须先取消流，待流终止后再执行删除。不得返回"请稍后重试"。
- **生成中可切换 Session**：`switch_session` 仅切换前端上下文指针，不影响后台进行中的生成流。但切换后前端应断开对旧 Session 流的消费。
- **检索与生成可并发**：`run_memory_retrieval` 与 `invoke_llm_generation` 作用于不同资源，允许并发。若检索结果晚于生成启动，不影响当次生成。
- **ResourceCrudCommand 不阻塞 UseCaseCommand**：配置读写与业务命令之间无冲突，除非操作同一实体。

## 状态约定

- 本文件中的规则分为两类：
  - 已生效工程规则：当前仓库工作流、文档门禁、公开语义约束。
  - 落地即生效规则：当相关能力开始实现时，必须同时满足的 Harness 要求。
- 当前仓库仍处于模板期；凡涉及 `Theme Card` / `Session` / `Memory Retrieval Layer` / `LLM Gateway` 的执行语义，均属于“落地即生效规则”。
