import { invoke } from "@tauri-apps/api/core";
import {
  AppendMessageInputSchema,
  CreateSessionInputSchema,
  MessageListSchema,
  MessageSchema,
  SessionSchema,
} from "@/schemas/session";
import type { AppendMessageInput, CreateSessionInput, Message, Session } from "@/types/bindings";

export async function createSession(input: CreateSessionInput): Promise<Session> {
  const validatedInput = CreateSessionInputSchema.parse(input);
  const response = await invoke("create_session", { input: validatedInput });
  return SessionSchema.parse(response);
}

export async function listMessages(sessionId: string): Promise<Message[]> {
  const response = await invoke("list_messages", { sessionId });
  return MessageListSchema.parse(response);
}

export async function appendMessage(input: AppendMessageInput): Promise<Message> {
  const validatedInput = AppendMessageInputSchema.parse(input);
  const response = await invoke("append_message", { input: validatedInput });
  return MessageSchema.parse(response);
}
