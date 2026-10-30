<script lang="ts">
  import { onMount } from "svelte";
  import { app, unlock, unlockWithBiometric, errText } from "../lib/app.svelte";
  import Icon from "../lib/ui/Icon.svelte";
  import Logo from "../lib/ui/Logo.svelte";

  let pass = $state("");
  let busy = $state(false);
  let showPass = $state(false);
  const bio = $derived(app.identity.passphrase && app.bio.enabled);

  async function viaBio() {
    busy = true;
    try {
      await unlockWithBiometric();
    } catch (e) {
      showPass = true;
      app.error = errText(e);
    } finally {
      busy = false;
    }
  }

  // Při zamčení hned nabídnout biometriku; zrušením se objeví heslo.
  onMount(() => {
    if (bio) void viaBio();
  });

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
  {#if bio}
    <button class="btn primary unlock" onclick={viaBio} disabled={busy}><Icon name="shieldcheck" size={22} />Odemknout biometrikou</button>
    {#if !showPass}
      <button class="btn alt" onclick={() => (showPass = true)}>Použít heslo</button>
    {/if}
  {/if}
  {#if app.identity.passphrase && (!bio || showPass)}
    <form onsubmit={(e) => { e.preventDefault(); void go(); }}>
      <input class="field" type="password" placeholder="Heslo" bind:value={pass} autocomplete="current-password" />
      <button class="btn primary" disabled={busy || !pass}>Odemknout</button>
    </form>
  {:else if !app.identity.passphrase}
    <button class="btn primary unlock" onclick={go} disabled={busy}>Odemknout</button>
  {/if}
</section>

<style>
  .wrap { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--space-3); padding: var(--space-4); }
  h1 { margin: var(--space-2) 0 0; font-size: var(--fs-2xl); letter-spacing: -0.02em; }
  .state { margin: 0 0 var(--space-4); display: inline-flex; align-items: center; gap: 6px; }
  form { display: flex; flex-direction: column; gap: var(--space-3); width: min(320px, 100%); }
  .alt { background: transparent; color: var(--muted); }
  .unlock { min-width: 220px; min-height: 52px; }
</style>
