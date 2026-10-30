<script lang="ts">
  import { app, unlock, errText } from "../lib/app.svelte";
  import Icon from "../lib/ui/Icon.svelte";

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
  <div class="mark"><Icon name="lock" size={40} /></div>
  <h1>Zamčeno</h1>
  {#if app.identity.passphrase}
    <form onsubmit={(e) => { e.preventDefault(); void go(); }}>
      <input class="field" type="password" placeholder="Heslo" bind:value={pass} autocomplete="current-password" />
      <button class="btn primary" disabled={busy || !pass}>Odemknout</button>
    </form>
  {:else}
    <button class="btn primary" onclick={go} disabled={busy}>Odemknout</button>
  {/if}
</section>

<style>
  .wrap { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--space-4); padding: var(--space-4); }
  .mark { color: var(--accent); }
  h1 { margin: 0; font-size: var(--fs-xl); }
  form { display: flex; flex-direction: column; gap: var(--space-3); width: min(320px, 100%); }
</style>
