import { Channel, invoke } from "@tauri-apps/api/core";
import { LlmStreamEventSchema } from "@/schemas/llm";
import type { LlmStreamEvent } from "@/types/bindings";

export interface InvokeLlmGenerationParams {
  sessionId: string;
  themeCardId: string;
  onEvent: (event: LlmStreamEvent) => void;
}

export async function invokeLlmGeneration(params: InvokeLlmGenerationParams): Promise<void> {
  const channel = new Channel<LlmStreamEvent>();
  channel.onmessage = (event) => {
    const parsed = LlmStreamEventSchema.parse(event);
    params.onEvent(parsed);
  };

  await invoke("invoke_llm_generation", {
    sessionId: params.sessionId,
    themeCardId: params.themeCardId,
    channel,
  });
}
