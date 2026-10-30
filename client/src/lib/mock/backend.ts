// Vývojová paměť místo Rustu (jen `vite dev` v prohlížeči a testy). Nic z toho není v release bundlu.
import { mockIPC } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import type {
  Commands, Contact, ConversationSummary, Message, Settings, ConnectionState, IdentityStatus,
} from "../types";

export interface MockOptions {
  /** Počet zpráv v konverzaci „Zátěž“ (měření virtuálního seznamu, §13.11). */
  stressMessages?: number;
  seed?: boolean;
  /** Prodleva simulované odpovědi protistrany v ms, 0 = bez odpovědi. */
  replyMs?: number;
}

const b64u = (n: number) => Array.from({ length: n }, (_, i) => "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"[(i * 7 + n) % 64]).join("");

function digits(seed: string): string {
  let h = 2166136261;
  const out: string[] = [];
  for (let g = 0; g < 12; g++) {
    for (const c of seed + g) h = Math.imul(h ^ c.charCodeAt(0), 16777619) >>> 0;
    out.push(String(h % 100000).padStart(5, "0"));
  }
  return out.join(" ");
}

/** Zástupný vzor místo skutečného QR; skutečný SVG generuje Rust (`address_qr`). */
function fakeQr(text: string): string {
  const n = 25, cell = 8;
  let h = 5381;
  for (const c of text) h = Math.imul(h, 33) ^ c.charCodeAt(0);
  let rects = "";
  for (let y = 0; y < n; y++) for (let x = 0; x < n; x++) {
    h = Math.imul(h ^ (x * 31 + y), 2654435761) >>> 0;
    if (h & 0x10000) rects += `<rect x="${x * cell}" y="${y * cell}" width="${cell}" height="${cell}"/>`;
  }
  const s = n * cell;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${s} ${s}" shape-rendering="crispEdges"><rect width="${s}" height="${s}" fill="#fff"/><g fill="#000">${rects}</g></svg>`;
}

export function createMockBackend(opts: MockOptions = {}) {
  const now = Date.now();
  let identity: IdentityStatus = { state: opts.seed === false ? "none" : "ready", passphrase: false };
  let passphrase: string | undefined;
  let conn: ConnectionState = "online";
  let settings: Settings = {
    theme: "dark", enhancedProtection: true, lockAfterSecs: 300, notificationPreview: false, keepConnection: false,
  };
  const myAddress = `ancord:${b64u(43)}`;
  let seq = 0;
  const id = () => `m${++seq}`;

  const contacts: Contact[] = [];
  const messages = new Map<string, Message[]>();
  const unread = new Map<string, number>();

  const addContact = (name: string, address = `ancord:${b64u(43).split("").reverse().join("")}${contacts.length}`): Contact => {
    const c: Contact = { id: `c${contacts.length + 1}`, name, address, verified: false };
    contacts.push(c);
    messages.set(c.id, []);
    return c;
  };

  if (opts.seed !== false) {
    const alice = addContact("Alice");
    const bob = addContact("Bob");
    const ops = addContact("Zátěž");
    bob.verified = true;
    const m = (cid: string, mine: boolean, text: string, ago: number, state: Message["state"] = "delivered") =>
      messages.get(cid)?.push({ id: id(), ts: now - ago, mine, text, state });
    m(alice.id, false, "Nody běží, 6 z 6 se zaregistrovalo.", 3_600_000);
    m(alice.id, true, "Super. Zkusím klienta na telefonu.", 3_540_000);
    m(alice.id, false, "Bezpečnostní číslo si ověříme osobně.", 120_000);
    unread.set(alice.id, 1);
    m(bob.id, true, "Posílám adresu.", 86_400_000 * 2);
    m(bob.id, false, "Přišlo, díky.", 86_400_000 * 2 - 60_000);
    const big = messages.get(ops.id) ?? [];
    const total = opts.stressMessages ?? 10_000;
    for (let i = 0; i < total; i++) {
      big.push({
        id: id(), ts: now - (total - i) * 30_000, mine: i % 3 === 0,
        text: i % 17 === 0 ? `Delší zpráva číslo ${i}. ${"Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(3)}` : `Zpráva ${i}`,
        state: "delivered",
      });
    }
    messages.set(ops.id, big);
  }

  const summaries = (): ConversationSummary[] =>
    contacts
      .map((c) => {
        const list = messages.get(c.id) ?? [];
        const last = list[list.length - 1];
        return {
          id: c.id, contactId: c.id, name: c.name, lastText: last?.text ?? "", lastTs: last?.ts ?? 0,
          unread: unread.get(c.id) ?? 0, lastState: last?.mine ? last.state : null,
        };
      })
      .filter((s) => s.lastTs > 0)
      .sort((a, b) => b.lastTs - a.lastTs);

  const handlers: { [K in keyof Commands]: (a: Commands[K][0]) => Commands[K][1] | Promise<Commands[K][1]> } = {
    identity_status: () => identity,
    identity_create: ({ passphrase: p }) => {
      identity = { state: "ready", passphrase: !!p };
      passphrase = p;
      return myAddress;
    },
    address_get: () => myAddress,
    address_qr: () => fakeQr(myAddress),
    contact_add: ({ address, name }) => {
      if (!/^(ancord:)?[A-Za-z0-9_-]{43}$/.test(address.trim())) throw "Neplatná adresa";
      if (contacts.some((c) => c.address === address.trim())) throw "Kontakt už existuje";
      return addContact(name?.trim() || `Kontakt ${contacts.length + 1}`, address.trim());
    },
    contact_list: () => contacts.map((c) => ({ ...c })),
    contact_safety_number: ({ id: cid }) => digits(cid),
    contact_mark_verified: ({ id: cid }) => {
      const c = contacts.find((x) => x.id === cid);
      if (c) c.verified = true;
    },
    conversation_list: () => summaries(),
    conversation_open: ({ id: cid, before, limit }) => {
      unread.set(cid, 0);
      const all = messages.get(cid) ?? [];
      const end = before === undefined ? all.length : all.findIndex((m) => m.ts >= before);
      const stop = end < 0 ? all.length : end;
      const start = Math.max(0, stop - limit);
      return { messages: all.slice(start, stop), more: start > 0 };
    },
    message_send: ({ id: cid, text }) => {
      const msg: Message = { id: id(), ts: Date.now(), mine: true, text, state: conn === "online" ? "pending" : "queued" };
      messages.get(cid)?.push(msg);
      if (conn === "online") {
        setTimeout(() => {
          msg.state = "delivered";
          void emit("delivery_state", { conversationId: cid, messageId: msg.id, state: "delivered" });
        }, 400);
        if (opts.replyMs !== 0) {
          setTimeout(() => void emit("typing", { conversationId: cid, on: true }), opts.replyMs ?? 700);
          setTimeout(() => {
            const reply: Message = { id: id(), ts: Date.now(), mine: false, text: "Přijato. (simulovaná odpověď)", state: "delivered" };
            messages.get(cid)?.push(reply);
            void emit("typing", { conversationId: cid, on: false });
            void emit("message_received", { conversationId: cid, message: reply });
          }, (opts.replyMs ?? 700) + 1200);
        }
      }
      return { ...msg };
    },
    typing_set: () => undefined,
    conversation_delete: ({ id: cid }) => {
      messages.set(cid, []);
    },
    connection_state: () => conn,
    settings_get: () => ({ ...settings }),
    settings_set: ({ settings: s }) => {
      settings = { ...s };
    },
    lock: () => {
      identity = { ...identity, state: "locked" };
    },
    unlock: ({ passphrase: p }) => {
      if (identity.passphrase && p !== passphrase) throw "Špatné heslo";
      identity = { ...identity, state: "ready" };
    },
  };

  return {
    handlers,
    setConnection(state: ConnectionState) {
      conn = state;
      void emit("connection_state", { state });
    },
  };
}

export async function installMockBackend(opts?: MockOptions) {
  const backend = createMockBackend(opts);
  mockIPC(
    (cmd, payload) => {
      if (cmd.startsWith("plugin:")) return undefined;
      const h = (backend.handlers as Record<string, ((a: unknown) => unknown) | undefined>)[cmd];
      if (!h) throw `neznámý příkaz: ${cmd}`;
      return h(payload);
    },
    { shouldMockEvents: true },
  );
  return backend;
}
