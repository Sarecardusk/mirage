import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";
import {
  ListLlmModelsInputSchema,
  LlmConfigSchema,
  TestLlmConnectionInputSchema,
} from "@/schemas/config";
import type { ListLlmModelsInput, LlmConfig, TestLlmConnectionInput } from "@/types/bindings";

export async function getLlmConfig(): Promise<LlmConfig> {
  const response = await invoke("get_llm_config");
  return LlmConfigSchema.parse(response);
}

export async function setLlmConfig(input: LlmConfig): Promise<LlmConfig> {
  const validatedInput = LlmConfigSchema.parse(input);
  const response = await invoke("set_llm_config", { input: validatedInput });
  return LlmConfigSchema.parse(response);
}

export async function listLlmModels(input: ListLlmModelsInput): Promise<string[]> {
  const validatedInput = ListLlmModelsInputSchema.parse(input);
  const response = await invoke("list_llm_models", { input: validatedInput });
  return z.array(z.string()).parse(response);
}

export async function testLlmConnection(input: TestLlmConnectionInput): Promise<void> {
  const validatedInput = TestLlmConnectionInputSchema.parse(input);
  await invoke("test_llm_connection", { input: validatedInput });
}
