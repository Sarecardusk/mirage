import { computed, ref } from "vue";
import { useLlmStream } from "@/composables/useLlmStream";
import { appendMessage, createSession, listMessages, touchSession } from "@/services/session";
import type { Message, Session } from "@/types/bindings";

// themeCardId: 当前所在卡片（响应式 getter）
// initialSessionId: 可选，从路由参数传入的已有 session id
export function useChat(themeCardId: () => string, initialSessionId?: () => string | undefined) {
  const session = ref<Session | null>(null);
  const messages = ref<Message[]>([]);
  const loading = ref(false);
  const sending = ref(false);
  const error = ref("");

  const llmStream = useLlmStream();
  const sessionId = computed(() => session.value?.id ?? "");

  // 加载指定 session 的消息，不传 sid 时创建新 session
  async function loadConversation(targetSessionId?: string) {
    loading.value = true;
    error.value = "";
    try {
      const sid = targetSessionId ?? initialSessionId?.();
      if (sid) {
        // 直接加载已有 session 的消息，session 对象从 id 构造即可（其他字段仅用于展示）
        session.value = {
          id: sid,
          themeCardId: themeCardId(),
          createdAt: "",
          updatedAt: "",
          lastOpenedAt: null,
        };
        messages.value = await listMessages(sid);
        // 记录本次打开时间，用于下次自动跳转到"最近打开"
        await touchSession(sid);
      } else {
        // 未指定 session，创建新会话
        const created = await createSession({ themeCardId: themeCardId() });
        session.value = created;
        messages.value = [];
      }
    } catch (loadError) {
      error.value = loadError instanceof Error ? loadError.message : String(loadError);
    } finally {
      loading.value = false;
    }
  }

  // 切换到另一个已有 session
  async function switchSession(newSessionId: string) {
    llmStream.clear();
    await loadConversation(newSessionId);
  }

  // 创建全新 session 并切换过去
  async function startNewSession() {
    llmStream.clear();
    loading.value = true;
    error.value = "";
    try {
      const created = await createSession({ themeCardId: themeCardId() });
      session.value = created;
      messages.value = [];
    } catch (createError) {
      error.value = createError instanceof Error ? createError.message : String(createError);
    } finally {
      loading.value = false;
    }
  }

  async function sendUserMessage(content: string) {
    if (!content.trim()) {
      return;
    }

    sending.value = true;
    error.value = "";
    llmStream.clear();

    try {
      // 没有当前 session 时先创建一个（兜底）
      if (!session.value) {
        const created = await createSession({ themeCardId: themeCardId() });
        session.value = created;
        messages.value = [];
      }

      const userMessage = await appendMessage({
        sessionId: session.value.id,
        role: "user",
        content: content.trim(),
      });
      messages.value.push(userMessage);

      await llmStream.generate(session.value.id, themeCardId(), {
        onCompletion: (fullText) => {
          messages.value.push({
            id: `stream-${Date.now()}`,
            role: "assistant",
            content: fullText,
            createdAt: new Date().toISOString(),
          });
        },
      });
    } catch (sendError) {
      error.value = sendError instanceof Error ? sendError.message : String(sendError);
    } finally {
      sending.value = false;
    }
  }

  return {
    error,
    llmStream,
    loadConversation,
    loading,
    messages,
    sendUserMessage,
    sending,
    sessionId,
    startNewSession,
    switchSession,
  };
}
