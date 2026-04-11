<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { useRoute } from "vue-router";
import { useChat } from "@/composables/useChat";
import AppLayout from "@/components/AppLayout.vue";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";

const route = useRoute();
const input = ref("");

const themeCardId = computed(() => String(route.params.themeCardId ?? ""));
const {
  error,
  llmStream: rawLlmStream,
  loadConversation,
  loading,
  messages,
  sendUserMessage,
  sending,
  sessionId,
} = useChat(() => themeCardId.value);
const llmStream = reactive(rawLlmStream);

function roleClass(role: string): string {
  return role === "user"
    ? "bg-primary text-primary-foreground"
    : "bg-muted text-foreground border border-border";
}

async function sendMessage() {
  const content = input.value;
  input.value = "";
  await sendUserMessage(content);
}

onMounted(() => {
  loadConversation();
});
</script>

<template>
  <AppLayout>
    <div class="space-y-4">
      <div class="rounded-lg border p-3 text-xs text-muted-foreground">
        Theme Card: {{ themeCardId }} | Session: {{ sessionId || "creating..." }}
      </div>

      <div class="h-[52vh] space-y-3 overflow-y-auto rounded-lg border p-3">
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

        <div
          v-if="llmStream.streamingText"
          class="mr-auto max-w-[80%] rounded-lg border border-border bg-muted p-3 text-sm"
        >
          <div class="mb-1 text-[11px] opacity-70">Assistant (streaming)</div>
          <div class="whitespace-pre-wrap">{{ llmStream.streamingText }}</div>
        </div>

        <p v-if="loading" class="text-sm text-muted-foreground">加载会话中...</p>
      </div>

      <p v-if="error" class="text-sm text-destructive">{{ error }}</p>
      <p v-if="llmStream.streamError" class="text-sm text-destructive">
        {{ llmStream.streamError }}
      </p>

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
  </AppLayout>
</template>
