<script lang="ts">
  import { app, send } from "../lib/app.svelte";
  import { call } from "../lib/ipc";
  import { connText } from "../lib/format";
  import Icon from "../lib/ui/Icon.svelte";

  let text = $state("");
  let area: HTMLTextAreaElement | undefined = $state();
  let typingOn = false;
  let typingTimer: ReturnType<typeof setTimeout> | undefined;

  function setTyping(on: boolean) {
    const id = app.chatId;
    if (!id || typingOn === on) return;
    typingOn = on;
    void call("typing_set", { id, on });
  }

  function onInput() {
    if (area) {
      area.style.height = "auto";
      area.style.height = `${Math.min(area.scrollHeight, 180)}px`;
    }
    setTyping(text.length > 0);
    clearTimeout(typingTimer);
    typingTimer = setTimeout(() => setTyping(false), 3000);
  }

  async function submit() {
    const t = text;
    if (!t.trim()) return;
    text = "";
    clearTimeout(typingTimer);
    setTyping(false);
    if (area) area.style.height = "auto";
    try {
      await send(t);
    } catch (e) {
      text = t;
      app.error = typeof e === "string" ? e : "Zprávu se nepodařilo uložit";
    }
  }

  function onKey(e: KeyboardEvent) {
    // Klávesnice na dotykovém zařízení vkládá nový řádek, odeslání je tlačítkem.
    if (e.key === "Enter" && !e.shiftKey && !e.isComposing && matchMedia("(pointer: fine)").matches) {
      e.preventDefault();
      void submit();
    }
  }
</script>

<form class="composer" onsubmit={(e) => { e.preventDefault(); void submit(); }}>
  <textarea
    bind:this={area}
    bind:value={text}
    oninput={onInput}
    onkeydown={onKey}
    rows="1"
    placeholder="Napsat zprávu"
    aria-label="Zpráva"
    autocomplete="off"
    autocapitalize="sentences"
    spellcheck="false"
    enterkeyhint="send"
  ></textarea>
  <div class="row">
    <span class="chip dim status" title="Stav spojení">
      <span class="dot {app.conn}"></span>{connText(app.conn, app.connIssue)}
    </span>
    <button class="send" type="submit" disabled={!text.trim()} aria-label="Odeslat"><Icon name="send" size={22} /></button>
  </div>
</form>

<style>
  .composer {
    margin: 0 var(--space-3) calc(var(--space-3) + var(--safe-bottom));
    padding: var(--space-4) var(--space-4) var(--space-3);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }
  textarea {
    width: 100%;
    min-height: 28px;
    max-height: 180px;
    padding: 0;
    resize: none;
    border: 0;
    outline: 0;
    background: transparent;
    font-size: 1.25rem;
    line-height: 1.4;
    user-select: text;
  }
  textarea::placeholder { color: var(--muted); }
  .row { display: flex; align-items: center; justify-content: space-between; }
  .status { min-height: 36px; font-size: var(--fs-md); padding: 0 var(--space-3); }
  .send {
    width: 52px; height: 52px;
    display: grid; place-items: center;
    border-radius: 50%;
    background: var(--accent);
    color: var(--on-accent);
    transition: background 0.12s, opacity 0.12s;
  }
  .send:hover:not(:disabled) { background: var(--accent-hover); }
  .send:disabled { opacity: 0.4; cursor: default; }
</style>
