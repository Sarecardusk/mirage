<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useChat } from "@/composables/useChat";
import AppLayout from "@/components/AppLayout.vue";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Textarea } from "@/components/ui/textarea";
import { deleteSession, listSessions } from "@/services/session";
import type { Session } from "@/types/bindings";

const route = useRoute();
const router = useRouter();
const input = ref("");

const themeCardId = computed(() => String(route.params.themeCardId ?? ""));
const routeSessionId = computed(() => {
  const sid = route.params.sessionId;
  return sid ? String(sid) : undefined;
});

const {
  error,
  llmStream: rawLlmStream,
  loadConversation,
  loading,
  messages,
  sendUserMessage,
  sending,
  sessionId,
  startNewSession,
  switchSession,
} = useChat(
  () => themeCardId.value,
  () => routeSessionId.value,
);
const llmStream = reactive(rawLlmStream);

// session 列表状态
const sessions = ref<Session[]>([]);
const sessionsLoading = ref(false);
const sessionsError = ref("");
const deleting = ref<string | null>(null);

function formatSessionTime(isoString: string | null): string {
  if (!isoString) return "—";
  const date = new Date(isoString);
  return date.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function roleClass(role: string): string {
  return role === "user"
    ? "bg-primary text-primary-foreground"
    : "bg-muted text-foreground border border-border";
}

async function refreshSessions() {
  if (!themeCardId.value) return;
  sessionsLoading.value = true;
  sessionsError.value = "";
  try {
    sessions.value = await listSessions(themeCardId.value);
  } catch (e) {
    sessionsError.value = e instanceof Error ? e.message : String(e);
  } finally {
    sessionsLoading.value = false;
  }
}

async function handleSelectSession(targetSession: Session) {
  await router.replace({
    name: "chat",
    params: { themeCardId: themeCardId.value, sessionId: targetSession.id },
  });
  await switchSession(targetSession.id);
  // 切换后刷新列表，使 last_opened_at 排序更新
  await refreshSessions();
}

async function handleNewSession() {
  await startNewSession();
  await refreshSessions();
  if (sessionId.value) {
    await router.replace({
      name: "chat",
      params: { themeCardId: themeCardId.value, sessionId: sessionId.value },
    });
  }
}

async function handleDeleteSession(target: Session) {
  deleting.value = target.id;
  try {
    await deleteSession(target.id);
    await refreshSessions();

    // 删除的是当前 session → 切到列表第一个，或新建
    if (target.id === sessionId.value) {
      if (sessions.value.length > 0) {
        await handleSelectSession(sessions.value[0]);
      } else {
        await handleNewSession();
      }
    }
  } catch (e) {
    sessionsError.value = e instanceof Error ? e.message : String(e);
  } finally {
    deleting.value = null;
  }
}

async function sendMessage() {
  const content = input.value;
  input.value = "";
  await sendUserMessage(content);
  await refreshSessions();
}

onMounted(async () => {
  // 先获取 session 列表，再决定打开哪个
  await refreshSessions();

  if (routeSessionId.value) {
    // URL 已指定 session：直接加载
    await loadConversation(routeSessionId.value);
  } else if (sessions.value.length > 0) {
    // 无 URL 参数但存在历史 session：打开最近打开的（列表第一个）
    const target = sessions.value[0];
    await loadConversation(target.id);
    await router.replace({
      name: "chat",
      params: { themeCardId: themeCardId.value, sessionId: target.id },
    });
    await refreshSessions();
  } else {
    // 该卡片尚无任何 session：创建新会话
    await loadConversation();
    if (sessionId.value) {
      await router.replace({
        name: "chat",
        params: { themeCardId: themeCardId.value, sessionId: sessionId.value },
      });
      await refreshSessions();
    }
  }
});

// 从 sidebar 切换到不同 Theme Card 时重新初始化
watch(themeCardId, async () => {
  await refreshSessions();
  if (sessions.value.length > 0) {
    await loadConversation(sessions.value[0].id);
  } else {
    await loadConversation();
  }
});
</script>

<template>
  <AppLayout>
    <div class="flex h-[calc(100vh-8rem)] gap-4">
      <!-- session 历史侧栏 -->
      <aside class="flex w-52 shrink-0 flex-col gap-2">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium text-muted-foreground">会话记录</span>
          <Button size="sm" variant="outline" :disabled="loading" @click="handleNewSession">
            新建
          </Button>
        </div>
        <Separator />
        <p v-if="sessionsError" class="text-xs text-destructive">{{ sessionsError }}</p>
        <ScrollArea class="flex-1">
          <div class="space-y-1 pr-1">
            <div
              v-for="s in sessions"
              :key="s.id"
              class="group flex items-center gap-1 rounded-md pr-1 transition-colors"
              :class="s.id === sessionId ? 'bg-primary' : 'hover:bg-accent'"
            >
              <!-- session 名称/时间，点击切换 -->
              <button
                class="min-w-0 flex-1 truncate px-2 py-2 text-left text-xs"
                :class="s.id === sessionId ? 'text-primary-foreground' : 'text-muted-foreground'"
                @click="handleSelectSession(s)"
              >
                {{ formatSessionTime(s.lastOpenedAt ?? s.createdAt) }}
              </button>
              <!-- 删除按钮，hover 时才显示 -->
              <button
                class="hidden shrink-0 rounded p-0.5 opacity-60 hover:opacity-100 group-hover:flex"
                :class="
                  s.id === sessionId
                    ? 'text-primary-foreground hover:bg-primary-foreground/20'
                    : 'text-muted-foreground hover:bg-destructive/10 hover:text-destructive'
                "
                :disabled="deleting === s.id"
                title="删除会话"
                @click.stop="handleDeleteSession(s)"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
            <p
              v-if="!sessionsLoading && sessions.length === 0"
              class="px-2 py-2 text-xs text-muted-foreground"
            >
              暂无会话
            </p>
          </div>
        </ScrollArea>
      </aside>

      <!-- 聊天主区域 -->
      <div class="flex min-w-0 flex-1 flex-col gap-3">
        <!-- 消息列表 -->
        <ScrollArea class="flex-1 rounded-lg border p-3">
          <div class="space-y-3">
            <div
              v-for="message in messages"
              :key="message.id"
              class="max-w-[80%] rounded-lg p-3 text-sm"
              :class="[
                roleClass(message.role),
                message.role === 'user' ? 'ml-auto text-right' : 'mr-auto text-left',
              ]"
            >
              <div class="mb-1 text-[11px] opacity-70">{{ message.role }}</div>
              <div class="whitespace-pre-wrap">{{ message.content }}</div>
            </div>

            <!-- LLM 流式输出气泡 -->
            <div
              v-if="llmStream.streamingText"
              class="mr-auto max-w-[80%] rounded-lg border border-border bg-muted p-3 text-sm"
            >
              <div class="mb-1 text-[11px] opacity-70">assistant</div>
              <div class="whitespace-pre-wrap">{{ llmStream.streamingText }}</div>
            </div>

            <p v-if="loading" class="text-sm text-muted-foreground">加载会话中...</p>
          </div>
        </ScrollArea>

        <!-- 错误提示 -->
        <p v-if="error" class="text-sm text-destructive">{{ error }}</p>
        <p v-if="llmStream.streamError" class="text-sm text-destructive">
          {{ llmStream.streamError }}
        </p>

        <!-- 输入区 -->
        <div class="space-y-2">
          <Textarea
            v-model="input"
            placeholder="输入你的消息..."
            :disabled="sending || llmStream.isStreaming"
          />
          <div class="flex justify-end">
            <Button
              :disabled="sending || llmStream.isStreaming || !input.trim()"
              @click="sendMessage"
            >
              {{ llmStream.isStreaming ? "生成中..." : sending ? "发送中..." : "发送" }}
            </Button>
          </div>
        </div>
      </div>
    </div>
  </AppLayout>
</template>
