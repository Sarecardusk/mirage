import { describe, expect, it } from "vitest";
import { CreateThemeCardInputSchema, ThemeCardSchema } from "@/schemas/themeCard";

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
