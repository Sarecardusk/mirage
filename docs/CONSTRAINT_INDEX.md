# 约束索引

> Last Updated: 2026-04-05

本文件是导航性索引，不定义约束。
所有约束的权威定义在各来源文档中维护；本索引仅做提取、去重、分级与执行信息汇总，便于一览全貌。

**冲突裁决**：HR > MH > RC > TD。同级冲突以安全/数据完整性优先，余由 owner 裁定。

## 权威来源

- `DESIGN.md` -- 系统边界、模块职责、IPC 分层与稳定接口约束。
- `RELIABILITY.md` -- 数据一致性、迁移、失败路径与降级策略。
- `SECURITY.md` -- 安全模型、密钥策略、硬拒绝规则。
- `FRONTEND.md` -- UI Host 可做/不可做边界与当前期 UI 技术基线约定。
- `HARNESS.md` -- Harness 工程执行语义、取消/超时/背压、公开面变更门禁。
- `COMPATIBILITY.md` -- 公开契约兼容性与 breaking change 定义。
- `TEST_STRATEGY.md` -- 测试层次、回归矩阵与约束证据策略。
- `OPERATIONS.md` -- 关联 ID、结构化日志、脱敏与诊断要求。
- `PRODUCT_SENSE.md` -- 产品取舍原则与实现边界的产品理由。
- `QUALITY_SCORE.md` -- 边界向验收标准与评审清单。

## 执行列说明

- `Owner`：该约束的主责维护方。
- `Enforcement`：`RT` 运行时阻断，`AT` 自动化测试，`PR` 代码审查，`DOC` 文档门禁。
- `Evidence`：当前或落地后应提供的最小证据类型。
- `Waiver`：临时例外路径；`禁止` 表示除非删改约束，否则不得豁免。
- `Status`：`Planned`（目标态，尚无实现）/ `Implemented`（已有实现与证据）。
- `Evidence Link`：指向测试文件、CI 日志或审查记录的路径/URL；Planned 状态填 `—`。
- `Last Verified`：最近一次人工或自动验证日期；Planned 状态填 `—`。
- 脚手架阶段 Implemented 约束的 Evidence Link 可为文档引用或代码审查记录；首个相关功能合并后，应升级为具体测试文件路径（如 `src/schemas/themeCard.test.ts`）。

## Hard Reject（HR）-- 系统必须拒绝，不可覆盖

| ID    | 约束                                                                        | 来源                   | Owner                | Enforcement   | Evidence              | Waiver            | Status      | Evidence Link              | Last Verified |
| ----- | --------------------------------------------------------------------------- | ---------------------- | -------------------- | ------------- | --------------------- | ----------------- | ----------- | -------------------------- | ------------- |
| HR-01 | 未配对设备必须硬拒绝                                                        | SECURITY.md            | Rust Core            | RT + AT       | 安全拒绝集成测试      | 禁止              | Planned     | —                          | —             |
| HR-02 | 口令错误必须硬拒绝，不得明文 fallback                                       | SECURITY.md            | Rust Core            | RT + AT       | Vault/口令失败测试    | 禁止              | Planned     | —                          | —             |
| HR-03 | 签名校验失败必须硬拒绝                                                      | SECURITY.md            | Rust Core            | RT + AT       | 安全拒绝集成测试      | 禁止              | Planned     | —                          | —             |
| HR-04 | 禁止引入可执行同步流程（v1）                                                | DESIGN.md, SECURITY.md | 架构 Owner           | PR + DOC + AT | 无同步入口回归用例    | 仅专项评审改约束  | Implemented | 代码审查（无同步代码路径） | 2026-04-05    |
| HR-05 | 迁移失败禁止 silent fallback                                                | RELIABILITY.md         | Rust Core            | RT + AT       | 迁移失败阻断测试      | 禁止              | Planned     | —                          | —             |
| HR-06 | 删除资产不得产生悬挂引用                                                    | RELIABILITY.md         | Rust Core            | RT + AT       | 资产一致性测试        | 禁止              | Planned     | —                          | —             |
| HR-07 | UI 不得绕过领域命令直接写关键对象                                           | DESIGN.md, FRONTEND.md | Frontend + Rust Core | PR + AT       | 前端服务/命令边界测试 | 仅专项评审改约束  | Implemented | 代码审查（无直接写路径）   | 2026-04-05    |
| HR-08 | 不建设云端中转 LLM Gateway                                                  | DESIGN.md              | 架构 Owner           | DOC + PR      | 设计审查记录          | 仅产品/架构重定向 | Implemented | DESIGN.md 非目标声明       | 2026-04-05    |
| HR-09 | 不引入云账号中心                                                            | DESIGN.md              | 架构 Owner           | DOC + PR      | 设计审查记录          | 仅产品/架构重定向 | Implemented | DESIGN.md 非目标声明       | 2026-04-05    |
| HR-10 | API Key、口令、密钥、签名原文、未脱敏敏感输入禁止进入日志/事件/错误 message | OPERATIONS.md          | Rust Core            | RT + AT       | 脱敏合规测试          | 禁止              | Planned     | —                          | —             |

## Must-Have（MH）-- 验收必备

| ID    | 约束                                                                                                     | 来源                        | Owner                | Enforcement   | Evidence                     | Waiver                 | Status      | Evidence Link                | Last Verified |
| ----- | -------------------------------------------------------------------------------------------------------- | --------------------------- | -------------------- | ------------- | ---------------------------- | ---------------------- | ----------- | ---------------------------- | ------------- |
| MH-01 | Rust 类型系统是跨边界契约真源                                                                            | DESIGN.md                   | Rust Core            | PR + AT       | 命令契约测试 + 类型导出检查  | 仅 ExecPlan + 文档更新 | Planned     | —                            | —             |
| MH-02 | ThemeCard 必带 schemaVersion                                                                             | DESIGN.md, RELIABILITY.md   | Rust Core            | RT + AT       | schema/迁移测试              | 仅 ExecPlan + 迁移方案 | Planned     | —                            | —             |
| MH-03 | 破坏性字段变更必须附带显式迁移                                                                           | RELIABILITY.md              | Rust Core            | DOC + AT      | 迁移回放测试                 | 仅 ExecPlan + 迁移证据 | Planned     | —                            | —             |
| MH-04 | 同卡不同 Session 消息历史独立                                                                            | RELIABILITY.md              | Rust Core            | AT            | Session 隔离测试             | 仅临时缺口登记         | Planned     | —                            | —             |
| MH-05 | 同卡不同 Session 检索记忆上下文独立                                                                      | RELIABILITY.md              | Rust Core            | AT            | Session 隔离测试             | 仅临时缺口登记         | Planned     | —                            | —             |
| MH-06 | 卡片共享设定在该卡全部 Session 间复用                                                                    | RELIABILITY.md              | Rust Core            | AT            | ThemeCard/Session 作用域测试 | 仅临时缺口登记         | Planned     | —                            | —             |
| MH-07 | TS 边界手写 Zod schema 并与 Rust 类型对齐                                                                | DESIGN.md, FRONTEND.md      | Frontend             | PR + AT       | schema 测试 + 服务测试       | 仅 ExecPlan + 补测计划 | Planned     | —                            | —             |
| MH-08 | 敏感输入必须在 Rust 侧叠加 validator 校验                                                                | DESIGN.md, SECURITY.md      | Rust Core            | RT + AT       | 非法输入测试                 | 禁止在实现后无证据合并 | Planned     | —                            | —             |
| MH-09 | 检索失败必须可展示，不得崩溃                                                                             | DESIGN.md                   | Rust Core + Frontend | AT            | 降级路径测试                 | 仅临时缺口登记         | Planned     | —                            | —             |
| MH-10 | 模型失败返回标准化错误 + retryable 标识                                                                  | RELIABILITY.md, FRONTEND.md | Rust Core            | AT            | gateway/command 错误测试     | 仅临时缺口登记         | Planned     | —                            | —             |
| MH-11 | 归一化失败视为网关内部错误，不透出供应商结构                                                             | RELIABILITY.md, FRONTEND.md | Rust Core            | RT + AT       | 网关异常样本测试             | 禁止在实现后无证据合并 | Planned     | —                            | —             |
| MH-12 | 元数据写入失败必须回滚 + 返回可重试错误                                                                  | RELIABILITY.md              | Rust Core            | RT + AT       | 存储失败回滚测试             | 仅临时缺口登记         | Planned     | —                            | —             |
| MH-13 | 资产写入失败不落悬挂引用 + 返回资源错误码                                                                | RELIABILITY.md              | Rust Core            | RT + AT       | 资产失败测试                 | 仅临时缺口登记         | Planned     | —                            | —             |
| MH-14 | IPC 分层：UseCaseCommand / ResourceCrudCommand / RuntimeEvent                                            | DESIGN.md                   | 架构 Owner           | DOC + PR + AT | 命令分层检查 + 集成测试      | 仅 ExecPlan + 文档更新 | Planned     | —                            | —             |
| MH-15 | 六个最小 UseCase 集合                                                                                    | DESIGN.md                   | 架构 Owner           | DOC + PR      | 设计审查记录                 | 仅产品/架构评审        | Implemented | DESIGN.md UseCase 最小集合   | 2026-04-05    |
| MH-16 | 五个稳定接口名                                                                                           | DESIGN.md                   | Rust Core            | DOC + PR      | 设计审查记录                 | 仅 ExecPlan + 文档更新 | Implemented | DESIGN.md 稳定接口名         | 2026-04-05    |
| MH-17 | UI 默认 TailwindCSS + shadcn-vue，偏离需评审                                                             | FRONTEND.md                 | Frontend             | DOC + PR      | 设计文档记录                 | 允许评审后偏离         | Implemented | FRONTEND.md 技术基线         | 2026-04-05    |
| MH-18 | 前端不得在 UI 侧实现 provider 特判逻辑                                                                   | FRONTEND.md                 | Frontend             | PR + AT       | 服务层测试 + 代码审查        | 仅 ExecPlan + 文档更新 | Implemented | 代码审查（无 provider 特判） | 2026-04-05    |
| MH-19 | 前端不得复制领域规则作为主判定路径                                                                       | FRONTEND.md                 | Frontend             | PR + AT       | 服务/视图边界测试            | 仅 ExecPlan + 文档更新 | Implemented | 代码审查（无领域规则复制）   | 2026-04-05    |
| MH-20 | 新建会话创建新 session_id 绑定当前 theme_card_id                                                         | FRONTEND.md                 | Rust Core + Frontend | AT            | create_session 测试          | 仅临时缺口登记         | Planned     | —                            | —             |
| MH-21 | 切换会话仅切换会话态，不重写 ThemeCard 共享设定                                                          | FRONTEND.md                 | Rust Core + Frontend | AT            | switch_session 测试          | 仅临时缺口登记         | Planned     | —                            | —             |
| MH-22 | 错误必须归入四类：Domain / Infra / Security / Gateway                                                    | DESIGN.md                   | Rust Core            | RT + AT       | 命令错误包络测试             | 仅 ExecPlan + 文档更新 | Planned     | —                            | —             |
| MH-23 | 同一实体变更命令串行化（per-entity mutex，实现规范见 HARNESS.md）                                        | DESIGN.md, HARNESS.md       | Rust Core            | RT + AT       | 并发命令测试                 | 仅专项评审改约束       | Planned     | —                            | —             |
| MH-24 | 应用级迁移失败阻塞启动                                                                                   | RELIABILITY.md              | Rust Core            | RT + AT       | 启动迁移失败测试             | 禁止                   | Planned     | —                            | —             |
| MH-25 | 启动序列五步完成后 emit `AppReady`，前端在此前显示 loading                                               | DESIGN.md, ARCHITECTURE.md  | Rust Core + Frontend | RT + AT       | 启动序列测试                 | 仅临时缺口登记         | Planned     | —                            | —             |
| MH-26 | 同一 session_id 同时刻仅允许一个活跃 LLM 生成流                                                          | HARNESS.md                  | Rust Core            | RT + AT       | 流式并发测试                 | 仅专项设计文档放宽     | Planned     | —                            | —             |
| MH-27 | 每次命令调用必须以成功或结构化错误结束，不允许悬空等待                                                   | HARNESS.md                  | Rust Core            | RT + AT       | 命令终止测试                 | 禁止                   | Planned     | —                            | —             |
| MH-28 | 流式命令必须以 Completion 或 Error 终止，二者互斥                                                        | HARNESS.md                  | Rust Core            | RT + AT       | 流式终止语义测试             | 禁止                   | Planned     | —                            | —             |
| MH-29 | 取消是一等语义，不得依赖隐式丢弃                                                                         | HARNESS.md                  | Rust Core            | RT + AT       | 取消路径测试                 | 仅 ExecPlan + 文档更新 | Planned     | —                            | —             |
| MH-30 | 流式通道不得使用无界内存积压（参考值见 HARNESS.md 背压章节）                                             | HARNESS.md                  | Rust Core            | RT + AT       | 背压路径测试                 | 仅 ExecPlan + 文档更新 | Planned     | —                            | —             |
| MH-31 | 错误响应至少包含 category / error_code / message / retryable / correlation_id（完整字段集见 HARNESS.md） | HARNESS.md                  | Rust Core            | RT + AT       | 错误包络字段测试             | 仅 ExecPlan + 文档更新 | Planned     | —                            | —             |

## Recommended（RC）-- 稳定性条目，允许缺口但须附修复计划

| ID    | 约束                                  | 来源                             | Owner                | Enforcement | Evidence                | Waiver                                                                   | Status  | Evidence Link | Last Verified |
| ----- | ------------------------------------- | -------------------------------- | -------------------- | ----------- | ----------------------- | ------------------------------------------------------------------------ | ------- | ------------- | ------------- |
| RC-01 | Zod schema 与 Rust 导出类型一致性验证 | QUALITY_SCORE.md                 | Frontend             | AT          | schema 对齐测试         | 允许阶段性缺口，需补测计划                                               | Planned | —             | —             |
| RC-02 | 错误返回包含定位信息便于追踪          | QUALITY_SCORE.md, OPERATIONS.md  | Rust Core            | RT + AT     | correlation_id/错误测试 | 允许阶段性缺口，需补测计划。注：MH-31 已将 correlation_id 提升为必选字段 | Planned | —             | —             |
| RC-03 | 会话级配置不污染同卡其他 Session      | QUALITY_SCORE.md, RELIABILITY.md | Rust Core + Frontend | AT          | Session 配置隔离测试    | 允许阶段性缺口，需补测计划                                               | Planned | —             | —             |

## Technology Decision（TD）-- 技术选型决策，变更需文档更新与评审

| ID    | 约束                                                                        | 来源        | Owner                | Enforcement   | Evidence               | Waiver                 | Status      | Evidence Link               | Last Verified |
| ----- | --------------------------------------------------------------------------- | ----------- | -------------------- | ------------- | ---------------------- | ---------------------- | ----------- | --------------------------- | ------------- |
| TD-01 | 错误码 Rust/TS 两侧手动维护，保持一一对应                                   | DESIGN.md   | Rust Core + Frontend | DOC + PR + AT | 错误码映射检查         | 仅 ExecPlan + 文档更新 | Planned     | —                           | —             |
| TD-02 | 前端 Locked 级选型变更需文档更新 + 评审                                     | FRONTEND.md | Frontend             | DOC + PR      | 设计文档记录           | 允许评审后变更         | Implemented | FRONTEND.md 选型表          | 2026-04-05    |
| TD-03 | Rust 架构遵循 domain/infra/gateway/command 分层，domain 无外向依赖          | DESIGN.md   | Rust Core            | PR + AT       | 分层测试/代码审查      | 仅 ExecPlan + 文档更新 | Implemented | DESIGN.md 架构分层          | 2026-04-05    |
| TD-04 | 模块入口使用 moduleName.ts/.rs 模式，不用 index.ts/mod.rs                   | AGENTS.md   | 全体贡献者           | PR            | 代码审查               | 允许评审后例外         | Implemented | AGENTS.md 工作约定          | 2026-04-05    |
| TD-05 | LLM 流式响应用 Tauri Channel API，全局通知用 Tauri Event System             | DESIGN.md   | Rust Core + Frontend | DOC + PR + AT | 流式/事件测试          | 仅 ExecPlan + 文档更新 | Implemented | DESIGN.md IPC 通信模式      | 2026-04-05    |
| TD-06 | Pinia stores 仅持有 UI 状态与读缓存，领域数据真源在 Rust/SurrealDB          | FRONTEND.md | Frontend             | PR + AT       | store/service 边界测试 | 仅 ExecPlan + 文档更新 | Implemented | FRONTEND.md 状态管理        | 2026-04-05    |
| TD-07 | Tauri Capability 遵循最小权限，新增插件同步更新 `capabilities/default.json` | SECURITY.md | Rust Core            | DOC + PR      | capability 变更审查    | 禁止绕过               | Implemented | `capabilities/default.json` | 2026-04-05    |
| TD-08 | 生产环境 CSP 必须为 `'self'` + 必需 API 域名，dev 下可为 null               | SECURITY.md | Rust Core            | DOC + PR + RT | 发布检查单             | 仅发布评审例外         | Implemented | `tauri.conf.json` CSP 配置  | 2026-04-05    |
| TD-09 | SFC 内部代码顺序遵循八段排列约定                                            | FRONTEND.md | Frontend             | PR            | 代码审查               | 允许评审后例外         | Implemented | FRONTEND.md 八段约定        | 2026-04-05    |
