# CHANGELOG

> Last Updated: 2026-04-15

本文件记录项目对外可见或对协作有影响的重要变更。

## v0.1.2 — LLM 生成参数配置（2026-04-17）

### 新增

- **生成参数全链路**：`LlmConfig` 新增 `temperature`、`max_tokens`、`top_p`、`frequency_penalty`、`presence_penalty` 五个可选参数；`null` = 使用供应商默认值，gateway 只在值非空时才向上游注入对应字段。
- **模型列表与连通性测试命令**：新增 `list_llm_models`、`test_llm_connection` 两个 IPC 命令；使用表单实时值（无需先保存），通过 gateway 直接调用上游 API。
- **SettingsView 生成参数 UI**：每个参数独立启用复选框，未勾选时发送 `null` 并实际保留供应商默认；移除了 range 滑块与硬编码上限（原 32768）。

### 默认值变更

- LLM 配置默认 endpoint 由 `https://api.openai.com/v1` 改为 `https://api.deepseek.com`，默认模型由 `gpt-4o-mini` 改为 `deepseek-chat`。

### 数据库迁移

- Migration v3：`app_config` 表新增五个 `option<float/int>` 字段；SCHEMAFULL 已有记录自动为 NONE，无需回填。

---

## v0.1.1 — MVP 会话循环与持久化（2026-04-15）

### 新增

- **Rust 四层架构**：建立 `domain/` → `infra/` → `gateway/` → `command/` 分层，Specta 自动导出 TypeScript 类型。
- **Theme Card 管理**：`create_theme_card`、`list_theme_cards`、`get_theme_card` 三个 IPC 命令，支持创建与浏览 Theme Card。
- **会话系统**：`create_session`、`list_messages`、`append_message` 三个 IPC 命令，实现最简对话循环。
- **LLM 网关**：`invoke_llm_generation` 命令，通过 reqwest 调用 OpenAI 兼容 API，支持 SSE 流式响应。
- **应用配置**：`get_llm_config`、`set_llm_config` 命令，管理 LLM API 连接参数。
- **SurrealDB 持久化**：ThemeCard、Session、AppConfig 全部落库（SurrealKV 文件引擎），重启后数据保留。
- **前端三页面**：ThemeCardListView（首页列表）、ChatView（对话界面）、SettingsView（LLM 配置）。
- **前端服务层**：`services/`（invoke 封装）、`schemas/`（Zod 运行时校验）、`composables/`（useChat、useLlmStream）。
- **UI 基础设施**：TailwindCSS + shadcn-vue 组件库、AppLayout 布局组件、vue-router 路由。

### 工程改进

- 引入 `@tailwindcss/vite` 与 shadcn-vue，建立统一的 UI 组件体系。
- 添加 `src/` 工作区路径别名（`@/`）。

### 文档

- 完善基础项目文档结构。

## v0.1.0 — 项目初始化（2026-03-30）

- 初始化 Tauri + Vue 3 + TypeScript 项目骨架。
- 引入 Vitest、ESLint、Prettier，建立基本工程链。
- 完成文档体系初版：ARCHITECTURE.md、DESIGN.md、RELIABILITY.md、SECURITY.md 等权威文档就位。
- 确立核心实体为 `Theme Card`，明确不兼容 SillyTavern 角色卡格式。
