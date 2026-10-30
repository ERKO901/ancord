<script lang="ts">
  import { app, send } from "../lib/app.svelte";
  import { call } from "../lib/ipc";
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
      area.style.height = `${Math.min(area.scrollHeight, 160)}px`;
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
  <div class="box">
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
  </div>
  <button class="send" type="submit" disabled={!text.trim()} aria-label="Odeslat"><Icon name="send" size={24} /></button>
</form>

<style>
  .composer {
    display: flex; align-items: flex-end; gap: var(--space-2);
    padding: var(--space-2) var(--space-3) calc(var(--space-2) + var(--safe-bottom));
  }
  .box {
    flex: 1; min-width: 0; padding: 12px var(--space-4);
    background: var(--surface); border: 1px solid var(--border); border-radius: 26px;
  }
  .box:focus-within { border-color: var(--accent); }
  textarea {
    display: block; width: 100%; min-height: 24px; max-height: 160px; padding: 0; resize: none;
    border: 0; outline: 0; background: transparent; font-size: var(--fs-lg); line-height: 1.4; user-select: text;
  }
  textarea::placeholder { color: var(--muted); }
  .send {
    flex: none; width: 50px; height: 50px; display: grid; place-items: center; border-radius: 50%;
    background: var(--accent); color: var(--on-accent); transition: background 0.12s, opacity 0.12s, transform 0.12s;
  }
  .send:hover:not(:disabled) { background: var(--accent-hover); }
  .send:active:not(:disabled) { transform: scale(0.94); }
  .send:disabled { opacity: 0.35; cursor: default; }
</style>
