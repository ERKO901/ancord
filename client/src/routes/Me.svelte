<script lang="ts">
  import { app, errText } from "../lib/app.svelte";
  import { call } from "../lib/ipc";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";

  let address = $state("");
  let qr = $state("");
  let copied = $state(false);
  let name = $state("");
  let savedName = $state("");
  let nameSaved = $state(false);

  $effect(() => {
    call("display_name_get").then((n) => (name = savedName = n), (e) => (app.error = errText(e)));
    call("address_get").then((a) => (address = a), (e) => (app.error = errText(e)));
    call("address_qr").then((s) => (qr = s), (e) => (app.error = errText(e)));
  });

  // QR je SVG vytvořené v Rustu; do UI jde jako obrázek (data:), ne jako vložené HTML.
  const qrSrc = $derived(qr ? `data:image/svg+xml;utf8,${encodeURIComponent(qr)}` : "");
  const canShare = typeof navigator !== "undefined" && typeof navigator.share === "function";

  async function saveName() {
    try {
      name = savedName = await call("display_name_set", { name });
      nameSaved = true;
      setTimeout(() => (nameSaved = false), 1500);
    } catch (e) {
      app.error = errText(e);
    }
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(address);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      app.error = "Adresu se nepodařilo zkopírovat";
    }
  }

  async function share() {
    try {
      await navigator.share({ text: address });
    } catch {
      // uživatel sdílení zrušil
    }
  }
</script>

<section class="page">
  <Header title="Profil" large />
  <div class="body">
    <form class="card name" onsubmit={(e) => { e.preventDefault(); void saveName(); }}>
      <label for="my-name" class="lead">Moje jméno</label>
      <input id="my-name" class="field" type="text" maxlength="32" placeholder="Bez jména" bind:value={name} autocomplete="nickname" />
      <p class="muted small">Přidává se k vašim zprávám, aby vás kontakty poznaly. Vidí ho jen ten, komu napíšete (zprávy jsou šifrované); sítí ani servery se nešíří. Nové jméno se objeví u dalších zpráv.</p>
      <button class="btn primary" type="submit" disabled={name.trim() === savedName}>{nameSaved ? "Uloženo" : "Uložit jméno"}</button>
    </form>
    <div class="id card">
      <p class="lead">Moje adresa</p>
      {#if qrSrc}<img class="qr" src={qrSrc} alt="QR kód adresy" width="240" height="240" />{/if}
      <p class="mono addr selectable">{address}</p>
      <div class="actions">
        <button class="btn" onclick={copy}><Icon name="copy" size={20} />{copied ? "Zkopírováno" : "Kopírovat"}</button>
        {#if canShare}<button class="btn primary" onclick={share}><Icon name="share" size={20} />Sdílet</button>{/if}
      </div>
    </div>
    <div class="tip card">
      <Icon name="info" size={20} />
      <p class="muted">Adresa je váš veřejný klíč. Můžete ji sdílet; kdo ji má, vám může napsat, ale nic víc. Kontakt ji naskenuje v části Kontakty › Přidat.</p>
    </div>
  </div>
</section>

<style>
  .page { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .body { overflow-y: auto; display: flex; flex-direction: column; gap: var(--space-4); padding: var(--space-2) var(--space-4) var(--space-5); max-width: 480px; width: 100%; margin: 0 auto; }
  .id { display: flex; flex-direction: column; align-items: center; gap: var(--space-4); padding: var(--space-5) var(--space-4); }
  .lead { margin: 0; font-weight: 600; font-size: var(--fs-lg); }
  .name { display: flex; flex-direction: column; gap: var(--space-3); padding: var(--space-4); align-items: stretch; }
  .small { font-size: var(--fs-sm); margin: 0; }
  .qr { background: #fff; padding: 14px; border-radius: var(--r-md); image-rendering: pixelated; max-width: 100%; height: auto; }
  .addr { overflow-wrap: anywhere; text-align: center; margin: 0; font-size: var(--fs-sm); color: var(--muted); }
  .actions { display: flex; gap: var(--space-3); flex-wrap: wrap; justify-content: center; }
  .tip { display: flex; gap: var(--space-3); padding: var(--space-3) var(--space-4); align-items: flex-start; color: var(--accent); }
  .tip p { margin: 0; font-size: var(--fs-sm); }
</style>
