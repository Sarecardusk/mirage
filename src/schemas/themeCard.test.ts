import { describe, expect, it } from "vitest";
import {
  CreateThemeCardInputSchema,
  ThemeCardSchema,
  UpdateThemeCardInputSchema,
} from "@/schemas/themeCard";

const validThemeCard = {
  id: "card-1",
  schemaVersion: 1,
  name: "Detective",
  systemPrompt: "Stay in noir tone",
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};

describe("ThemeCardSchema", () => {
  it("parses a valid theme card payload", () => {
    const parsed = ThemeCardSchema.parse(validThemeCard);
    expect(parsed.name).toBe("Detective");
  });

  it("rejects negative schemaVersion", () => {
    expect(() => ThemeCardSchema.parse({ ...validThemeCard, schemaVersion: -1 })).toThrow();
  });

  it("rejects non-integer schemaVersion", () => {
    expect(() => ThemeCardSchema.parse({ ...validThemeCard, schemaVersion: 1.5 })).toThrow();
  });

  it("rejects missing required fields", () => {
    const { id, schemaVersion, systemPrompt, createdAt, updatedAt } = validThemeCard;
    expect(() =>
      ThemeCardSchema.parse({ id, schemaVersion, systemPrompt, createdAt, updatedAt }),
    ).toThrow();
  });

  it("rejects empty name", () => {
    expect(() => ThemeCardSchema.parse({ ...validThemeCard, name: "" })).toThrow();
  });

  it("rejects empty systemPrompt", () => {
    expect(() => ThemeCardSchema.parse({ ...validThemeCard, systemPrompt: "" })).toThrow();
  });
});

describe("CreateThemeCardInputSchema", () => {
  it("parses valid input", () => {
    const parsed = CreateThemeCardInputSchema.parse({
      name: "Detective",
      systemPrompt: "Stay in noir tone",
    });
    expect(parsed.name).toBe("Detective");
  });

  it("rejects empty name", () => {
    expect(() =>
      CreateThemeCardInputSchema.parse({ name: "", systemPrompt: "valid prompt" }),
    ).toThrow();
  });

  it("rejects empty systemPrompt", () => {
    expect(() =>
      CreateThemeCardInputSchema.parse({ name: "Detective", systemPrompt: "" }),
    ).toThrow();
  });
});

describe("UpdateThemeCardInputSchema", () => {
  it("parses valid input", () => {
    const parsed = UpdateThemeCardInputSchema.parse({
      themeCardId: "card-1",
      name: "Updated Name",
      systemPrompt: "Updated prompt",
    });
    expect(parsed.themeCardId).toBe("card-1");
    expect(parsed.name).toBe("Updated Name");
  });

  it("rejects empty themeCardId", () => {
    expect(() =>
      UpdateThemeCardInputSchema.parse({ themeCardId: "", name: "Name", systemPrompt: "Prompt" }),
    ).toThrow();
  });

  it("rejects empty name", () => {
    expect(() =>
      UpdateThemeCardInputSchema.parse({ themeCardId: "card-1", name: "", systemPrompt: "Prompt" }),
    ).toThrow();
  });

  it("rejects empty systemPrompt", () => {
    expect(() =>
      UpdateThemeCardInputSchema.parse({ themeCardId: "card-1", name: "Name", systemPrompt: "" }),
    ).toThrow();
  });
});
