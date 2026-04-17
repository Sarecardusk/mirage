# Mirage 协作指南

> Last Updated: 2026-04-17

此文件只做轻量导航与协作约定。
产品与系统细节请在对应文档中维护。

## 仓库方向

- 项目类型：Tauri 跨平台应用。
- 产品方向：SillyTavern-like AI RolePlay 应用。
- 核心实体：`Theme Card`（不是 `Character Card`）。
- 兼容性立场：**不以**兼容 SillyTavern 角色卡格式为目标。

## 文档导航

```text
AGENTS.md                              # 协作约定与入口导航
ARCHITECTURE.md                        # 系统架构地图与不变量
CHANGELOG.md                           # 项目版本功能更新记录
PLANS.md                               # ExecPlan 元规范
README.md                              # 项目信息说明
docs/
├── README.md                          # docs 总入口
├── CONSTRAINT_INDEX.md                # 全约束导航索引（按执行级别分组）
├── DESIGN.md                          # 设计原则总入口
├── HARNESS.md                         # Harness 工程执行语义与门禁
├── FRONTEND.md                        # 前端边界与约束
├── GARDENING_POLICY.md                # 文档养护策略
├── COMPATIBILITY.md                   # 公开契约兼容性策略
├── TEST_STRATEGY.md                   # 测试证据与回归矩阵
├── OPERATIONS.md                      # 可观测性与诊断约束
├── PRODUCT_SENSE.md                   # 产品取舍原则
├── QUALITY_SCORE.md                   # 质量评分框架
├── RELIABILITY.md                     # 可靠性目标与退化策略
├── SECURITY.md                        # 安全模型与边界
├── design-docs/
│   └── README.md                      # 设计文档索引
├── exec-plans/
│   ├── README.md                      # ExecPlan 使用说明
│   ├── tech-debt-tracker.md           # 技术债跟踪
│   ├── active/                        # 进行中的计划
│   │   └── _template.md               # 新计划模板
│   └── completed/
│       └── _template.md               # 已归档计划模板
├── product-specs/
│   ├── README.md                      # 产品规格索引
│   └── v1.md                          # v1 产品规格
└── references/
    ├── README.md                      # 参考资料索引
    ├── code-patterns.md               # 关键代码模式骨架示例
    └── first-feature-walkthrough.md   # 首个功能端到端实战示例
```

## 当前实现状态

项目已完成 MVP 最简会话循环、SurrealDB 持久化与 Vault 密钥存储集成，具备端到端可用的基础功能。

文档中的约束分两类：

- 当前已生效：协作流程、文档门禁、基础工程链、capability/CSP 开发态、四层架构分层、IPC 契约模式、SurrealDB 持久化。
- 落地即生效：迁移版本管理、检索、流式中断 / 重试等高级业务约束。

**工程链**：Tauri 2 + Vue 3 + TypeScript、Vite 构建、ESLint + Prettier + Vitest、TailwindCSS + shadcn-vue。

**后端四层**（`src-tauri/src/`）：

- `domain/`：ThemeCard、Session、LLM 实体与 Repository trait。
- `infra/`：SurrealDB 持久化实现（ThemeCard / Session / AppConfig）、Vault 存储、数据库初始化与迁移。
- `gateway/`：LLM HTTP 流式网关（reqwest + SSE）。
- `command/`：16 个 IPC 命令（ThemeCard × 5、Session × 5、Config × 5、LLM × 1）。

**前端**（`src/`）：

- `views/`：ThemeCardListView、ChatView、SettingsView（3 个页面）。
- `components/`：AppLayout + shadcn-vue UI 组件库。
- `composables/`：useChat、useLlmStream。
- `services/`：themeCard、session、config、llm（封装 `invoke()` 调用）。
- `schemas/`：themeCard、session、config、llm（Zod 运行时校验）。
- `types/`：bindings.ts（Specta 自动生成）。
- `router.ts`：单文件路由（vue-router）。

**尚未创建的目录**：

- 前端：`stores/`（Pinia store 尚无业务需求）。

**尚未安装的预批准依赖**：

- 前端：`@tauri-store/pinia`。
- 后端：`argon2`、`validator`（安全与校验相关，按需引入）。

## 预批准依赖

以下依赖已在架构文档（`ARCHITECTURE.md`、`FRONTEND.md`）中确定选型，首次安装时无需在 commit message 中额外说明理由。

**前端 dependencies**（`pnpm add`）：
`vue-router`、`pinia`、`@tauri-store/pinia`、`zod`、`vueuse`

**前端 devDependencies**（`pnpm add -D`）：
`tailwindcss`、`@tailwindcss/vite`（shadcn-vue 的初始化会引入额外依赖，视 CLI 输出为准）

**后端 Cargo.toml `[dependencies]`**：
`surrealdb`、`tokio`、`tracing`、`tracing-subscriber`、`reqwest`、`anyhow`、`thiserror`、`ring`、`argon2`、`validator`、`specta`、`tauri-specta`

> 不在此列表中的新运行时依赖仍需在 commit message 中说明理由。

## 工作约定

- 文档与实现术语必须一致。
- 仅在行为不直观时补充注释。
- 设计保持简洁，优先可实现性。
- 默认使用自解释命名，仅必要之处使用注释。
- 任何公开命令、事件、错误码、持久化 schema 的改动，必须同步检查 `HARNESS.md`、`COMPATIBILITY.md`、`TEST_STRATEGY.md`。
- TypeScript 侧的结构化输入/输出、持久化快照、跨边界载荷与配置对象默认使用 `Zod` 定义 schema，并以 schema 作为运行时校验与类型收窄入口。
- 模块入口命名默认避免 `index.ts` / `mod.rs` 风格；优先使用 `modules.ts + modules/`、`modules.rs + modules/` 这类同名入口文件加目录的结构，只有在确有约定收益时才例外。
- 实现新功能前，先查阅 `docs/references/code-patterns.md` 中的骨架示例。

### 决策默认值

在模糊情境下按以下默认规则判断：

| 决策点                 | 默认规则                                                                      |
| ---------------------- | ----------------------------------------------------------------------------- |
| composable vs store    | 需要跨组件共享状态 → store；仅封装单组件可复用逻辑 → composable               |
| domain/ vs infra/      | 纯业务规则、不依赖外部库 → domain；依赖 SurrealDB/文件系统/加密库 → infra     |
| 新文件 vs 扩展现有文件 | 同一实体或功能域 → 扩展现有文件；新实体或新功能域 → 新文件                    |
| ExecPlan 是否需要      | 直接修改 HR/MH 级约束的实现方式 → 需要；新增符合既有模式的命令或组件 → 不需要 |
| 新增 Tauri 插件        | 同步更新 `capabilities/default.json` 添加对应权限                             |
| 是否添加注释/文档      | 仅在行为不直观时添加；不为已有代码补注释或文档                                |
| 是否添加错误处理       | 系统边界（用户输入、外部 API）需要；内部调用信任框架保证                      |
| 是否抽象/封装          | 一次性操作不封装；三行相似代码优于过早抽象                                    |

## 验证命令

每次修改后，根据变更范围运行对应检查：

| 变更范围　　　　 | 命令　　　　　　　　　　　　　　　　 | 说明　　　　　　　　　　　　　　　　　　 |
| ---------------- | ------------------------------------ | ---------------------------------------- |
| TypeScript / Vue | `pnpm check:web`　　　　　　　　　　 | fmt 检查 + ESLint（零警告） + typecheck  |
| Rust　　　　　　 | `pnpm check:rust && pnpm lint:rust`  | cargo check + clippy（-D warnings）　　  |
| 全量　　　　　　 | `pnpm check && pnpm lint`　　　　　  | 双侧检查 + 双侧 lint　　　　　　　　　　 |
| 提交前　　　　　 | `pnpm test`　　　　　　　　　　　　  | 双侧单元测试（vitest + cargo test）　　  |

格式化修复：`pnpm fmt`（双侧）或 `pnpm fmt:web` / `pnpm fmt:rust`（单侧）。

## CI/CD 状态

- 当前无 CI/CD 流水线配置（GitHub Actions 等），所有质量门禁通过本地命令执行。
- 零警告策略：ESLint 不允许 warning（视为 error）、clippy 使用 `-D warnings`。
- 未来引入 CI 时，流水线应复用上述本地验证命令，不引入额外检查逻辑。

## 测试约定

### 前端（Vitest）

- 文件命名：`*.test.ts`（与 `vite.config.ts` 中 `include` 配置一致）。
- 文件位置：与被测文件同目录（如 `src/services/themeCard.test.ts`）。
- Mock IPC：使用 `vi.mock("@tauri-apps/api/core")` mock `invoke` 调用。
- 重点覆盖：Zod schema 校验边界、service 层错误处理、composable 状态流转。

### 后端（cargo test）

- 单元测试：在模块内使用 `#[cfg(test)] mod tests`。
- `domain/` 层：纯单元测试，不依赖外部服务。
- `infra/` / `gateway/` 层：需要外部依赖时通过 trait mock 隔离。
- `command/` 层：集成测试视需要放在 `src-tauri/tests/` 目录。

## 提交约定

- 语言：中文。
- 格式：`类型(范围): 描述\n\n可选的细节`（Conventional Commits）。
- 常用类型：`feat` / `fix` / `refactor` / `chore` / `docs` / `test` / `style`。
- 范围示例：`domain` / `infra` / `gateway` / `command` / `frontend` / `ipc` / `scripts` / `docs`。
- 描述简明扼要，说清"做了什么"。
- 当提交业务较复杂时，空两行交代可选的细节。

## 依赖管理

- 新增运行时依赖（npm dependencies / Cargo.toml \[dependencies\]）需在 commit message 中说明理由。
- `FRONTEND.md` 的 Locked 级选型（Vue、Pinia、Zod 等）变更需文档更新 + 评审。
- 优先复用已有依赖，避免功能重叠的新包。

## 端到端 UseCase 实现清单

新增或修改一个 UseCase 命令时，按以下顺序操作：

1. **domain/ 层**：定义/修改实体 struct 与 Repository trait。
   - struct 带 `#[derive(Debug, Clone, Serialize, Deserialize, Type)]`。
   - 跨 IPC 暴露的 struct 加 `#[serde(rename_all = "camelCase")]`。
2. **infra/ 或 gateway/ 层**：实现对应的 Repository trait 或 Gateway trait。
3. **command/ 层**：编写 `#[tauri::command] #[specta::specta]` 处理器。
   - 含 per-entity mutex（MH-23）。
   - 含各层 error → IpcError 转换。
4. **lib.rs**：注册新 command 到 Tauri Builder。
5. **构建触发导出**：运行 `pnpm tauri dev` 或 `cargo build`，Specta 生成 TS 类型到 `src/types/`。
6. **Zod schema**：在 `src/schemas/` 手写对应 Zod schema，使用 `satisfies z.ZodType<T>` 约束对齐。
7. **Service**：在 `src/services/` 封装 `invoke()` 调用，入参/返回值经 Zod 校验。
8. **Composable**（如需）：在 `src/composables/` 封装响应式状态逻辑。
9. **View**：在 `src/views/` 或 `src/components/` 消费 service/composable。
10. **测试**：domain 单元测试 + Zod schema 测试 + service 测试。
11. **验证**：`pnpm check && pnpm lint && pnpm test`。

> 若仅涉及类型变更（不新增命令），从步骤 5 开始即可。
> `specta-zod` 自动管线当前不可用（DEBT-001），Zod schema 需手动维护。
> 关键代码模式骨架见 `docs/references/code-patterns.md`。
> 首次实现功能时的完整走通示例见 `docs/references/first-feature-walkthrough.md`。

## ExecPlan 触发规则

- 变更涉及 `CONSTRAINT_INDEX.md` 中的约束（HR / MH 级别）时，创建 exec-plan。
- 纯代码重构、文档修缮、测试补充等低边界成本变更直接实施。

### 边界案例判定

| 场景                                       | 是否需要 ExecPlan | 理由                                         |
| ------------------------------------------ | ----------------- | -------------------------------------------- |
| 新增一个 UseCase 命令（符合既有模式）      | 不需要            | 遵循已定义的 UseCase 清单与实现清单即可      |
| 修改已有错误码的语义或 `retryable` 值      | 需要              | 改变公开契约行为，影响 COMPATIBILITY.md      |
| 新增错误码（不改变已有错误码语义）         | 不需要            | 追加是非破坏性变更                           |
| 修改 per-entity mutex 的超时策略           | 需要              | 直接影响 MH-23 的实现方式                    |
| 新增 ResourceCrudCommand（配置读写）       | 不需要            | 遵循既有 IPC 分层模式                        |
| 将 Locked 级技术依赖替换为另一选型         | 需要              | TD-02 约束要求文档更新 + 评审                |
| 变更 `ThemeCard` 的 schemaVersion 迁移策略 | 需要              | 影响 MH-02、MH-03、HR-05 多个约束            |
| 为已有命令新增一个可选的输出字段           | 不需要            | 非破坏性追加，但须更新 COMPATIBILITY.md 条目 |

## 审查与提交节奏

- 每完成一个小功能或小段文档，就进入一次审查点。
- 优先小步快跑、频繁小提交，方便人工审查。
