import { afterEach, describe, expect, it } from "vitest";
import { mount, unmount, flushSync, tick } from "svelte";
import { clearMocks } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import { installMockBackend } from "../src/lib/mock/backend";
import { app, disposeListeners } from "../src/lib/app.svelte";
import App from "../src/App.svelte";

let instance: ReturnType<typeof mount> | undefined;

async function start(opts: Parameters<typeof installMockBackend>[0] = {}) {
  disposeListeners();
  Object.assign(app, { ready: false, view: "list", chatId: null, messages: [], conversations: [], contacts: [], error: null, settings: null });
  const backend = await installMockBackend({ stressMessages: 300, replyMs: 0, ...opts });
  document.body.innerHTML = '<div id="app"></div>';
  instance = mount(App, { target: document.getElementById("app") as HTMLElement });
  await settle();
  return backend;
}

async function settle() {
  for (let i = 0; i < 8; i++) { await new Promise((r) => setTimeout(r, 15)); flushSync(); await tick(); }
}

const text = () => document.body.textContent ?? "";
const click = async (label: string | RegExp, scope = "") => {
  const el = [...document.querySelectorAll<HTMLElement>(`${scope} button, ${scope} [role=radio]`.trim())].find((b) => {
    const t = `${b.textContent ?? ""} ${b.getAttribute("aria-label") ?? ""}`;
    return typeof label === "string" ? t.includes(label) : label.test(t);
  });
  expect(el, `tlačítko „${String(label)}“`).toBeDefined();
  el?.click();
  await settle();
};

afterEach(() => {
  if (instance) void unmount(instance);
  instance = undefined;
  disposeListeners();
  clearMocks();
});

describe("obrazovky (D4: každá se vykreslí a reaguje)", () => {
  it("onboarding → vytvoření identity → seznam", async () => {
    await start({ seed: false });
    expect(text()).toContain("Začít");
    await click("Začít");
    await click("Další");
    await click("Přeskočit");
    expect(text()).toContain("Vytvořit identitu");
    await click("Vytvořit identitu");
    expect(text()).toContain("Vítejte v ancordu");
  });

  it("seznam konverzací ukáže stav spojení, nepřečtené a poslední zprávu", async () => {
    await start();
    expect(text()).toContain("Online");
    for (const n of ["Alice", "Bob", "Zátěž"]) expect(text()).toContain(n);
    expect(document.querySelector(".badge")?.textContent).toBe("1");
  });

  it("offline ukáže důvod selhání spojení a po připojení zmizí", async () => {
    const backend = await start();
    backend.setConnection("offline");
    await emit("connection_issue", { issue: "node_rejected" });
    await settle();
    expect(text()).toContain("Uzel odmítl požadavek");
    backend.setConnection("online");
    await emit("connection_issue", { issue: null });
    await settle();
    expect(text()).not.toContain("Uzel odmítl");
    expect(text()).toContain("Online");
  });

  it("konverzace: virtuální seznam vykreslí jen okno, zpráva se odešle a pole se vyprázdní", async () => {
    await start();
    await click("Zátěž");
    const rendered = document.querySelectorAll("article.msg").length;
    expect(rendered).toBeGreaterThan(0);
    expect(rendered).toBeLessThan(100);
    const area = document.querySelector("textarea") as HTMLTextAreaElement;
    area.value = "ahoj světe";
    area.dispatchEvent(new Event("input", { bubbles: true }));
    await settle();
    await click("Odeslat");
    expect(area.value).toBe("");
    expect(app.messages[app.messages.length - 1]?.text).toBe("ahoj světe");
  });

  it("konverzace s 10 000 zprávami se otevře do rozpočtu (§13.11)", async () => {
    await start({ stressMessages: 10_000 });
    const t0 = performance.now();
    await click("Zátěž");
    const total = performance.now() - t0;
    const rendered = document.querySelectorAll("article.msg").length;
    console.log(`otevření 10 000 zpráv: ${total.toFixed(0)} ms (včetně ${8 * 15} ms čekání settle), vykresleno ${rendered}`);
    expect(rendered).toBeGreaterThan(0);
    expect(rendered).toBeLessThan(100);
    // settle() čeká pevných 120 ms; rozpočet 150 ms se měří nad ním
    expect(total - 8 * 15).toBeLessThan(150);
  });

  it("zprávy jsou čistý text: HTML se nevykreslí", async () => {
    await start();
    await click("Alice");
    const area = document.querySelector("textarea") as HTMLTextAreaElement;
    area.value = '<img src=x onerror="alert(1)"> **tučně**';
    area.dispatchEvent(new Event("input", { bubbles: true }));
    await settle();
    await click("Odeslat");
    expect(document.querySelector(".vlist img")).toBeNull();
    expect(document.querySelector(".vlist strong")).toBeNull();
    expect(text()).toContain("**tučně**");
  });

  it("kontakty → detail: bezpečnostní číslo (60 číslic) a ověření", async () => {
    await start();
    await click("Kontakty");
    expect(text()).toContain("Alice");
    await click("Alice", "main");
    const num = document.querySelector(".num")?.textContent ?? "";
    expect(num.replaceAll(" ", "")).toMatch(/^\d{60}$/);
    await click("Čísla souhlasí");
    expect(text()).toContain("Ověřený kontakt");
  });

  it("nový kontakt: neplatná adresa ukáže chybu, platná otevře detail", async () => {
    await start();
    await click("Kontakty");
    await click("Přidat kontakt");
    const [addr] = document.querySelectorAll<HTMLInputElement>("input.field");
    if (!addr) throw new Error("chybí pole adresy");
    addr.value = "blbost";
    addr.dispatchEvent(new Event("input", { bubbles: true }));
    await settle();
    await click("Poslat žádost");
    expect(document.querySelector("[role=alert]")?.textContent).toContain("Neplatná adresa");
    addr.value = `ancord:${"B".repeat(43)}`;
    addr.dispatchEvent(new Event("input", { bubbles: true }));
    await settle();
    await click("Poslat žádost");
    expect(text()).toContain("Bezpečnostní číslo");
  });

  it("moje adresa: QR jako obrázek (data:), ne vložené HTML", async () => {
    await start();
    await click("Profil");
    const img = document.querySelector<HTMLImageElement>("img.qr");
    expect(img?.src.startsWith("data:image/svg+xml")).toBe(true);
    expect(text()).toContain("ancord:");
  });

  it("nastavení: téma se přepne na světlé a zapíše; zámek zamkne", async () => {
    await start();
    await click("Nastavení");
    await click("Světlý");
    expect(document.documentElement.dataset["theme"]).toBe("light");
    expect(app.settings?.theme).toBe("light");
    await click("Zamknout teď");
    expect(text()).toContain("Zamčeno");
    expect(text()).not.toContain("Alice");
    await click("Odemknout");
    expect(text()).toContain("Alice");
  });

  it("biometrika: u profilu bez hesla není, s heslem se zapne a zamčeno jde odemknout; zrušení nechá heslo", async () => {
    // bez hesla volba v nastavení není
    await start();
    await click("Nastavení");
    expect(text()).not.toContain("Odemykat biometrikou");

    // profil s heslem
    const backend = await start({ seed: false });
    await click("Začít");
    await click("Další");
    await click("Přeskočit");
    await click("Chránit heslem");
    const [p1, p2] = document.querySelectorAll<HTMLInputElement>("input[type=password]");
    if (!p1 || !p2) throw new Error("chybí pole hesla");
    for (const el of [p1, p2]) { el.value = "tajne-heslo"; el.dispatchEvent(new Event("input", { bubbles: true })); }
    await settle();
    await click("Vytvořit identitu");
    await click("Nastavení");
    expect(text()).toContain("Odemykat biometrikou");
    await click("Odemykat biometrikou");
    expect(app.bio.enabled).toBe(true);

    // zamčeno: biometrika se nabídne sama a odemkne
    await click("Zamknout teď");
    await settle();
    expect(text()).not.toContain("Zamčeno");
    expect(text()).toContain("Chaty");

    // zrušená biometrika: zůstane zamčeno, objeví se pole hesla a chyba
    backend.setBiometricAllowed(false);
    await click("Nastavení");
    await click("Zamknout teď");
    await settle();
    expect(text()).toContain("Zamčeno");
    expect(document.querySelector("input[type=password]")).not.toBeNull();
    expect(document.querySelector("[role=alert]")?.textContent).toContain("Biometrika");
  });
});
