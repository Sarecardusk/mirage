import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createThemeCard,
  deleteThemeCard,
  getThemeCard,
  listThemeCards,
  updateThemeCard,
} from "@/services/themeCard";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

const validCard = {
  id: "card-1",
  schemaVersion: 1,
  name: "Detective",
  systemPrompt: "Stay in noir tone",
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};

describe("themeCard service", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  describe("createThemeCard", () => {
    it("rejects empty input before calling invoke", async () => {
      await expect(createThemeCard({ name: "", systemPrompt: "" })).rejects.toThrow();
      expect(invokeMock).not.toHaveBeenCalled();
    });

    it("calls invoke and parses valid response", async () => {
      invokeMock.mockResolvedValueOnce(validCard);

      const card = await createThemeCard({
        name: "Detective",
        systemPrompt: "Stay in noir tone",
      });

      expect(invokeMock).toHaveBeenCalledWith("create_theme_card", {
        input: { name: "Detective", systemPrompt: "Stay in noir tone" },
      });
      expect(card.id).toBe("card-1");
    });

    it("rejects malformed backend response", async () => {
      invokeMock.mockResolvedValueOnce({ id: "card-1" });
      await expect(
        createThemeCard({ name: "Detective", systemPrompt: "prompt" }),
      ).rejects.toThrow();
    });
  });

  describe("listThemeCards", () => {
    it("parses valid list response", async () => {
      invokeMock.mockResolvedValueOnce([validCard]);
      const cards = await listThemeCards();
      expect(invokeMock).toHaveBeenCalledWith("list_theme_cards");
      expect(cards).toHaveLength(1);
    });

    it("returns empty array for empty response", async () => {
      invokeMock.mockResolvedValueOnce([]);
      const cards = await listThemeCards();
      expect(cards).toHaveLength(0);
    });
  });

  describe("getThemeCard", () => {
    it("calls invoke with correct args and parses response", async () => {
      invokeMock.mockResolvedValueOnce(validCard);
      const card = await getThemeCard("card-1");
      expect(invokeMock).toHaveBeenCalledWith("get_theme_card", { themeCardId: "card-1" });
      expect(card.name).toBe("Detective");
    });

    it("rejects malformed backend response", async () => {
      invokeMock.mockResolvedValueOnce({ broken: true });
      await expect(getThemeCard("card-1")).rejects.toThrow();
    });
  });

  describe("updateThemeCard", () => {
    it("rejects invalid input before calling invoke", async () => {
      await expect(
        updateThemeCard({ themeCardId: "", name: "", systemPrompt: "" }),
      ).rejects.toThrow();
      expect(invokeMock).not.toHaveBeenCalled();
    });

    it("calls invoke with validated input and parses response", async () => {
      const updatedCard = { ...validCard, name: "Renamed", updatedAt: "2026-06-01T00:00:00Z" };
      invokeMock.mockResolvedValueOnce(updatedCard);

      const card = await updateThemeCard({
        themeCardId: "card-1",
        name: "Renamed",
        systemPrompt: "Stay in noir tone",
      });

      expect(invokeMock).toHaveBeenCalledWith("update_theme_card", {
        input: { themeCardId: "card-1", name: "Renamed", systemPrompt: "Stay in noir tone" },
      });
      expect(card.name).toBe("Renamed");
    });

    it("rejects malformed backend response", async () => {
      invokeMock.mockResolvedValueOnce({ bad: true });
      await expect(
        updateThemeCard({ themeCardId: "card-1", name: "Name", systemPrompt: "Prompt" }),
      ).rejects.toThrow();
    });
  });

  describe("deleteThemeCard", () => {
    it("rejects empty themeCardId before calling invoke", async () => {
      await expect(deleteThemeCard("")).rejects.toThrow();
      expect(invokeMock).not.toHaveBeenCalled();
    });

    it("calls invoke with correct themeCardId", async () => {
      invokeMock.mockResolvedValueOnce(null);
      await deleteThemeCard("card-1");
      expect(invokeMock).toHaveBeenCalledWith("delete_theme_card", { themeCardId: "card-1" });
    });

    it("propagates invoke errors", async () => {
      invokeMock.mockRejectedValueOnce(new Error("not found"));
      await expect(deleteThemeCard("card-1")).rejects.toThrow("not found");
    });
  });
});
