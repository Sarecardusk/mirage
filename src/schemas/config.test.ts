import { describe, expect, it } from "vitest";
import { LlmConfigSchema } from "@/schemas/config";

const validConfig = {
  endpoint: "https://api.openai.com/v1",
  apiKey: "sk-test-key",
  model: "gpt-4o-mini",
};

describe("LlmConfigSchema", () => {
  it("parses valid config", () => {
    const parsed = LlmConfigSchema.parse(validConfig);
    expect(parsed.model).toBe("gpt-4o-mini");
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
