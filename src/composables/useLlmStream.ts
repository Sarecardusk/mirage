import { ref } from "vue";
import { invokeLlmGeneration } from "@/services/llm";
import type { LlmStreamEvent } from "@/types/bindings";

export interface UseLlmStreamCallbacks {
  onTokenChunk?: (chunk: string) => void;
  onCompletion?: (fullText: string) => void;
}

export function useLlmStream() {
  const streamingText = ref("");
  const isStreaming = ref(false);
  const streamError = ref("");

  function clear() {
    streamingText.value = "";
    streamError.value = "";
    isStreaming.value = false;
  }

  function handleEvent(event: LlmStreamEvent, callbacks?: UseLlmStreamCallbacks) {
    if (event.type === "tokenChunk") {
      streamingText.value += event.text;
      callbacks?.onTokenChunk?.(event.text);
      return;
    }

    if (event.type === "completion") {
      callbacks?.onCompletion?.(event.fullText);
      streamingText.value = "";
      isStreaming.value = false;
      return;
    }

    streamError.value = event.message;
    isStreaming.value = false;
  }

  async function generate(
    sessionId: string,
    themeCardId: string,
    callbacks?: UseLlmStreamCallbacks,
  ): Promise<void> {
    clear();
    isStreaming.value = true;

    try {
      await invokeLlmGeneration({
        sessionId,
        themeCardId,
        onEvent: (event) => handleEvent(event, callbacks),
      });
    } catch (error) {
      streamError.value = error instanceof Error ? error.message : String(error);
      isStreaming.value = false;
      throw error;
    }
  }

  return {
    clear,
    generate,
    isStreaming,
    streamError,
    streamingText,
  };
}
