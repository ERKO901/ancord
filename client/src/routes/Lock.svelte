<script lang="ts">
  import { app, unlock, errText } from "../lib/app.svelte";
  import Icon from "../lib/ui/Icon.svelte";
  import Logo from "../lib/ui/Logo.svelte";

  let pass = $state("");
  let busy = $state(false);
  async function go() {
    busy = true;
    try {
      await unlock(app.identity.passphrase ? pass : undefined);
      pass = "";
    } catch (e) {
      app.error = errText(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="wrap">
  <Logo size={72} />
  <h1>ancord</h1>
  <p class="muted state"><Icon name="lock" size={18} />Zamčeno</p>
  {#if app.identity.passphrase}
    <form onsubmit={(e) => { e.preventDefault(); void go(); }}>
      <input class="field" type="password" placeholder="Heslo" bind:value={pass} autocomplete="current-password" />
      <button class="btn primary" disabled={busy || !pass}>Odemknout</button>
    </form>
  {:else}
    <button class="btn primary unlock" onclick={go} disabled={busy}>Odemknout</button>
  {/if}
</section>

<style>
  .wrap { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--space-3); padding: var(--space-4); }
  h1 { margin: var(--space-2) 0 0; font-size: var(--fs-2xl); letter-spacing: -0.02em; }
  .state { margin: 0 0 var(--space-4); display: inline-flex; align-items: center; gap: 6px; }
  form { display: flex; flex-direction: column; gap: var(--space-3); width: min(320px, 100%); }
  .unlock { min-width: 220px; min-height: 52px; }
</style>
