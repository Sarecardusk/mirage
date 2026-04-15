import { invoke } from "@tauri-apps/api/core";
import { LlmConfigSchema } from "@/schemas/config";
import type { LlmConfig } from "@/types/bindings";

export async function getLlmConfig(): Promise<LlmConfig> {
  const response = await invoke("get_llm_config");
  return LlmConfigSchema.parse(response);
}

export async function setLlmConfig(input: LlmConfig): Promise<LlmConfig> {
  const validatedInput = LlmConfigSchema.parse(input);
  const response = await invoke("set_llm_config", { input: validatedInput });
  return LlmConfigSchema.parse(response);
}
