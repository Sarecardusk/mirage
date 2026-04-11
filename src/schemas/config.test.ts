import { describe, expect, it } from "vitest";
import { LlmConfigSchema, SetLlmConfigInputSchema } from "@/schemas/config";

const testEndPoint = process.env.MIRAGE_COMPLETION_ENDPOINT?.trim() || "https://api.openai.com/v1";
const testApiKey = process.env.MIRAGE_COMPLETION_API_KEY?.trim() || "sk-test-key";
const testModel = process.env.MIRAGE_COMPLETION_MODEL?.trim() || "gpt-4o-mini";

const validConfig = {
  endpoint: testEndPoint,
  apiKey: testApiKey,
  model: testModel,
};

describe("LlmConfigSchema", () => {
  it("parses valid config", () => {
    const parsed = LlmConfigSchema.parse(validConfig);
    expect(parsed.model).toBe(testModel);
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
});

describe("SetLlmConfigInputSchema", () => {
  it("shares validation with LlmConfigSchema", () => {
    const parsed = SetLlmConfigInputSchema.parse(validConfig);
    expect(parsed.endpoint).toBe(validConfig.endpoint);
  });

  it("rejects empty fields", () => {
    expect(() => SetLlmConfigInputSchema.parse({ ...validConfig, model: "" })).toThrow();
  });
});

describe("对话补全在线测试", () => {
  it("apiKey 有效时可完成最小 chat 请求", async () => {
    const endpoint = process.env.MIRAGE_COMPLETION_ENDPOINT?.trim();
    const apiKey = process.env.MIRAGE_COMPLETION_API_KEY?.trim();
    const model = process.env.MIRAGE_COMPLETION_MODEL?.trim();
    if (!endpoint || !apiKey || !model) {
      throw new Error(
        "需配置 MIRAGE_COMPLETION_ENDPOINT、MIRAGE_COMPLETION_API_KEY、MIRAGE_COMPLETION_MODEL" +
          "（见 .env.test.example ）",
      );
    }

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
