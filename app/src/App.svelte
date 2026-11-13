<script>
  import { onMount } from "svelte";
  import Leiste from "./ui/Leiste.svelte";
  import Dock from "./ui/Dock.svelte";
  import Toasts from "./ui/Toasts.svelte";
  import RechteSpalte from "./ui/RechteSpalte.svelte";
  import Icon from "./ui/Icon.svelte";
  import Start from "./pages/Start.svelte";
  import Suche from "./pages/Suche.svelte";
  import Spiel from "./pages/Spiel.svelte";
  import Bibliothek from "./pages/Bibliothek.svelte";
  import Flags from "./pages/Flags.svelte";
  import Roblox from "./pages/Roblox.svelte";
  import Einstellungen from "./pages/Einstellungen.svelte";
  import { app, init, back, go, eingeklappt, leiste } from "./lib/app.svelte.js";
  import { ladeFlags } from "./lib/flags.svelte.js";
  import { suche } from "./lib/suche.svelte.js";
  import { showWindow, isDemo } from "./lib/bridge.js";

  let bereit = $state(false);
  let suchfeld = $state();
  let inhalt = $state();

  // Neue Seite beginnt oben.
  $effect(() => {
    if (app.route && inhalt) inhalt.scrollTop = 0;
  });

  onMount(async () => {
    try {
      await init();
    } finally {
      bereit = true;
      // Fenster erst zeigen, wenn gezeichnet ist (kein weißer Blitz).
      requestAnimationFrame(() => requestAnimationFrame(showWindow));
    }
    ladeFlags().catch(() => {});
  });

  function tasten(e) {
    const tippt = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement;
    if ((e.ctrlKey && e.key.toLowerCase() === "k") || (e.key === "/" && !tippt)) {
      e.preventDefault();
      suchfeld?.focus();
      suchfeld?.select();
    } else if (e.altKey && e.key === "ArrowLeft") {
      e.preventDefault();
      back();
    } else if (e.key === "Escape" && tippt && e.target === suchfeld) {
      suchfeld.blur();
    }
  }

  function maus(e) {
    // Maustaste „Zurück“
    if (e.button === 3) {
      e.preventDefault();
      back();
    }
  }

  function sucheTippen(e) {
    suche.query = e.currentTarget.value;
    if (app.route.name !== "suche") go("suche");
  }
</script>

<svelte:window onkeydown={tasten} onmouseup={maus} />

{#if bereit}
  <div class="shell" class:zu={eingeklappt()} class:breit={leiste.breite >= 1360}>
    <Leiste />
    <div class="haupt">
      <header class="kopf">
        <button class="knopf flach rund" onclick={back} disabled={app.history.length === 0} aria-label="Zurück" title="Zurück (Alt+←)">
          <Icon name="zurueck" size={18} />
        </button>
        <label class="suche">
          <Icon name="suche" size={16} />
          <input
            bind:this={suchfeld}
            type="search"
            placeholder="Spiel suchen oder Place-ID / Link einfügen"
            value={suche.query}
            oninput={sucheTippen}
            onfocus={() => app.route.name !== "suche" && suche.query && go("suche")}
            spellcheck="false"
            autocomplete="off"
          />
          <kbd>Strg K</kbd>
        </label>
        {#if isDemo}<span class="marke warnung" title="Im Browser ohne Tauri: alle Daten sind erfunden">Demo-Daten</span>{/if}
      </header>
      <main class="inhalt" bind:this={inhalt}>
        {#key app.route}
          <div class="seite">
            {#if app.route.name === "start"}<Start />
            {:else if app.route.name === "suche"}<Suche />
            {:else if app.route.name === "spiel"}<Spiel params={app.route.params} />
            {:else if app.route.name === "bibliothek"}<Bibliothek />
            {:else if app.route.name === "flags"}<Flags />
            {:else if app.route.name === "roblox"}<Roblox />
            {:else if app.route.name === "einstellungen"}<Einstellungen />
            {/if}
          </div>
        {/key}
      </main>
    </div>
    {#if leiste.breite >= 1360}<RechteSpalte />{/if}
    <Dock />
  </div>
  <Toasts />
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: var(--leiste) 1fr;
    /* nur zwei feste Zustände, kein animiertes Breitenspiel */
    grid-template-rows: 1fr var(--dock);
    height: 100vh;
  }
  .shell.zu { --leiste: 68px; }
  .shell.breit { grid-template-columns: var(--leiste) 1fr auto; }
  .haupt { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
  .kopf {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 60px;
    padding: 0 24px 0 16px;
    flex: none;
  }
  .suche {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    max-width: 620px;
    height: 38px;
    padding: 0 10px 0 12px;
    border-radius: 10px;
    background: var(--s1);
    border: 1px solid var(--linie-stark);
    color: var(--t3);
    transition: border-color 120ms ease, background-color 120ms ease;
  }
  .suche:hover { border-color: var(--linie-hover); }
  .suche:focus-within { border-color: var(--t2); background: var(--s2); color: var(--t2); }
  .suche input {
    flex: 1;
    min-width: 0;
    height: 100%;
    border: 0;
    background: transparent;
    color: var(--t1);
    font: inherit;
    outline: none;
  }
  .suche input::placeholder { color: var(--t3); }
  .suche input::-webkit-search-cancel-button { display: none; }
  kbd {
    font: inherit;
    font-size: 11.5px;
    font-weight: 600;
    color: var(--t3);
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--s3);
  }
  .inhalt { flex: 1; min-height: 0; overflow-y: auto; overflow-x: hidden; scrollbar-gutter: stable; }
  .seite {
    --seite-rand: 28px;
    padding: 8px var(--seite-rand) 40px;
    max-width: 1600px;
    animation: seite-rein 180ms var(--ein);
  }
  @keyframes seite-rein { from { opacity: 0; transform: translateY(4px); } }
</style>
