import { z } from "zod";
import type { LlmStreamEvent } from "@/types/bindings";

export const TokenChunkEventSchema = z.object({
  type: z.literal("tokenChunk"),
  text: z.string(),
});

export const CompletionEventSchema = z.object({
  type: z.literal("completion"),
  full_text: z.string(),
});

export const ErrorEventSchema = z.object({
  type: z.literal("error"),
  error_code: z.string(),
  message: z.string(),
  retryable: z.boolean(),
});

export const LlmStreamEventSchema = z.discriminatedUnion("type", [
  TokenChunkEventSchema,
  CompletionEventSchema,
  ErrorEventSchema,
]) satisfies z.ZodType<LlmStreamEvent>;
