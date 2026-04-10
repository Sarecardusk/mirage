# 首个功能端到端实战示例

**仅作参考**

本文档以 `create_theme_card` 为例，走通 `AGENTS.md` 端到端 UseCase 实现清单的完整 11 步。
目标读者：首次在本项目中实现业务功能的开发者或 AI 代理。

> 本示例为伪代码级别，重点在于明确**目录创建顺序、文件放置位置、引导配置**等清单未覆盖的操作细节。
> 骨架代码参考 `docs/references/code-patterns.md`。

## 前置条件

项目当前状态：Tauri 2 + Vue 3 + TypeScript 脚手架就绪，仅有 `greet` 演示命令。
以下目录和依赖**尚不存在**，需在本流程中首次创建/安装。

## 步骤 0：安装预批准依赖

首次实现功能前，需安装架构文档中已确定选型的依赖。

### Rust 侧

在 `src-tauri/` 目录下执行（`serde`、`serde_json` 已在 Cargo.toml 中，无需重复添加）：

```bash
cd src-tauri

# specta / tauri-specta 目前仅有 2.0.0-rc 预发布版，需指定完整版本
# ⚠ 以下版本号仅为示意，执行前请先查询 crates.io 最新 rc 版本
cargo add specta@2.0.0-rc.24 --features derive
cargo add tauri-specta@2.0.0-rc.24 --features typescript

# 其余稳定依赖
cargo add tokio --features full
cargo add thiserror@2
cargo add anyhow
...
```

> `surrealdb`、`tracing`、`reqwest`、`ring`、`argon2`、`validator` 等在需要时按需添加，不必一次全装。
> 版本号以执行时 crates.io 最新为准；上述版本基于 2026-04 查询结果。

### 前端侧

```bash
pnpm add zod
pnpm add -D tailwindcss @tailwindcss/vite
```

> `vue-router`、`pinia`、`@tauri-store/pinia`、`vueuse` 在需要时按需添加。
> `shadcn-vue` 的初始化会引入额外依赖，视 CLI 输出为准。

## 步骤 1：domain/ 层 — 定义实体与 Repository trait

### 创建目录与模块入口

```
src-tauri/src/
├── domain.rs          ← 模块入口（非 domain/mod.rs）
├── domain/
│   ├── theme_card.rs  ← 实体 struct
│   └── error.rs       ← DomainError enum
├── lib.rs             ← 已存在
└── main.rs            ← 已存在
```

**关键约定**：遵循 TD-04，使用 `domain.rs` + `domain/` 结构，不使用 `domain/mod.rs`。

### domain.rs（模块入口）

```rust
pub mod theme_card;
pub mod error;
```

### domain/theme_card.rs

```rust
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ThemeCard {
    pub id: String,
    pub schema_version: u32,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateThemeCardInput {
    pub name: String,
    pub description: String,
}

/// Repository trait — domain 层定义，infra 层实现
pub trait ThemeCardRepository: Send + Sync {
    async fn create(&self, input: CreateThemeCardInput) -> Result<ThemeCard, crate::domain::error::DomainError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<ThemeCard>, crate::domain::error::DomainError>;
}
```

### domain/error.rs

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation failed: {field}")]
    ValidationFailed { field: String, error_code: String },

    #[error("entity not found: {id}")]
    NotFound { id: String, error_code: String },
}
```

### 在 lib.rs 中声明模块

在 `lib.rs` 顶部添加：

```rust
mod domain;
```

## 步骤 2：infra/ 层 — 实现 Repository trait

> 首个功能可先用内存实现，SurrealDB 适配器在安装 `surrealdb` 依赖后再替换。

### 创建目录

```
src-tauri/src/
├── infra.rs
├── infra/
│   └── theme_card_repo.rs
```

### infra.rs

```rust
pub mod theme_card_repo;
```

### infra/theme_card_repo.rs

```rust
use crate::domain::theme_card::{CreateThemeCardInput, ThemeCard, ThemeCardRepository};
use crate::domain::error::DomainError;
// ... 内存实现或 SurrealDB 实现
```

### 在 lib.rs 中声明

```rust
mod domain;
mod infra;
```

## 步骤 3：command/ 层 — 编写 Tauri 命令处理器

### 创建目录

```
src-tauri/src/
├── command.rs
├── command/
│   ├── theme_card.rs   ← 命令处理器
│   ├── error.rs        ← IpcError + 错误转换
│   └── state.rs        ← AppState（含 EntityMutexMap）
```

### command/error.rs

参考 `code-patterns.md` 第 2 节的 `IpcError` 结构。

### command/theme_card.rs

参考 `code-patterns.md` 第 3 节的 per-entity mutex 模式。

关键点：

- 函数签名必须带 `#[tauri::command]` 和 `#[specta::specta]`
- 返回类型为 `Result<ThemeCard, IpcError>`

### 在 lib.rs 中声明

```rust
mod domain;
mod infra;
mod command;
```

## 步骤 4：lib.rs — 注册命令 + 配置 Specta 导出

这是**首次功能的关键引导步骤**，需要同时配置 Tauri 命令注册和 Specta 类型导出。

### lib.rs 完整结构

```rust
mod domain;
mod infra;
mod command;

use command::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Specta 类型导出（开发时自动生成 TS 类型到 src/types/）
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            command::theme_card::create_theme_card,
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/types/bindings.ts",
        )
        .expect("Failed to export Specta types");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
```

> **注意**：`tauri-specta` 的 API 可能随版本变化。以 crate 文档为准，上述为 `tauri-specta 2.x` 的典型用法。
> 首次运行 `cargo build` 或 `pnpm tauri dev` 后，`src/types/bindings.ts` 将自动生成。

### 删除 greet 演示命令

移除 `lib.rs` 中原有的 `greet` 函数，将 `tauri::generate_handler![greet]` 替换为 `builder.invoke_handler()`（如步骤 4 示例所示）。

## 步骤 5：触发构建，Specta 生成 TS 类型

```bash
cd src-tauri && cargo build
```

构建成功后，检查 `src/types/bindings.ts` 是否已生成。

产出文件：

```
src/
├── types/
│   └── bindings.ts    ← Specta 自动生成，不手动编辑
```

> 如果 `src/types/` 目录不存在，Specta 会自动创建。

## 步骤 6：手写 Zod schema

### 创建目录

```
src/
├── schemas/
│   └── themeCard.ts
```

### schemas/themeCard.ts

```typescript
import { z } from "zod";
import type { ThemeCard, CreateThemeCardInput } from "@/types/bindings";

export const ThemeCardSchema = z.object({
  id: z.string(),
  schemaVersion: z.number().int().nonnegative(),
  name: z.string().min(1),
  description: z.string(),
  createdAt: z.string(),
  updatedAt: z.string(),
}) satisfies z.ZodType<ThemeCard>;

export const CreateThemeCardInputSchema = z.object({
  name: z.string().min(1),
  description: z.string(),
}) satisfies z.ZodType<CreateThemeCardInput>;
```

关键点：

- `satisfies z.ZodType<T>` 保证编译时类型对齐
- 对 Rust `u32` 使用 `z.number().int().nonnegative()` 而非简单 `z.number()`

## 步骤 7：Service 层封装 IPC 调用

### 创建目录

```
src/
├── services/
│   └── themeCard.ts
```

### services/themeCard.ts

```typescript
import { invoke } from "@tauri-apps/api/core";
import { ThemeCardSchema, CreateThemeCardInputSchema } from "@/schemas/themeCard";
import type { ThemeCard, CreateThemeCardInput } from "@/types/bindings";

export async function createThemeCard(input: CreateThemeCardInput): Promise<ThemeCard> {
  const validated = CreateThemeCardInputSchema.parse(input);
  const result = await invoke("create_theme_card", { input: validated });
  return ThemeCardSchema.parse(result);
}
```

## 步骤 8：Composable（如需）

本例中 `create_theme_card` 是一次性操作，不需要 composable。
需要 composable 的场景：列表状态管理、流式生成（参考 `code-patterns.md` 第 4 节 `useLlmStream`）。

## 步骤 9：View / Component 消费

在页面组件中直接调用 service 函数：

```vue
<script setup lang="ts">
import { createThemeCard } from "@/services/themeCard";

async function handleCreate() {
  const card = await createThemeCard({ name: "...", description: "..." });
  // 处理结果
}
</script>
```

## 步骤 10：测试

### domain 单元测试

```
src-tauri/src/domain/theme_card.rs  ← 在文件底部添加 #[cfg(test)] mod tests
```

### Zod schema 测试

```
src/schemas/themeCard.test.ts       ← 与 schema 文件同目录
```

测试内容：

- 合法 JSON 能通过 parse
- 缺少必填字段时 parse 抛出
- 类型边界（如 `schemaVersion` 为浮点数时应拒绝）

### Service 测试

```
src/services/themeCard.test.ts      ← 与 service 文件同目录
```

使用 `vi.mock("@tauri-apps/api/core")` mock `invoke` 调用。

## 步骤 11：验证

```bash
pnpm check && pnpm lint && pnpm test
```

## 最终文件树

首个功能完成后，新增文件如下：

```
src-tauri/src/
├── domain.rs
├── domain/
│   ├── theme_card.rs
│   └── error.rs
├── infra.rs
├── infra/
│   └── theme_card_repo.rs
├── command.rs
├── command/
│   ├── theme_card.rs
│   ├── error.rs
│   └── state.rs
├── lib.rs                  ← 已修改：Specta 配置 + 命令注册
└── main.rs                 ← 未修改

src/
├── types/
│   └── bindings.ts         ← Specta 自动生成
├── schemas/
│   ├── themeCard.ts
│   └── themeCard.test.ts
├── services/
│   ├── themeCard.ts
│   └── themeCard.test.ts
```
