import { call, on } from "./ipc";
import type {
  ConnectionState, Contact, ConversationSummary, IdentityStatus, Message, Settings,
} from "./types";

export type View = "list" | "chat" | "contacts" | "contact" | "add" | "settings" | "me";

const PAGE = 100;

export const app = $state({
  ready: false,
  identity: { state: "none", passphrase: false } as IdentityStatus,
  conn: "offline" as ConnectionState,
  settings: null as Settings | null,
  conversations: [] as ConversationSummary[],
  contacts: [] as Contact[],
  view: "list" as View,
  chatId: null as string | null,
  contactId: null as string | null,
  messages: [] as Message[],
  moreOlder: false,
  typing: {} as Record<string, boolean>,
  error: null as string | null,
});

let loadingOlder = false;
let unlisten: (() => void)[] = [];

export function errText(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : "Neznámá chyba";
}

export async function boot(): Promise<void> {
  app.identity = await call("identity_status");
  app.settings = await call("settings_get");
  app.conn = await call("connection_state");
  if (app.identity.state === "ready") await enterReady();
  app.ready = true;
}

async function enterReady(): Promise<void> {
  await Promise.all([refreshList(), refreshContacts()]);
  if (unlisten.length) return;
  unlisten = await Promise.all([
    on("message_received", ({ conversationId, message }) => {
      if (app.view === "chat" && app.chatId === conversationId) {
        if (!app.messages.some((m) => m.id === message.id)) app.messages.push(message);
      }
      void refreshList();
    }),
    on("delivery_state", ({ conversationId, messageId, state }) => {
      if (app.chatId === conversationId) {
        const m = app.messages.find((x) => x.id === messageId);
        if (m) m.state = state;
      }
      void refreshList();
    }),
    on("typing", ({ conversationId, on: isOn }) => { app.typing[conversationId] = isOn; }),
    on("connection_state", ({ state }) => { app.conn = state; }),
    on("contact_request", ({ contact }) => {
      if (!app.contacts.some((c) => c.id === contact.id)) app.contacts.push(contact);
      void refreshList();
    }),
  ]);
}

export async function createIdentity(passphrase?: string): Promise<void> {
  await call("identity_create", passphrase ? { passphrase } : {});
  app.identity = await call("identity_status");
  await enterReady();
}

export async function unlock(passphrase?: string): Promise<void> {
  await call("unlock", passphrase ? { passphrase } : {});
  app.identity = await call("identity_status");
  await enterReady();
}

export async function lockNow(): Promise<void> {
  await call("lock");
  app.identity = await call("identity_status");
  // Po zamknutí v paměti UI nezůstává žádný obsah zpráv.
  app.messages = [];
  app.conversations = [];
  app.contacts = [];
  app.chatId = null;
  app.contactId = null;
  app.view = "list";
}

export async function refreshList(): Promise<void> {
  app.conversations = await call("conversation_list");
}

export async function refreshContacts(): Promise<void> {
  app.contacts = await call("contact_list");
}

export function go(view: View, id?: string): void {
  app.error = null;
  app.view = view;
  if (view === "chat" && id) void openChat(id);
  if (view === "contact") app.contactId = id ?? null;
  if (view !== "chat") app.chatId = null;
}

export async function openChat(id: string): Promise<void> {
  app.chatId = id;
  app.messages = [];
  app.moreOlder = false;
  const page = await call("conversation_open", { id, limit: PAGE });
  if (app.chatId !== id) return;
  app.messages = page.messages;
  app.moreOlder = page.more;
  await refreshList();
}

export async function loadOlder(): Promise<void> {
  const id = app.chatId;
  const first = app.messages[0];
  if (!id || !first || !app.moreOlder || loadingOlder) return;
  loadingOlder = true;
  try {
    const page = await call("conversation_open", { id, before: first.ts, limit: PAGE });
    if (app.chatId !== id) return;
    app.messages = [...page.messages, ...app.messages];
    app.moreOlder = page.more;
  } finally {
    loadingOlder = false;
  }
}

export async function send(text: string): Promise<void> {
  const id = app.chatId;
  const t = text.trim();
  if (!id || !t) return;
  const msg = await call("message_send", { id, text: t });
  if (app.chatId === id && !app.messages.some((m) => m.id === msg.id)) app.messages.push(msg);
  void refreshList();
}

export async function addContact(address: string, name?: string): Promise<Contact> {
  const c = await call("contact_add", name ? { address, name } : { address });
  await refreshContacts();
  return c;
}

export async function markVerified(id: string): Promise<void> {
  await call("contact_mark_verified", { id });
  await refreshContacts();
}

export async function deleteConversation(id: string): Promise<void> {
  await call("conversation_delete", { id });
  await refreshList();
  go("list");
}

export async function saveSettings(patch: Partial<Settings>): Promise<void> {
  if (!app.settings) return;
  const next = { ...app.settings, ...patch };
  app.settings = next;
  await call("settings_set", { settings: next });
}

export function contactName(id: string | null): string {
  return app.contacts.find((c) => c.id === id)?.name ?? "";
}

export function disposeListeners(): void {
  for (const u of unlisten) u();
  unlisten = [];
}
