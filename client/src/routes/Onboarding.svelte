<script lang="ts">
  import { app, createIdentity, errText } from "../lib/app.svelte";

  let usePass = $state(false);
  let pass = $state("");
  let pass2 = $state("");
  let busy = $state(false);

  const weak = $derived(usePass && (pass.length < 8 || pass !== pass2));

  async function create() {
    busy = true;
    try {
      await createIdentity(usePass ? pass : undefined);
    } catch (e) {
      app.error = errText(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="wrap">
  <div class="hero">
    <div class="mark" aria-hidden="true">✺</div>
    <h1>ancord</h1>
    <p class="muted">Šifrované zprávy bez serveru jedné firmy. Klíče zůstávají v tomto zařízení a nikdy se nedostanou do UI.</p>
  </div>

  <div class="card form">
    <label class="opt">
      <input type="checkbox" bind:checked={usePass} />
      <span>
        <strong>Chránit heslem</strong>
        <span class="muted">Na pozadí pak nejdou zprávy dešifrovat, oznámení bude jen „Nová zpráva“ a zprávy se vyzvednou po odemknutí.</span>
      </span>
    </label>
    {#if usePass}
      <input class="field" type="password" placeholder="Heslo (alespoň 8 znaků)" bind:value={pass} autocomplete="new-password" />
      <input class="field" type="password" placeholder="Heslo znovu" bind:value={pass2} autocomplete="new-password" />
    {/if}
    <p class="muted warn">Ztráta zařízení znamená ztrátu identity. Záloha a více zařízení zatím nejsou.</p>
    <button class="btn primary" onclick={create} disabled={busy || weak}>Vytvořit identitu</button>
  </div>
</section>

<style>
  .wrap { height: 100%; overflow-y: auto; display: flex; flex-direction: column; justify-content: center; gap: var(--space-5); padding: calc(var(--space-5) + var(--safe-top)) var(--space-4) calc(var(--space-5) + var(--safe-bottom)); max-width: 480px; margin: 0 auto; }
  .hero { text-align: center; }
  .mark { font-size: 3rem; color: var(--accent); line-height: 1; }
  h1 { margin: var(--space-2) 0; font-size: var(--fs-xl); }
  .hero p { margin: 0; }
  .form { padding: var(--space-4); display: flex; flex-direction: column; gap: var(--space-3); }
  .opt { display: flex; gap: var(--space-3); align-items: flex-start; }
  .opt input { margin-top: 5px; accent-color: var(--accent); width: 18px; height: 18px; }
  .opt span { display: flex; flex-direction: column; gap: 2px; font-size: var(--fs-md); }
  .warn { margin: 0; font-size: var(--fs-sm); }
</style>
