# 代码模式参考

本文件展示关键实现模式的骨架示例，供首次实现时参考。
示例为伪代码级别，以传达结构意图为主，不保证可直接编译。

**构建管线衔接**：运行 `cargo build` 或 `pnpm tauri dev` 后，Specta 将 TS 类型输出到 `src/types/`。手写 Zod schema 在 `src/schemas/` 中使用 `satisfies z.ZodType<T>` 与生成类型对齐。详见 `AGENTS.md` 端到端 UseCase 实现清单步骤 5-6。

## 1. Rust → Specta → Zod 类型流转

### Rust 侧（domain/ 层定义，command/ 层暴露）

```rust
// domain/theme_card.rs
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ThemeCard {
    pub id: String,
    pub schema_version: u32,
    pub name: String,
    // ... 其他字段
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateThemeCardInput {
    pub name: String,
    // ... 其他字段
}
```

### Specta 生成的 TS 类型（src/types/ — 视为生成物，不手动编辑）

```typescript
// src/types/bindings.ts (Specta 生成)
export interface ThemeCard {
  id: string;
  schemaVersion: number;
  name: string;
  // ... 其他字段
}

export interface CreateThemeCardInput {
  name: string;
  // ... 其他字段
}
```

### 手写 Zod schema（src/schemas/）

```typescript
// src/schemas/themeCard.ts
import { z } from "zod";
import type { ThemeCard, CreateThemeCardInput } from "@/types/bindings";

export const ThemeCardSchema = z.object({
  id: z.string(),
  schemaVersion: z.number().int().nonnegative(),
  name: z.string().min(1),
  // ... 其他字段
}) satisfies z.ZodType<ThemeCard>;

export const CreateThemeCardInputSchema = z.object({
  name: z.string().min(1),
  // ... 其他字段
}) satisfies z.ZodType<CreateThemeCardInput>;
```

### IPC Service 封装（src/services/）

```typescript
// src/services/themeCard.ts
import { invoke } from "@tauri-apps/api/core";
import { ThemeCardSchema, CreateThemeCardInputSchema } from "@/schemas/themeCard";
import type { ThemeCard, CreateThemeCardInput } from "@/types/bindings";

export async function createThemeCard(input: CreateThemeCardInput): Promise<ThemeCard> {
  const validated = CreateThemeCardInputSchema.parse(input);
  const result = await invoke("create_theme_card", { input: validated });
  return ThemeCardSchema.parse(result);
}
```

## 2. 错误分类与 IPC 响应映射

### Rust 侧错误定义（各层 thiserror enum）

```rust
// domain/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation failed: {field}")]
    ValidationFailed { field: String, error_code: String },

    #[error("entity not found: {id}")]
    NotFound { id: String, error_code: String },

    #[error("schema version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32, error_code: String },
}
```

### command/ 层统一映射为 IPC 错误响应

```rust
// command/error.rs
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    pub category: ErrorCategory,
    pub error_code: String,
    pub message: String,
    pub retryable: bool,
    pub correlation_id: String,
    // 可选字段见 HARNESS.md "结构化错误最小字段集"
}

#[derive(Debug, Serialize, Type)]
pub enum ErrorCategory {
    Domain,
    Infra,
    Security,
    Gateway,
}

// DomainError → IpcError 转换
impl From<DomainError> for IpcError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::ValidationFailed { field, error_code } => IpcError {
                category: ErrorCategory::Domain,
                error_code,
                message: format!("validation failed: {field}"),
                retryable: false,
                correlation_id: uuid::Uuid::new_v4().to_string(),
            },
            // ... 其他变体
        }
    }
}
```

### TypeScript 侧错误 schema

```typescript
// src/schemas/error.ts
import { z } from "zod";

export const ErrorCategorySchema = z.enum(["Domain", "Infra", "Security", "Gateway"]);

export const IpcErrorSchema = z.object({
  category: ErrorCategorySchema,
  errorCode: z.string(),
  message: z.string(),
  retryable: z.boolean(),
  correlationId: z.string().uuid(),
  // 可选字段
  detail: z.string().nullable().optional(),
  timestamp: z.string().nullable().optional(),
});

export type IpcError = z.infer<typeof IpcErrorSchema>;
```

## 3. Tauri Command Handler + Per-Entity Mutex

```rust
// command/theme_card.rs
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

pub type EntityMutexMap = Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>;

async fn get_entity_lock(map: &EntityMutexMap, entity_id: &str) -> Arc<Mutex<()>> {
    let mut locks = map.lock().await;
    locks
        .entry(entity_id.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

#[tauri::command]
#[specta::specta]
pub async fn create_theme_card(
    state: State<'_, AppState>,
    input: CreateThemeCardInput,
) -> Result<ThemeCard, IpcError> {
    // 校验
    // ... domain 层校验逻辑

    // 获取 per-entity mutex（新建时用临时 ID 或全局创建锁）
    let lock = get_entity_lock(&state.theme_card_locks, "create").await;
    let _guard = lock.lock().await;

    // 委托 domain 层执行
    let card = state.theme_card_service
        .create(input)
        .await
        .map_err(IpcError::from)?;

    Ok(card)
}
```

## 4. 流式生成（Channel API）

```rust
// command/llm.rs
use tauri::ipc::Channel;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "type")]
pub enum LlmStreamEvent {
    TokenChunk { text: String },
    Completion,
    Error { error_code: String, message: String, retryable: bool },
}

#[tauri::command]
#[specta::specta]
pub async fn invoke_llm_generation(
    state: State<'_, AppState>,
    session_id: String,
    theme_card_id: String,
    channel: Channel<LlmStreamEvent>,
) -> Result<(), IpcError> {
    // 1. 检索上下文
    // 2. 构建 prompt
    // 3. 委托 gateway 层流式调用，逐块推送
    // channel.send(LlmStreamEvent::TokenChunk { text: chunk })?;
    // channel.send(LlmStreamEvent::Completion)?;
    Ok(())
}
```

### 前端消费侧（composable）

```typescript
// src/composables/useLlmStream.ts
import { ref } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";

export function useLlmStream() {
  const text = ref("");
  const isStreaming = ref(false);
  const error = ref<string | null>(null);

  async function generate(sessionId: string, themeCardId: string) {
    text.value = "";
    isStreaming.value = true;
    error.value = null;

    const channel = new Channel<LlmStreamEvent>();
    channel.onmessage = (event) => {
      switch (event.type) {
        case "TokenChunk":
          text.value += event.text;
          break;
        case "Completion":
          isStreaming.value = false;
          break;
        case "Error":
          error.value = event.message;
          isStreaming.value = false;
          break;
      }
    };

    await invoke("invoke_llm_generation", {
      sessionId,
      themeCardId,
      channel,
    });
  }

  return { text, isStreaming, error, generate };
}
```
