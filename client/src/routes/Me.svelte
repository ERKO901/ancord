<script lang="ts">
  import { app, go, errText } from "../lib/app.svelte";
  import { call } from "../lib/ipc";
  import Header from "../lib/ui/Header.svelte";
  import Icon from "../lib/ui/Icon.svelte";

  let address = $state("");
  let qr = $state("");
  let copied = $state(false);

  $effect(() => {
    call("address_get").then((a) => (address = a), (e) => (app.error = errText(e)));
    call("address_qr").then((s) => (qr = s), (e) => (app.error = errText(e)));
  });

  // QR je SVG vytvořené v Rustu; do UI jde jako obrázek (data:), ne jako vložené HTML.
  const qrSrc = $derived(qr ? `data:image/svg+xml;utf8,${encodeURIComponent(qr)}` : "");

  async function copy() {
    await navigator.clipboard.writeText(address);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<section class="page">
  <Header title="Moje adresa" onback={() => go("list")} />
  <div class="body">
    {#if qrSrc}<img class="qr" src={qrSrc} alt="QR kód adresy" width="224" height="224" />{/if}
    <p class="mono addr selectable">{address}</p>
    <button class="btn" onclick={copy}><Icon name="copy" size={20} />{copied ? "Zkopírováno" : "Kopírovat adresu"}</button>
    <p class="muted small">Adresa je veřejný klíč. Můžete ji sdílet; kdo ji má, vám může napsat, ale nic víc.</p>
  </div>
</section>

<style>
  .page { display: flex; flex-direction: column; height: 100%; }
  .body { display: flex; flex-direction: column; align-items: center; gap: var(--space-4); padding: var(--space-4); max-width: 480px; margin: 0 auto; }
  .qr { background: #fff; padding: 12px; border-radius: var(--r-md); image-rendering: pixelated; }
  .addr { overflow-wrap: anywhere; text-align: center; margin: 0; font-size: var(--fs-sm); }
  .small { font-size: var(--fs-sm); margin: 0; text-align: center; }
</style>
