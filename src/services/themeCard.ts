import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";
import {
  CreateThemeCardInputSchema,
  ThemeCardListSchema,
  ThemeCardSchema,
  UpdateThemeCardInputSchema,
} from "@/schemas/themeCard";
import type { CreateThemeCardInput, ThemeCard, UpdateThemeCardInput } from "@/types/bindings";

export async function createThemeCard(input: CreateThemeCardInput): Promise<ThemeCard> {
  const validatedInput = CreateThemeCardInputSchema.parse(input);
  const response = await invoke("create_theme_card", { input: validatedInput });
  return ThemeCardSchema.parse(response);
}

export async function listThemeCards(): Promise<ThemeCard[]> {
  const response = await invoke("list_theme_cards");
  return ThemeCardListSchema.parse(response);
}

export async function getThemeCard(themeCardId: string): Promise<ThemeCard> {
  const response = await invoke("get_theme_card", { themeCardId });
  return ThemeCardSchema.parse(response);
}

export async function updateThemeCard(input: UpdateThemeCardInput): Promise<ThemeCard> {
  const validatedInput = UpdateThemeCardInputSchema.parse(input);
  const response = await invoke("update_theme_card", { input: validatedInput });
  return ThemeCardSchema.parse(response);
}

export async function deleteThemeCard(themeCardId: string): Promise<void> {
  z.string().min(1, "Theme card ID is required").parse(themeCardId);
  await invoke("delete_theme_card", { themeCardId });
}
