# Mirage

> Last Updated: 2026-04-05

Mirage 是一个基于 Tauri 的本地优先 AI RolePlay 应用。
产品核心实体是 `Theme Card`，不是 `Character Card`。

## 当前状态（2026-03-30）

- 仓库处于模板期，当前代码仍是 Tauri 默认示例（`greet` 命令 + Vue 示例页）。
- 边界规范文档已建立，可作为后续实现的“先约束、后编码”基线。
- 当前文档已区分“立即生效的工程规则”和“功能落地即生效的目标态约束”，避免把规划态能力误读为现状实现。
- `Theme Card`、`Session`、`Memory Retrieval Layer`、`LLM Gateway` 尚未实现业务代码。

## 产品目标定位

- 打造 SillyTavern-like 的 AI RolePlay 体验，但不复刻其数据格式。
- 采用 `单用户单工作区 + Theme Card 下多 Session` 的作用域模型。
- 由 Rust Core 持有领域规则真源，前端负责交互编排与边界校验。
- 当前只做 Sync-Ready 抽象，不交付可执行同步流程。

## 非目标

- 不承诺兼容 SillyTavern 角色卡格式。
- 不建设云账号中心与云托管同步。
- 不建设云端中转 `LLM Gateway` 服务。

## 架构概览

- `src/`: Vue + TypeScript UI Host（展示、输入、编排）。
- `src-tauri/src/`: Rust Core（命令、规则、运行时能力）。
- `src-tauri/capabilities/`: Tauri capability 声明。
- `docs/`: 底层边界文档与执行计划。

## 文档入口

- 架构地图：`ARCHITECTURE.md`
- 边界权威文档：`docs/DESIGN.md`、`docs/RELIABILITY.md`、`docs/SECURITY.md`
- Harness 工程文档：`docs/HARNESS.md`、`docs/COMPATIBILITY.md`、`docs/TEST_STRATEGY.md`、`docs/OPERATIONS.md`
- 计划规范：`PLANS.md` 与 `docs/exec-plans/`
- 产品规格：`docs/product-specs/v1.md`

## 约束状态分类

- 当前已生效：协作规则、文档门禁、本地验证命令、基础 capability/CSP 开发态约束。
- 落地即生效：Theme Card / Session / 迁移 / 检索 / LLM Gateway / 流式生成等业务约束。
- 仅保留占位：Sync-Ready 相关接口，当前不可执行。

## 开发命令

```bash
pnpm install
pnpm tauri dev
pnpm check
pnpm test
```

## 协作约定

- 先更新边界文档，再落地跨边界实现。
- 每完成一个小功能进入一次审查点，优先小步提交。
