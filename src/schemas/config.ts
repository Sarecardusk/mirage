import { z } from "zod";
import type {
  ListLlmModelsInput,
  LlmConfig,
  SetLlmConfigInput,
  TestLlmConnectionInput,
} from "@/types/bindings";

export const LlmConfigSchema = z.object({
  endpoint: z.string().min(1),
  apiKeyRef: z.string().min(1),
  model: z.string().min(1),
  temperature: z.number().min(0).max(2).nullable(),
  maxTokens: z.number().int().min(1).nullable(),
  topP: z.number().min(0).max(1).nullable(),
  frequencyPenalty: z.number().min(-2).max(2).nullable(),
  presencePenalty: z.number().min(-2).max(2).nullable(),
}) satisfies z.ZodType<LlmConfig>;

export const SetLlmConfigInputSchema = z.object({
  endpoint: z.string().min(1),
  apiKey: z.string().min(1),
  model: z.string().min(1),
  temperature: z.number().min(0).max(2).nullable(),
  maxTokens: z.number().int().min(1).nullable(),
  topP: z.number().min(0).max(1).nullable(),
  frequencyPenalty: z.number().min(-2).max(2).nullable(),
  presencePenalty: z.number().min(-2).max(2).nullable(),
}) satisfies z.ZodType<SetLlmConfigInput>;

export const ListLlmModelsInputSchema = z.object({
  endpoint: z.string().min(1),
  apiKey: z.string().min(1),
}) satisfies z.ZodType<ListLlmModelsInput>;

export const TestLlmConnectionInputSchema = z.object({
  endpoint: z.string().min(1),
  apiKey: z.string().min(1),
  model: z.string().min(1),
}) satisfies z.ZodType<TestLlmConnectionInput>;
