import { z } from "zod";
import type { CreateThemeCardInput, ThemeCard, UpdateThemeCardInput } from "@/types/bindings";

export const ThemeCardSchema = z.object({
  id: z.string(),
  schemaVersion: z.number().int().nonnegative(),
  name: z.string().min(1),
  systemPrompt: z.string().min(1),
  createdAt: z.string(),
  updatedAt: z.string(),
}) satisfies z.ZodType<ThemeCard>;

export const CreateThemeCardInputSchema = z.object({
  name: z.string().min(1, "Name is required"),
  systemPrompt: z.string().min(1, "System prompt is required"),
}) satisfies z.ZodType<CreateThemeCardInput>;

export const ThemeCardListSchema = z.array(ThemeCardSchema);

export const UpdateThemeCardInputSchema = z.object({
  themeCardId: z.string().min(1, "Theme card ID is required"),
  name: z.string().min(1, "Name is required"),
  systemPrompt: z.string().min(1, "System prompt is required"),
}) satisfies z.ZodType<UpdateThemeCardInput>;
