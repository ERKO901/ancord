<script lang="ts">
  import { app, go, addContact, errText } from "../lib/app.svelte";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";

  let address = $state("");
  let name = $state("");
  let busy = $state(false);

  // Android: nativní skener QR (`MainActivity.ScanBridge`); jinde tlačítko není. Výsledek jen vyplní pole, přidání potvrdí uživatel.
  const scanner = (window as unknown as { AncordScan?: { scan(): void } }).AncordScan;
  function scan() {
    (window as unknown as { __ancordScan?: (t: string | null) => void }).__ancordScan = (t) => {
      if (t) address = t.trim();
    };
    scanner?.scan();
  }

  async function paste() {
    try {
      const t = (await navigator.clipboard.readText()).trim();
      if (t) address = t;
    } catch {
      app.error = "Schránku se nepodařilo přečíst";
    }
  }

  async function add() {
    busy = true;
    app.error = null;
    try {
      const c = await addContact(address.trim(), name.trim() || undefined);
      go("contact", c.id);
    } catch (e) {
      app.error = errText(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="page">
  <Header title="Nový kontakt" onback={() => go("contacts")} />
  <form onsubmit={(e) => { e.preventDefault(); void add(); }}>
    {#if scanner}
      <button type="button" class="scan card" onclick={scan}>
        <span class="art"><Icon name="qr" size={28} /></span>
        <span class="t"><strong>Skenovat QR kód</strong><span class="muted">Nejrychlejší a bez překlepů</span></span>
        <Icon name="chevron" size={20} />
      </button>
      <p class="or muted">nebo ručně</p>
    {/if}
    <label>
      <span class="lbl">Adresa</span>
      <span class="with-btn">
        <input class="field mono" placeholder="ancord:…" bind:value={address} autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" />
        <button type="button" class="icon-btn pasteb" aria-label="Vložit ze schránky" onclick={paste}><Icon name="paste" /></button>
      </span>
    </label>
    <label>
      <span class="lbl">Jméno <span class="muted">(vidíte jen vy)</span></span>
      <input class="field" placeholder="Například Alice" bind:value={name} autocomplete="off" autocorrect="off" />
    </label>
    <div class="hint card"><Icon name="shield" size={20} /><p class="muted">Adresu dostanete od kontaktu textem nebo QR kódem. Po přidání si ověřte bezpečnostní číslo mimo síť, jinak vám může někdo podstrčit cizí klíč.</p></div>
    <button class="btn primary" disabled={busy || !address.trim()}>Přidat</button>
  </form>
</section>

<style>
  .page { display: flex; flex-direction: column; height: 100%; min-height: 0; overflow-y: auto; }
  form { display: flex; flex-direction: column; gap: var(--space-4); padding: var(--space-2) var(--space-4) calc(var(--space-5) + var(--safe-bottom)); max-width: 560px; width: 100%; margin: 0 auto; }
  label { display: flex; flex-direction: column; gap: var(--space-2); }
  .lbl { font-weight: 500; }
  .with-btn { position: relative; display: block; }
  .with-btn .field { padding-right: 52px; }
  .pasteb { position: absolute; right: 2px; top: 0; color: var(--muted); }
  .scan { display: flex; align-items: center; gap: var(--space-3); padding: var(--space-4); text-align: left; width: 100%; }
  .scan:hover { background: var(--surface-hover); }
  .scan .art { width: 52px; height: 52px; display: grid; place-items: center; border-radius: 16px; background: var(--accent-soft); color: var(--accent); flex: none; }
  .scan .t { flex: 1; display: flex; flex-direction: column; }
  .or { margin: 0; text-align: center; font-size: var(--fs-sm); }
  .hint { display: flex; gap: var(--space-3); padding: var(--space-3) var(--space-4); align-items: flex-start; color: var(--accent); }
  .hint p { margin: 0; font-size: var(--fs-sm); }
</style>
