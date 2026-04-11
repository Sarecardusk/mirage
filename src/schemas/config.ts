import { z } from "zod";
import type { LlmConfig, SetLlmConfigInput } from "@/types/bindings";

export const LlmConfigSchema = z.object({
  endpoint: z.string().min(1),
  apiKey: z.string().min(1),
  model: z.string().min(1),
}) satisfies z.ZodType<LlmConfig>;

export const SetLlmConfigInputSchema: z.ZodType<SetLlmConfigInput> = LlmConfigSchema;
