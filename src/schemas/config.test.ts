import { describe, expect, it } from "vitest";
import {
  ListLlmModelsInputSchema,
  LlmConfigSchema,
  TestLlmConnectionInputSchema,
} from "@/schemas/config";

const validConfig = {
  endpoint: "https://api.openai.com/v1",
  apiKey: "sk-test-key",
  model: "gpt-4o-mini",
  temperature: null,
  maxTokens: null,
  topP: null,
  frequencyPenalty: null,
  presencePenalty: null,
};

describe("LlmConfigSchema", () => {
  it("parses valid config with all generation params null", () => {
    const parsed = LlmConfigSchema.parse(validConfig);
    expect(parsed.model).toBe("gpt-4o-mini");
    expect(parsed.temperature).toBeNull();
  });

  it("parses valid config with generation params set", () => {
    const parsed = LlmConfigSchema.parse({
      ...validConfig,
      temperature: 1.0,
      maxTokens: 512,
      topP: 0.9,
      frequencyPenalty: -1.0,
      presencePenalty: 1.0,
    });
    expect(parsed.temperature).toBe(1.0);
    expect(parsed.maxTokens).toBe(512);
    expect(parsed.topP).toBe(0.9);
    expect(parsed.frequencyPenalty).toBe(-1.0);
    expect(parsed.presencePenalty).toBe(1.0);
  });

  it("rejects empty endpoint", () => {
    expect(() => LlmConfigSchema.parse({ ...validConfig, endpoint: "" })).toThrow();
  });

  it("rejects empty apiKey", () => {
    expect(() => LlmConfigSchema.parse({ ...validConfig, apiKey: "" })).toThrow();
  });

  it("rejects empty model", () => {
    expect(() => LlmConfigSchema.parse({ ...validConfig, model: "" })).toThrow();
  });

  it("rejects missing fields", () => {
    expect(() => LlmConfigSchema.parse({})).toThrow();
  });

  it("rejects temperature above 2", () => {
    expect(() => LlmConfigSchema.parse({ ...validConfig, temperature: 2.1 })).toThrow();
  });

  it("rejects temperature below 0", () => {
    expect(() => LlmConfigSchema.parse({ ...validConfig, temperature: -0.1 })).toThrow();
  });

  it("rejects maxTokens below 1", () => {
    expect(() => LlmConfigSchema.parse({ ...validConfig, maxTokens: 0 })).toThrow();
  });

  it("rejects topP above 1", () => {
    expect(() => LlmConfigSchema.parse({ ...validConfig, topP: 1.1 })).toThrow();
  });

  it("rejects frequencyPenalty out of range", () => {
    expect(() => LlmConfigSchema.parse({ ...validConfig, frequencyPenalty: 2.5 })).toThrow();
  });

  it("rejects presencePenalty out of range", () => {
    expect(() => LlmConfigSchema.parse({ ...validConfig, presencePenalty: -3.0 })).toThrow();
  });

  it("accepts temperature at boundary values", () => {
    expect(() => LlmConfigSchema.parse({ ...validConfig, temperature: 0 })).not.toThrow();
    expect(() => LlmConfigSchema.parse({ ...validConfig, temperature: 2 })).not.toThrow();
  });
});

describe("ListLlmModelsInputSchema", () => {
  it("accepts endpoint and apiKey without model", () => {
    const parsed = ListLlmModelsInputSchema.parse({
      endpoint: "https://api.deepseek.com",
      apiKey: "sk-test-key",
    });
    expect(parsed.endpoint).toBe("https://api.deepseek.com");
  });

  it("rejects missing endpoint", () => {
    expect(() =>
      ListLlmModelsInputSchema.parse({
        endpoint: "",
        apiKey: "sk-test-key",
      }),
    ).toThrow();
  });
});

describe("TestLlmConnectionInputSchema", () => {
  it("requires model", () => {
    expect(() =>
      TestLlmConnectionInputSchema.parse({
        endpoint: "https://api.deepseek.com",
        apiKey: "sk-test-key",
        model: "",
      }),
    ).toThrow();
  });
});

const hasLlmEnvVars =
  !!process.env.MIRAGE_COMPLETION_ENDPOINT?.trim() &&
  !!process.env.MIRAGE_COMPLETION_API_KEY?.trim() &&
  !!process.env.MIRAGE_COMPLETION_MODEL?.trim();

describe.skipIf(!hasLlmEnvVars)("对话补全在线测试", () => {
  it("apiKey 有效时可完成最小 chat 请求", async () => {
    const endpoint = process.env.MIRAGE_COMPLETION_ENDPOINT!.trim();
    const apiKey = process.env.MIRAGE_COMPLETION_API_KEY!.trim();
    const model = process.env.MIRAGE_COMPLETION_MODEL!.trim();

    const base = endpoint.replace(/\/$/, "");
    const url = `${base}/chat/completions`;
    const res = await fetch(url, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${apiKey}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        model,
        messages: [{ role: "user", content: "hi" }],
        max_tokens: 1,
      }),
    });

    const body = await res.text();
    if (res.status !== 200) {
      throw new Error(`HTTP ${res.status}: ${body.slice(0, 500)}`);
    }
  });
});
