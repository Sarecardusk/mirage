import { beforeEach, describe, expect, it, vi } from "vitest";
import { getLlmConfig, listLlmModels, setLlmConfig, testLlmConnection } from "@/services/config";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

const validConfig = {
  endpoint: "https://api.deepseek.com",
  apiKey: "sk-test-key",
  model: "deepseek-chat",
  temperature: null,
  maxTokens: null,
  topP: null,
  frequencyPenalty: null,
  presencePenalty: null,
};

describe("config service", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  describe("getLlmConfig", () => {
    it("parses valid response", async () => {
      invokeMock.mockResolvedValueOnce(validConfig);

      const config = await getLlmConfig();

      expect(invokeMock).toHaveBeenCalledWith("get_llm_config");
      expect(config.model).toBe("deepseek-chat");
    });

    it("rejects malformed response", async () => {
      invokeMock.mockResolvedValueOnce({ endpoint: "https://api.deepseek.com" });
      await expect(getLlmConfig()).rejects.toThrow();
    });
  });

  describe("setLlmConfig", () => {
    it("rejects invalid input before calling invoke", async () => {
      await expect(
        setLlmConfig({
          ...validConfig,
          model: "",
        }),
      ).rejects.toThrow();
      expect(invokeMock).not.toHaveBeenCalled();
    });

    it("calls invoke with validated config", async () => {
      invokeMock.mockResolvedValueOnce(validConfig);

      const config = await setLlmConfig(validConfig);

      expect(invokeMock).toHaveBeenCalledWith("set_llm_config", { input: validConfig });
      expect(config.endpoint).toBe("https://api.deepseek.com");
    });
  });

  describe("listLlmModels", () => {
    it("allows fetching models without model field", async () => {
      invokeMock.mockResolvedValueOnce(["deepseek-chat", "deepseek-reasoner"]);

      const models = await listLlmModels({
        endpoint: "https://api.deepseek.com",
        apiKey: "sk-test-key",
      });

      expect(invokeMock).toHaveBeenCalledWith("list_llm_models", {
        input: {
          endpoint: "https://api.deepseek.com",
          apiKey: "sk-test-key",
        },
      });
      expect(models).toEqual(["deepseek-chat", "deepseek-reasoner"]);
    });

    it("rejects malformed list response", async () => {
      invokeMock.mockResolvedValueOnce(["deepseek-chat", 123]);
      await expect(
        listLlmModels({
          endpoint: "https://api.deepseek.com",
          apiKey: "sk-test-key",
        }),
      ).rejects.toThrow();
    });
  });

  describe("testLlmConnection", () => {
    it("requires model before calling invoke", async () => {
      await expect(
        testLlmConnection({
          endpoint: "https://api.deepseek.com",
          apiKey: "sk-test-key",
          model: "",
        }),
      ).rejects.toThrow();
      expect(invokeMock).not.toHaveBeenCalled();
    });

    it("calls invoke with narrowed input", async () => {
      invokeMock.mockResolvedValueOnce(null);

      await testLlmConnection({
        endpoint: "https://api.deepseek.com",
        apiKey: "sk-test-key",
        model: "deepseek-chat",
      });

      expect(invokeMock).toHaveBeenCalledWith("test_llm_connection", {
        input: {
          endpoint: "https://api.deepseek.com",
          apiKey: "sk-test-key",
          model: "deepseek-chat",
        },
      });
    });
  });
});
