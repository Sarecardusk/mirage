# Mirage Frontend Boundaries

> Last Updated: 2026-04-05

本文件约束 `Theme Card UI Host` 的实现边界，确保前端不侵入 Rust 领域层。
本文件是 `DESIGN.md` 的前端执行化约束，不单独定义全局硬约束。

## 前端职责

- 负责视图渲染、用户输入采集、交互编排与事件展示。
- 负责 IPC 入参与出参的边界解析与错误展示。
- 负责降级状态提示，不负责业务降级决策本身。
- 负责明确区分 `Theme Card` 共享设定与 `Session` 独立上下文。

## UI 技术基线（当前期）

- 默认 UI 栈为 `TailwindCSS + shadcn-vue`（MH-17）。
- `shadcn-vue` 作为默认组件体系与可访问性交互基线，不将 `Headless UI` 作为并列默认基线。
- 该基线属于“默认推荐可例外”约束；若偏离，必须在 `docs/design-docs/` 设计文档说明收益、影响面、回退方案，并通过评审。
- 本基线用于约束组件体系选择，不约束具体页面视觉风格。

## 前端技术栈（当前期）

| 技术　　　　　　　　　　　  | 角色　　　　　　　　　　　　 | 约束级别　　 |
| --------------------------- | ---------------------------- | ------------ |
| Vue 3 + TypeScript　　　　  | 应用框架　　　　　　　　　　 | Locked　　　 |
| Vue Router 4　　　　　　　  | 客户端路由　　　　　　　　　 | Locked　　　 |
| Pinia + @tauri-store/pinia  | 状态管理 + Tauri 原生持久化  | Locked　　　 |
| Zod　　　　　　　　　　　　 | IPC 边界 schema 校验　　　　 | Locked　　　 |
| Vite 6　　　　　　　　　　  | 构建工具　　　　　　　　　　 | Locked　　　 |
| Vitest　　　　　　　　　　  | 单元测试　　　　　　　　　　 | Locked　　　 |
| TailwindCSS + shadcn-vue　  | 原子化样式 + 无头组件库　　  | Default　　  |
| VueUse　　　　　　　　　　  | 通用 composables　　　　　　 | Recommended  |

约束级别说明：

- **Locked**：变更需先更新文档并通过评审。
- **Default**：偏离需在 `docs/design-docs/` 说明收益、影响面、回退方案并通过评审。
- **Recommended**：推荐使用但可替换，无需正式评审。

当前期不做国际化（i18n）；错误标识使用 error code，不依赖硬编码字符串。

## 无障碍基线

- 依赖 `shadcn-vue` 内置的 ARIA 属性与键盘导航支持作为默认无障碍基线。
- 自定义组件须保持语义化 HTML：按钮用 `<button>`，链接用 `<a>`，不用 `<div @click>` 替代。
- 交互元素须可键盘触达（Tab 导航、Enter/Space 激活）。
- 当前不设定具体 WCAG 等级目标，但不得主动破坏 `shadcn-vue` 提供的无障碍能力。

## SFC 约定

### 区块顺序

```vue
<script setup lang="ts">
// ...
</script>

<template>
  <!-- ... -->
</template>

<style scoped>
/* ... */
</style>
```

### script setup 内部顺序

1. imports
2. 类型定义（组件局部类型）
3. `defineProps` / `defineEmits`（使用类型参数风格）
4. composables 与 store 调用
5. 响应式状态（`ref` / `reactive`）
6. `computed`
7. 函数
8. 生命周期钩子（`onMounted` / `onUnmounted`）

### Props 与 Emits

- 使用类型参数风格定义：`defineProps<{ title: string }>()` 和 `defineEmits<{ update: [value: string] }>()`。
- 不使用运行时声明风格（`defineProps({ title: { type: String } })`）。

### 组件提取时机

- 当 SFC `<template>` 超过约 100 行或逻辑可独立复用时考虑提取为子组件。
- 优先提取到同目录下，按功能区域建子目录。
- 此阈值适用于 Vue 组件拆分决策。工具函数/辅助逻辑的抽象判断参见 `AGENTS.md` 决策默认值（"三行相似代码优于过早抽象"规则适用于非组件代码）。

## 目录结构

```
src/
  main.ts              -- Vue 应用入口
  App.vue              -- 根组件
  router/              -- Vue Router 配置与路由定义
  views/               -- 路由级页面组件（命名：XxxView.vue）
  components/          -- 可复用 UI 组件（可按功能区域建子目录）
  stores/              -- Pinia stores（命名：useXxxStore.ts）
  composables/         -- 状态逻辑提取（命名：useXxx.ts）
  services/            -- IPC 调用封装（每个 command 分组一个文件）
  schemas/             -- 手写 Zod schemas（每个领域实体或 command 分组一个文件，使用 satisfies z.ZodType<T> 对齐 Specta 生成类型）
  types/               -- Specta 生成的 TS 类型（视为生成物，不手动编辑）+ 共享类型定义
  assets/              -- 静态资源
```

模块入口遵循 `moduleName.ts + moduleName/` 约定，不使用 `index.ts`。

## 状态管理模式

- Pinia stores 是前端本地状态的唯一管理点（UI 状态、用户偏好、缓存视图数据）。
- `@tauri-store/pinia` 负责选中 store 的持久化到 Tauri 应用数据目录。
- 领域数据（Theme Card、Session 等）始终从 Rust/IPC 获取；前端 store 可缓存用于渲染，但 Rust/SurrealDB 是权威数据源。
- 原则：**stores 持有 UI 状态和读缓存；写操作必须经由 IPC 命令。**

### 持久化范围

| 数据类型                                     | 是否持久化 | 说明                                     |
| -------------------------------------------- | ---------- | ---------------------------------------- |
| 用户偏好（主题、布局、语言设置）             | 是         | 通过 `@tauri-store/pinia` 持久化         |
| 领域数据缓存（ThemeCard 列表、Session 列表） | 否         | 每次从 Rust/IPC 获取，store 仅作渲染缓存 |
| 临时 UI 状态（对话框开关、表单草稿）         | 否         | 随组件生命周期，不持久化                 |

### 更新策略

- 当前阶段不引入乐观更新：IPC 命令成功返回后再刷新 store 中的缓存数据。
- 写操作流程：view 调用 service → service 发起 IPC invoke → 成功后 store 更新缓存或重新拉取。
- 当前不引入 RuntimeEvent 驱动的缓存失效机制。若多命令可修改同一实体（如迁移更新 ThemeCard），后续可按需引入事件驱动刷新。

## 流式消费

前端消费 LLM 流式响应的分层约定：

1. **services 层**：创建 Tauri Channel 并传入 `invoke()`，封装底层 IPC 调用。
2. **composable 层**：如 `useLlmStream`，消费 Channel 事件，管理响应式状态（当前 token、完成状态、错误信息）。
3. **view 层**：绑定 composable 的 reactive refs，实现增量渲染。

RuntimeEvent 监听器须在 `onMounted` 注册、`onUnmounted` 注销，防止内存泄漏。

> 若 Channel 注册晚于首个 `TokenChunk` 到达，丢失的事件不可恢复（见 `HARNESS.md`"不提供事件补发机制"）。前端仍须正确处理后续 `Completion` / `Error` 终止事件。

## 前端禁止项

- 不得通过资源级 CRUD 直接修改关键领域对象（HR-07）。
- 不得在 UI 侧实现 provider 特判逻辑（MH-18）。
- 不得在 UI 侧复制领域规则作为主判定路径（MH-19）。
- 不得假设同步流程在当前可执行。
- 不得将同一卡片不同 Session 伪装成多个 Profile 做隔离。

## IPC 消费约束

- 仅消费 `UseCaseCommand`、`ResourceCrudCommand`、`RuntimeEvent` 约定的稳定语义。
- 所有结构化输入/输出默认使用 `Zod` schema 校验。
- schema 应与 Rust 导出类型保持一致，禁止”编译过但运行不校验”。
- Channel API 消费必须遵循 services → composable → view 分层，不得在 view 中直接操作 Channel。
- Event System 监听器必须管理注册/注销生命周期，不得遗留悬挂监听器。

## 错误与降级展示

- 检索失败时可展示“无检索上下文”降级状态。
- 模型失败时必须展示标准化错误与 `retryable` 信息（MH-10）。
- 不暴露供应商原始不稳定响应结构给最终用户（MH-11）。

## Session 交互边界

- “新建会话”应创建新的 `session_id`，并绑定当前 `theme_card_id`（MH-20）。
- 切换会话仅切换会话态数据，不应重写 `Theme Card` 共享设定（MH-21）。
- 会话级临时配置应默认限定在当前 Session，不外溢到同卡其他会话。

## 审查清单

- UI 是否只编排，不承载核心规则。
- 所有跨边界 payload 是否有 `Zod` 校验。
- 是否存在绕过领域命令的隐式写路径。
- 是否对当前不支持同步有显式处理。
- 同一卡片多 Session 是否保持独立上下文。
