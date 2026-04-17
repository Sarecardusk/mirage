import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";
import {
  ListLlmModelsInputSchema,
  LlmConfigSchema,
  SetLlmConfigInputSchema,
  TestLlmConnectionInputSchema,
} from "@/schemas/config";
import type {
  ListLlmModelsInput,
  LlmConfig,
  SetLlmConfigInput,
  TestLlmConnectionInput,
} from "@/types/bindings";

export async function getLlmConfig(): Promise<LlmConfig> {
  const response = await invoke("get_llm_config");
  return LlmConfigSchema.parse(response);
}

export async function getLlmApiKey(): Promise<string> {
  return z.string().parse(await invoke("get_llm_api_key"));
}

export async function setLlmConfig(input: SetLlmConfigInput): Promise<LlmConfig> {
  const validatedInput = SetLlmConfigInputSchema.parse(input);
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
