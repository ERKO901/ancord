import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { clearMocks } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import { installMockBackend } from "../src/lib/mock/backend";
import { app, boot, createIdentity, disposeListeners, go, loadOlder, lockNow, openChat, send, addContact, renameContact, changePassphrase } from "../src/lib/app.svelte";

async function fresh(opts: Parameters<typeof installMockBackend>[0] = {}) {
  disposeListeners();
  Object.assign(app, { ready: false, view: "list", chatId: null, messages: [], conversations: [], contacts: [], error: null });
  return installMockBackend({ stressMessages: 250, replyMs: 0, ...opts });
}

beforeEach(() => { document.body.innerHTML = ""; });
afterEach(() => { disposeListeners(); clearMocks(); });

describe("stav aplikace nad IPC", () => {
  it("boot načte identitu, seznam a kontakty", async () => {
    await fresh();
    await boot();
    expect(app.identity.state).toBe("ready");
    expect(app.conversations.map((c) => c.name)).toContain("Alice");
    expect(app.contacts).toHaveLength(3);
  });

  it("bez identity nabídne vytvoření a po něm je ready", async () => {
    await fresh({ seed: false });
    await boot();
    expect(app.identity.state).toBe("none");
    await createIdentity();
    expect(app.identity.state).toBe("ready");
  });

  it("otevře konverzaci po stránkách a doplní starší zprávy", async () => {
    await fresh();
    await boot();
    const big = app.conversations.find((c) => c.name === "Zátěž");
    expect(big).toBeDefined();
    await openChat(big?.id ?? "");
    expect(app.messages).toHaveLength(100);
    expect(app.moreOlder).toBe(true);
    await loadOlder();
    expect(app.messages).toHaveLength(200);
    const ts = app.messages.map((m) => m.ts);
    expect(ts).toEqual([...ts].sort((a, b) => a - b));
    await loadOlder();
    expect(app.messages).toHaveLength(250);
    expect(app.moreOlder).toBe(false);
  });

  it("odeslání přidá zprávu a doručení ji přepne ze „pending“ na „delivered“", async () => {
    await fresh();
    await boot();
    const alice = app.conversations.find((c) => c.name === "Alice");
    await openChat(alice?.id ?? "");
    await send("  ahoj  ");
    const last = app.messages[app.messages.length - 1];
    expect(last?.text).toBe("ahoj");
    expect(last?.state).toBe("pending");
    await new Promise((r) => setTimeout(r, 600));
    expect(app.messages[app.messages.length - 1]?.state).toBe("delivered");
  });

  it("prázdná zpráva se neposílá", async () => {
    await fresh();
    await boot();
    await openChat(app.conversations[0]?.id ?? "");
    const n = app.messages.length;
    await send("   ");
    expect(app.messages).toHaveLength(n);
  });

  it("příchozí zpráva se zobrazí jen v otevřené konverzaci a duplicita se zahodí", async () => {
    await fresh();
    await boot();
    const alice = app.conversations.find((c) => c.name === "Alice")?.id ?? "";
    await openChat(alice);
    app.view = "chat";
    const message = { id: "x1", ts: Date.now(), mine: false, text: "ahoj", state: "delivered" as const };
    await emit("message_received", { conversationId: alice, message });
    await emit("message_received", { conversationId: alice, message });
    expect(app.messages.filter((m) => m.id === "x1")).toHaveLength(1);
    const before = app.messages.length;
    await emit("message_received", { conversationId: "jiny", message: { ...message, id: "x2" } });
    expect(app.messages).toHaveLength(before);
  });

  it("zámek vymaže obsah z paměti UI", async () => {
    await fresh();
    await boot();
    await openChat(app.conversations[0]?.id ?? "");
    expect(app.messages.length).toBeGreaterThan(0);
    await lockNow();
    expect(app.identity.state).toBe("locked");
    expect(app.messages).toEqual([]);
    expect(app.conversations).toEqual([]);
    expect(app.contacts).toEqual([]);
  });

  it("kontakt: špatná adresa a duplicita vrátí chybu z jádra", async () => {
    await fresh();
    await boot();
    await expect(addContact("nesmysl")).rejects.toBe("Neplatná adresa");
    const addr = `ancord:${"A".repeat(43)}`;
    await addContact(addr, "Karel");
    await expect(addContact(addr)).rejects.toBe("Kontakt už existuje");
    go("contacts");
    expect(app.view).toBe("contacts");
  });
});

describe("jména", () => {
  it("kontakt bez jména ukazuje začátek klíče, po odpovědi jméno protistrany; vlastní jméno má přednost", async () => {
    await fresh({ replyMs: 10 });
    await boot();
    const addr = `ancord:${"B".repeat(43)}`;
    const c = await addContact(addr);
    expect(c.name).toBe("BBBBBBBB…");
    await openChat(c.id);
    await send("ahoj");
    await new Promise((r) => setTimeout(r, 1500));
    expect(app.contacts.find((x) => x.id === c.id)?.name).toBe("Simulovaný soused");
    await renameContact(c.id, "Soused");
    expect(app.contacts.find((x) => x.id === c.id)).toMatchObject({ name: "Soused", peerName: "Simulovaný soused" });
    await renameContact(c.id, "");
    expect(app.contacts.find((x) => x.id === c.id)?.name).toBe("Simulovaný soused");
  });

  it("moje jméno se uloží ořezané a změna hesla chce staré heslo", async () => {
    await fresh();
    await boot();
    const { call } = await import("../src/lib/ipc");
    expect(await call("display_name_set", { name: "  Jan  " })).toBe("Jan");
    expect(await call("display_name_get")).toBe("Jan");
    await expect(changePassphrase(undefined, "krátké")).rejects.toBe("Heslo musí mít alespoň 8 znaků");
    await changePassphrase(undefined, "prvni-heslo");
    expect(app.identity.passphrase).toBe(true);
    await expect(changePassphrase("spatne", "druhe-heslo")).rejects.toBe("Špatné heslo");
    await changePassphrase("prvni-heslo", "druhe-heslo");
  });
});
