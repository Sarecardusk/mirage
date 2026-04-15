import { describe, expect, it } from "vitest";
import {
  AppendMessageInputSchema,
  CreateSessionInputSchema,
  MessageSchema,
  SessionListSchema,
  SessionSchema,
} from "@/schemas/session";

const validSession = {
  id: "session-1",
  themeCardId: "card-1",
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
  lastOpenedAt: "2026-01-02T00:00:00Z",
};

const validMessage = {
  id: "msg-1",
  role: "user" as const,
  content: "Hello",
  createdAt: "2026-01-01T00:00:00Z",
};

describe("SessionListSchema", () => {
  it("parses an array of sessions", () => {
    const parsed = SessionListSchema.parse([validSession, { ...validSession, id: "session-2" }]);
    expect(parsed).toHaveLength(2);
  });

  it("parses an empty array", () => {
    expect(SessionListSchema.parse([])).toEqual([]);
  });

  it("rejects non-array input", () => {
    expect(() => SessionListSchema.parse(validSession)).toThrow();
  });
});

describe("SessionSchema", () => {
  it("parses a valid session", () => {
    const parsed = SessionSchema.parse(validSession);
    expect(parsed.id).toBe("session-1");
  });

  it("accepts null lastOpenedAt for legacy sessions", () => {
    const parsed = SessionSchema.parse({ ...validSession, lastOpenedAt: null });
    expect(parsed.lastOpenedAt).toBeNull();
  });

  it("rejects missing themeCardId", () => {
    const { id, createdAt, updatedAt, lastOpenedAt } = validSession;
    expect(() => SessionSchema.parse({ id, createdAt, updatedAt, lastOpenedAt })).toThrow();
  });
});

describe("MessageSchema", () => {
  it("parses a valid message", () => {
    const parsed = MessageSchema.parse(validMessage);
    expect(parsed.content).toBe("Hello");
  });

  it("rejects invalid role", () => {
    expect(() => MessageSchema.parse({ ...validMessage, role: "admin" })).toThrow();
  });

  it("accepts all valid roles", () => {
    for (const role of ["user", "assistant", "system"]) {
      expect(MessageSchema.parse({ ...validMessage, role }).role).toBe(role);
    }
  });
});

describe("CreateSessionInputSchema", () => {
  it("parses valid input", () => {
    const parsed = CreateSessionInputSchema.parse({ themeCardId: "card-1" });
    expect(parsed.themeCardId).toBe("card-1");
  });

  it("rejects empty themeCardId", () => {
    expect(() => CreateSessionInputSchema.parse({ themeCardId: "" })).toThrow();
  });
});

describe("AppendMessageInputSchema", () => {
  it("parses valid input", () => {
    const parsed = AppendMessageInputSchema.parse({
      sessionId: "session-1",
      role: "user",
      content: "Hello",
    });
    expect(parsed.content).toBe("Hello");
  });

  it("rejects empty sessionId", () => {
    expect(() =>
      AppendMessageInputSchema.parse({ sessionId: "", role: "user", content: "Hello" }),
    ).toThrow();
  });

  it("rejects empty content", () => {
    expect(() =>
      AppendMessageInputSchema.parse({ sessionId: "s-1", role: "user", content: "" }),
    ).toThrow();
  });

  it("rejects invalid role", () => {
    expect(() =>
      AppendMessageInputSchema.parse({ sessionId: "s-1", role: "moderator", content: "Hi" }),
    ).toThrow();
  });
});
