import { computed, ref } from "vue";
import { useLlmStream } from "@/composables/useLlmStream";
import { appendMessage, createSession, listMessages } from "@/services/session";
import type { Message, Session } from "@/types/bindings";

export function useChat(themeCardId: () => string) {
  const session = ref<Session | null>(null);
  const messages = ref<Message[]>([]);
  const loading = ref(false);
  const sending = ref(false);
  const error = ref("");

  const llmStream = useLlmStream();
  const sessionId = computed(() => session.value?.id ?? "");

  async function ensureSession(): Promise<Session> {
    if (session.value) {
      return session.value;
    }

    const created = await createSession({
      themeCardId: themeCardId(),
    });
    session.value = created;
    return created;
  }

  async function loadConversation() {
    loading.value = true;
    error.value = "";
    try {
      const currentSession = await ensureSession();
      messages.value = await listMessages(currentSession.id);
    } catch (loadError) {
      error.value = loadError instanceof Error ? loadError.message : String(loadError);
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
      const currentSession = await ensureSession();
      const userMessage = await appendMessage({
        sessionId: currentSession.id,
        role: "user",
        content: content.trim(),
      });
      messages.value.push(userMessage);

      await llmStream.generate(currentSession.id, themeCardId(), {
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
  };
}
