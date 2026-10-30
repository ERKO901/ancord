<script lang="ts">
  import { onMount } from "svelte";
  import { app, boot, lockNow, errText, isTopLevel } from "./lib/app.svelte";
  import Chat from "./routes/Chat.svelte";
  import ConversationList from "./routes/ConversationList.svelte";
  import Contacts from "./routes/Contacts.svelte";
  import ContactDetail from "./routes/ContactDetail.svelte";
  import AddContact from "./routes/AddContact.svelte";
  import Me from "./routes/Me.svelte";
  import Settings from "./routes/Settings.svelte";
  import About from "./routes/About.svelte";
  import TabBar from "./lib/ui/TabBar.svelte";
  import Logo from "./lib/ui/Logo.svelte";
  import Onboarding from "./routes/Onboarding.svelte";
  import Lock from "./routes/Lock.svelte";

  onMount(() => {
    boot().catch((e) => { app.error = errText(e); app.ready = true; });
  });

  // Téma: „system“ sleduje prefers-color-scheme.
  $effect(() => {
    const choice = app.settings?.theme ?? "dark";
    const mq = matchMedia("(prefers-color-scheme: light)");
    const apply = () => {
      document.documentElement.dataset["theme"] = choice === "system" ? (mq.matches ? "light" : "dark") : choice;
    };
    apply();
    mq.addEventListener("change", apply);
    return () => mq.removeEventListener("change", apply);
  });

  // Zámek po nečinnosti: UI se zamkne, obsah zpráv z paměti UI zmizí (`lockNow`).
  $effect(() => {
    const secs = app.settings?.lockAfterSecs ?? 0;
    if (app.identity.state !== "ready" || secs <= 0) return;
    let timer: ReturnType<typeof setTimeout>;
    const arm = () => {
      clearTimeout(timer);
      timer = setTimeout(() => void lockNow(), secs * 1000);
    };
    const events = ["pointerdown", "keydown", "wheel", "touchstart"] as const;
    for (const e of events) addEventListener(e, arm, { passive: true });
    arm();
    return () => {
      clearTimeout(timer);
      for (const e of events) removeEventListener(e, arm);
    };
  });

  $effect(() => {
    if (!app.error) return;
    const t = setTimeout(() => (app.error = null), 5000);
    return () => clearTimeout(t);
  });

  const hasMain = $derived(app.view !== "list");
  const top = $derived(isTopLevel(app.view));
</script>

{#if !app.ready}
  <div class="boot" aria-busy="true"><Logo size={72} /></div>
{:else if app.identity.state === "none"}
  <Onboarding />
{:else if app.identity.state === "locked"}
  <Lock />
{:else}
  <div class="shell" class:has-main={hasMain} class:top>
    <aside class="side"><ConversationList /></aside>
    <main class="main">
      {#key app.view}
        <div class="page-in">
          {#if app.view === "chat"}<Chat />
          {:else if app.view === "contacts"}<Contacts />
          {:else if app.view === "contact"}<ContactDetail />
          {:else if app.view === "add"}<AddContact />
          {:else if app.view === "me"}<Me />
          {:else if app.view === "settings"}<Settings />
          {:else if app.view === "about"}<About />
          {:else}
            <div class="placeholder muted"><Logo size={64} tile={false} /><h2>ancord</h2><p>Vyberte konverzaci nebo začněte novou.</p></div>
          {/if}
        </div>
      {/key}
    </main>
    <div class="nav"><TabBar /></div>
  </div>
{/if}

{#if app.error}
  <div class="toast" role="alert">{app.error}</div>
{/if}

<style>
  .boot { height: 100%; display: grid; place-items: center; }
  .shell { display: grid; grid-template-columns: 1fr; grid-template-rows: minmax(0, 1fr) auto; height: 100%; }
  .side, .main { min-width: 0; min-height: 0; height: 100%; }
  /* Úzké okno (telefon): jeden sloupec, buď seznam, nebo detail; spodní navigace jen na nejvyšší úrovni. */
  .shell.has-main .side { display: none; }
  .shell:not(.has-main) .main { display: none; }
  .shell:not(.top) .nav { display: none; }
  .main { position: relative; }
  .page-in { height: 100%; animation: page-in 0.18s ease-out; }
  @keyframes page-in { from { opacity: 0; transform: translateY(6px); } to { opacity: 1; transform: none; } }
  .placeholder { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--space-2); }
  .placeholder h2 { margin: var(--space-2) 0 0; font-size: var(--fs-xl); color: var(--text); }
  .placeholder p { margin: 0; }

  /* Desktop a tablet: seznam vedle detailu, navigace pod seznamem. */
  @media (min-width: 900px) {
    .shell, .shell.has-main { grid-template-columns: 360px 1fr; }
    .shell.has-main .side, .shell:not(.has-main) .main { display: block; }
    .shell:not(.top) .nav { display: block; }
    .side { grid-column: 1; grid-row: 1; border-right: 1px solid var(--border); }
    .nav { grid-column: 1; grid-row: 2; border-right: 1px solid var(--border); }
    .main { grid-column: 2; grid-row: 1 / span 2; }
  }

  .toast {
    position: fixed; left: 50%; bottom: calc(var(--tabbar) + 24px + var(--safe-bottom)); transform: translateX(-50%);
    max-width: min(92vw, 420px); padding: var(--space-3) var(--space-4);
    background: var(--surface-2); border: 1px solid var(--danger); border-radius: var(--r-md);
    box-shadow: var(--shadow-lg); font-size: var(--fs-md); z-index: 10;
  }
</style>
