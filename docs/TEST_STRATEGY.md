# Mirage Test Strategy

> Last Updated: 2026-04-17

本文件定义 Mirage 约束的验证策略。
目标不是“多写测试”，而是让每个高成本约束都有证据。

## 验证原则

- HR 级约束必须有运行时阻断或自动化测试证据。
- MH 级约束在能力落地后必须有自动化证据，不接受只靠人工记忆维持。
- RC 级约束允许阶段性缺口，但必须能在评审中指出缺口与修复计划。
- TD 级约束至少要有文档门禁与代码审查证据。

## 测试分层

### Rust 侧

- `domain/`：纯单元测试，覆盖实体规则、状态机、迁移函数。
- `infra/`：一致性与失败路径测试，覆盖存储失败、资产失败、回滚、悬挂引用。
- `gateway/`：供应商响应归一化测试，覆盖标准成功、可重试失败、不可重试失败、格式异常。
  - 对 OpenAI-compatible endpoint 归一化至少覆盖 base URL、`/chat/completions`、`/models` 三种输入形式。
- `command/`：契约集成测试，覆盖命令入参、错误收敛、per-entity mutex、流式终止语义。

### Frontend 侧

- `schemas/`：Zod schema 对齐测试，覆盖必填字段、可选字段、错误收敛。
- `services/`：IPC 调用封装测试，覆盖成功返回、结构化错误、流式消费、监听器注销。
  - LLM 配置链路至少覆盖：保存配置、空 `model` 仍可获取模型列表、测试连接仍要求 `model`。
- `composables/`：状态流转测试，覆盖 loading、retryable、degraded、cancelled。

### Zod ↔ Rust 对齐测试要求

每个手写 Zod schema 必须附带 round-trip 测试（RC-01 证据要求），覆盖：

1. **正向解析**：使用 Rust 侧 fixture 样本（或等价 JSON），验证 `schema.parse()` 成功且字段完整。
2. **必填字段缺失**：移除每个必填字段，验证 `parse()` 抛出明确错误。
3. **可选字段缺省**：省略可选字段，验证 `parse()` 成功且默认值符合预期。
4. **错误包络对齐**：使用 `IpcError` schema 解析 Rust 侧错误 fixture，验证 `category`、`error_code`、`retryable`、`correlation_id` 全部可提取。

测试文件放在对应 schema 同目录（如 `src/schemas/themeCard.test.ts`），命名遵循 `*.test.ts` 约定。

> 此要求在 `specta-zod` 自动管线可用前（DEBT-001）始终生效，作为手动维护的安全网。

### 对齐验证流程

由于 `satisfies z.ZodType<T>` 仅在编译时检查结构兼容性，以下语义差异需通过 fixture 测试捕获：

- **数值精度**：Rust `u32` 在 JSON 中为整数，但 `z.number()` 默认接受浮点数。应使用 `z.number().int().nonnegative()`。
- **可选性**：Rust `Option<T>` 序列化为 `null` 或缺失字段，Zod 侧需用 `z.xxx().nullable().optional()` 匹配。
- **枚举变体**：Rust `#[serde(tag = "type")]` 的标签式枚举需在 Zod 侧用 `z.discriminatedUnion()` 对应。
- **字段命名**：`#[serde(rename_all = "camelCase")]` 会将 `schema_version` 序列化为 `schemaVersion`，Zod 字段名必须匹配序列化后的形式。

**Fixture 来源**：手动创建匹配 Rust `serde_json::to_value()` 输出的 JSON 文件，或从 `cargo test` 中 `println!` 序列化结果后复制。

**Fixture 存放位置**：`src/schemas/__fixtures__/`，命名为 `{entity}.{scenario}.json`。

示例目录结构：

```
src/schemas/__fixtures__/
├── themeCard.valid.json
├── themeCard.v1_legacy.json
├── themeCard.missing_required.json
└── ipcError.domain_validation.json
```

**覆盖范围**：所有使用 `satisfies z.ZodType<T>` 约束的 Zod schema 必须有对应的 fixture 测试文件。

## 最小回归矩阵

| 区域         | 必测场景                                           | 证据类型                    |
| ------------ | -------------------------------------------------- | --------------------------- |
| 迁移         | 旧版 `ThemeCard` 升级成功；迁移失败阻断启动        | Rust 集成测试               |
| Session 隔离 | 同卡不同 Session 的消息与检索上下文互不串扰        | Rust 单元/集成测试          |
| 资产一致性   | 资产写入失败不留悬挂引用；删除前校验引用           | Rust 集成测试               |
| 检索降级     | 检索失败仍可进入“无检索上下文”路径                 | 命令测试 + 前端服务测试     |
| 网关错误     | 模型失败返回标准化错误与 `retryable`               | gateway 测试 + command 测试 |
| 安全拒绝     | 未配对、口令错误、签名失败全部硬拒绝               | 安全集成测试                |
| 流式生成     | `TokenChunk` / `Completion` / `Error` 终止语义稳定 | 流式序列测试                |
| 前端边界     | 所有结构化 payload 经 Zod 校验                     | schema/service 测试         |
| 启动序列     | `AppReady` 前禁止业务命令；完成后发出事件          | 启动集成测试                |

## Fixture 策略

以下样本一旦引入实现，就应固定为长期回归资产：

- 历史版本 `ThemeCard` 样本。
- 非法敏感输入样本。
- 供应商原始成功/失败响应样本。
- 损坏的 vault / 错误口令样本。
- 资产引用不一致样本。

fixture 应优先覆盖”历史兼容””坏输入””失败降级”三类场景，而不是只覆盖 happy path。

### Fixture 存放约定

| 范围          | 目录                        | 命名格式                   | 格式 |
| ------------- | --------------------------- | -------------------------- | ---- |
| Rust 侧       | `src-tauri/tests/fixtures/` | `{entity}.{scenario}.json` | JSON |
| 前端 Zod 对齐 | `src/schemas/__fixtures__/` | `{entity}.{scenario}.json` | JSON |

- fixture 文件为 JSON 格式，内容应与 Rust `serde_json::to_value()` 的输出一致。
- 场景命名示例：`valid`、`v1_legacy`、`missing_required`、`corrupted`、`empty_response`。
- 新增实体时，至少创建 `{entity}.valid.json` 和 `{entity}.missing_required.json` 两个 fixture。

## 门禁命令

当前本地最小门禁固定为：

- `pnpm check`
- `pnpm lint`
- `pnpm test`

随着业务代码落地，应把新增测试接入以上总命令，而不是放成额外的手工脚本。

## Harness 边界回归用例

以下用例覆盖 Harness 工程约束中高返工成本的边界场景，在对应能力落地时必须同步实现。

### 重复提交与幂等

| 用例 ID | 场景                                        | 预期行为                                           | 证据类型         |
| ------- | ------------------------------------------- | -------------------------------------------------- | ---------------- |
| HB-01   | 用户快速双击 `create_theme_card`            | 创建两张独立卡片（前端应防重复提交，但后端不拒绝） | command 集成测试 |
| HB-02   | 同一 payload 重复调用 `update_theme_card`   | 第二次调用成功但无实际变更                         | command 单元测试 |
| HB-03   | 同 Session 并发调用 `invoke_llm_generation` | 第二次调用返回 `GENERATION_CONFLICT` 错误          | command 集成测试 |

### 晚到事件与监听器时序

| 用例 ID | 场景                                                 | 预期行为                                     | 证据类型              |
| ------- | ---------------------------------------------------- | -------------------------------------------- | --------------------- |
| HB-04   | 前端在 `invoke` 后才注册 Channel 监听器              | 丢失的 `TokenChunk` 不可恢复，流仍能正常终止 | 前端服务测试          |
| HB-05   | 流终止后前端尝试继续消费 Channel                     | 无新事件到达，监听器空转后注销               | 前端服务测试          |
| HB-06   | `Completion` 到达后 Rust 侧尝试再次 `channel.send()` | 不发送（实现层保证终止后不再推送）           | Rust command 单元测试 |

### 取消与超时

| 用例 ID | 场景                            | 预期行为                                                                  | 证据类型                 |
| ------- | ------------------------------- | ------------------------------------------------------------------------- | ------------------------ |
| HB-07   | 用户在生成中点击取消            | Rust 终止上游请求，发出 `Error(GENERATION_CANCELLED)`，释放 session mutex | command 集成测试         |
| HB-08   | 前端页面卸载时仍有活跃生成流    | Rust 检测断连，发出 `CLIENT_DISCONNECTED`，清理资源                       | command 集成测试         |
| HB-09   | 上游供应商响应超时              | gateway 返回 `GATEWAY_TIMEOUT`（retryable），command 层转为标准错误       | gateway + command 测试   |
| HB-10   | 非流式命令超过性能预算（100ms） | 记录慢命令日志，不中断执行                                                | Rust 集成测试 + 日志验证 |
| HB-11   | 取消后用户立即重新发起生成      | 旧流完全终止后新流正常启动，不出现 mutex 死锁                             | command 集成测试         |

### 背压与慢消费

| 用例 ID | 场景                             | 预期行为                                    | 证据类型                 |
| ------- | -------------------------------- | ------------------------------------------- | ------------------------ |
| HB-12   | 前端消费速度远慢于 Rust 推送速度 | Channel 有界缓冲区生效，Rust 侧进入背压等待 | command 集成测试         |
| HB-13   | 背压持续超过阈值                 | 记录诊断日志，不崩溃不丢数据                | Rust 集成测试 + 日志验证 |

### 启动前拒绝

| 用例 ID | 场景                                      | 预期行为                                            | 证据类型         |
| ------- | ----------------------------------------- | --------------------------------------------------- | ---------------- |
| HB-14   | `AppReady` 前前端调用 `create_theme_card` | 返回 `APP_NOT_READY`（InfraError, retryable: true） | command 集成测试 |
| HB-15   | `AppReady` 前前端调用读命令               | 同样返回 `APP_NOT_READY`                            | command 集成测试 |
| HB-16   | 迁移失败后前端调用业务命令                | 应用未进入 `AppReady`，同 HB-14                     | 启动序列集成测试 |

### 并发冲突

| 用例 ID | 场景                                                                    | 预期行为                                 | 证据类型               |
| ------- | ----------------------------------------------------------------------- | ---------------------------------------- | ---------------------- |
| HB-17   | 生成进行中删除当前 Session                                              | 先取消生成流，流终止后执行删除           | command 集成测试       |
| HB-18   | 生成进行中删除 Theme Card                                               | 先取消该卡下所有活跃流，再执行删除       | command 集成测试       |
| HB-19   | 同一 Theme Card 同时执行 `update_theme_card` 和 `invoke_llm_generation` | per-entity mutex 排队，不死锁            | command 并发测试       |
| HB-20   | 生成进行中 `switch_session` 到另一 Session                              | switch 成功，原 Session 的生成流不受影响 | command + 前端服务测试 |

## 文档到证据映射

- `CONSTRAINT_INDEX.md` 中每条 HR / MH 约束都必须能指向一项证据。
- 若某约束处于 `Planned` 状态，可先指向“待补测试项”，但在实现同轮不得缺证据合并。
