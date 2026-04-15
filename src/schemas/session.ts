import { z } from "zod";
import type {
  AppendMessageInput,
  ChatRole,
  CreateSessionInput,
  Message,
  Session,
} from "@/types/bindings";

export const ChatRoleSchema = z.enum(["user", "assistant", "system"]) satisfies z.ZodType<ChatRole>;

export const SessionSchema = z.object({
  id: z.string(),
  themeCardId: z.string(),
  createdAt: z.string(),
  updatedAt: z.string(),
  lastOpenedAt: z.string().nullable(),
}) satisfies z.ZodType<Session>;

export const MessageSchema = z.object({
  id: z.string(),
  role: ChatRoleSchema,
  content: z.string(),
  createdAt: z.string(),
}) satisfies z.ZodType<Message>;

export const SessionListSchema = z.array(SessionSchema);

export const MessageListSchema = z.array(MessageSchema);

export const CreateSessionInputSchema = z.object({
  themeCardId: z.string().min(1),
}) satisfies z.ZodType<CreateSessionInput>;

export const AppendMessageInputSchema = z.object({
  sessionId: z.string().min(1),
  role: ChatRoleSchema,
  content: z.string().min(1),
}) satisfies z.ZodType<AppendMessageInput>;
