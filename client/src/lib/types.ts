// IPC kontrakt UI ↔ Rust (ARCHITECTURE.md §13.5). UI dostane jen to, co zobrazuje:
// žádné klíče, tickety ani adresy nodů. Strany musí zůstat shodné s `src-tauri/src/contract.rs`.

export type DeliveryState = "queued" | "pending" | "delivered" | "unknown";
export type ConnectionState = "offline" | "connecting" | "online";
/** Proč spojení nejde (`null` = bez problému). */
export type ConnectionIssue = "worker_unreachable" | "worker_error" | "route_untrusted" | "node_unreachable" | "node_rejected";
export type ThemeChoice = "system" | "dark" | "light";
export type IdentityState = "none" | "locked" | "ready";

export interface IdentityStatus {
  state: IdentityState;
  /** Režim s heslem (§13.6): odemknutí vyžaduje `unlock(passphrase)`. */
  passphrase: boolean;
}

export interface Contact {
  id: string;
  name: string;
  address: string;
  verified: boolean;
}

export interface ConversationSummary {
  id: string;
  contactId: string;
  name: string;
  lastText: string;
  lastTs: number;
  unread: number;
  lastState: DeliveryState | null;
}

export interface Message {
  id: string;
  ts: number;
  mine: boolean;
  /** Čistý text. UI ho nikdy nevykresluje jako HTML ani Markdown (§13.5). */
  text: string;
  state: DeliveryState;
}

export interface Settings {
  theme: ThemeChoice;
  /** FLAG_SECURE, skrytí před přístupnostními službami, overlay ochrana (§13.9). */
  enhancedProtection: boolean;
  /** Sekundy nečinnosti do zamknutí UI, 0 = nikdy. */
  lockAfterSecs: number;
  /** Náhled obsahu v oznámení; výchozí je generické „Nová zpráva“ (§13.8). */
  notificationPreview: boolean;
  /** Android bez distributora: trvalé spojení (§13.8). */
  keepConnection: boolean;
}

export interface ConversationPage {
  messages: Message[];
  /** Zbývají starší zprávy. */
  more: boolean;
}

/** Příkazy: název → [argumenty, výsledek]. Názvy argumentů jsou camelCase (konvence Tauri). */
export interface Commands {
  identity_status: [void, IdentityStatus];
  identity_create: [{ passphrase?: string }, string];
  address_get: [void, string];
  address_qr: [void, string];
  contact_add: [{ address: string; name?: string }, Contact];
  contact_list: [void, Contact[]];
  contact_safety_number: [{ id: string }, string];
  contact_mark_verified: [{ id: string }, void];
  conversation_list: [void, ConversationSummary[]];
  conversation_open: [{ id: string; before?: number; limit: number }, ConversationPage];
  message_send: [{ id: string; text: string }, Message];
  typing_set: [{ id: string; on: boolean }, void];
  conversation_delete: [{ id: string }, void];
  connection_state: [void, ConnectionState];
  settings_get: [void, Settings];
  settings_set: [{ settings: Settings }, void];
  lock: [void, void];
  unlock: [{ passphrase?: string }, void];
}

/** Události Rust → UI. */
export interface Events {
  message_received: { conversationId: string; message: Message };
  delivery_state: { conversationId: string; messageId: string; state: DeliveryState };
  typing: { conversationId: string; on: boolean };
  connection_state: { state: ConnectionState };
  contact_request: { contact: Contact };
  connection_issue: { issue: ConnectionIssue | null };
}

export const COMMAND_NAMES = [
  "identity_status", "identity_create", "address_get", "address_qr",
  "contact_add", "contact_list", "contact_safety_number", "contact_mark_verified",
  "conversation_list", "conversation_open", "message_send", "typing_set", "conversation_delete",
  "connection_state", "settings_get", "settings_set", "lock", "unlock",
] as const satisfies readonly (keyof Commands)[];
