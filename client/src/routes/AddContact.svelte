<script lang="ts">
  import { app, go, addContact, errText } from "../lib/app.svelte";
  import Header from "../lib/ui/Header.svelte";

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
    <label>
      <span class="muted">Adresa</span>
      <input class="field mono" placeholder="ancord:…" bind:value={address} autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" />
    </label>
    {#if scanner}
      <button type="button" class="btn" onclick={scan}>Skenovat QR kód</button>
    {/if}
    <label>
      <span class="muted">Jméno (jen u vás)</span>
      <input class="field" placeholder="Například Alice" bind:value={name} autocomplete="off" autocorrect="off" />
    </label>
    <p class="muted hint">Adresu dostanete od kontaktu textem nebo QR kódem. Po přidání si ověřte bezpečnostní číslo mimo síť, jinak vám může někdo podstrčit cizí klíč.</p>
    <button class="btn primary" disabled={busy || !address.trim()}>Přidat</button>
  </form>
</section>

<style>
  .page { display: flex; flex-direction: column; height: 100%; }
  form { display: flex; flex-direction: column; gap: var(--space-4); padding: var(--space-4); max-width: 560px; width: 100%; margin: 0 auto; }
  label { display: flex; flex-direction: column; gap: var(--space-1); }
  .hint { margin: 0; font-size: var(--fs-sm); }
</style>
