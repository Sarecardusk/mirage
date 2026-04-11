import { invoke } from "@tauri-apps/api/core";
import {
  CreateThemeCardInputSchema,
  ThemeCardListSchema,
  ThemeCardSchema,
} from "@/schemas/themeCard";
import type { CreateThemeCardInput, ThemeCard } from "@/types/bindings";

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
