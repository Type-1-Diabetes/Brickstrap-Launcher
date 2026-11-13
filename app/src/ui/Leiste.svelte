<script>
  // Navigation links: Marke, Hauptziele, angepinnte Spiele, Einstellungen.
  import Icon from "./Icon.svelte";
  import Bild from "./Bild.svelte";
  import zeichen from "./zeichen.svg";
  import wortmarke from "./wortmarke.svg";
  import { app, go, openGame, prefetch, eingeklappt, leisteUmschalten } from "../lib/app.svelte.js";

  const zu = $derived(eingeklappt());

  const ZIELE = [
    { name: "start", label: "Start", icon: "start" },
    { name: "suche", label: "Suchen", icon: "suche" },
    { name: "bibliothek", label: "Bibliothek", icon: "bibliothek" },
    { name: "flags", label: "FastFlags", icon: "flags" },
    { name: "roblox", label: "Roblox-Einstellungen", icon: "regler" },
  ];

  const aktiv = (n) => app.route.name === n;
  const spielAktiv = (id) => app.route.name === "spiel" && app.route.params.universeId === id;
</script>

<nav class="leiste" class:zu aria-label="Hauptnavigation">
  <div class="marke-oben">
    <img class="zeichen" src={zeichen} alt="" width="28" height="28" />
    <img class="wort" src={wortmarke} alt="Brickstrap" height="15" />
    <button class="knopf flach klein rund klappen" onclick={leisteUmschalten} aria-label={zu ? "Leiste ausklappen" : "Leiste einklappen"} aria-expanded={!zu} title={zu ? "Ausklappen" : "Einklappen"}>
      <Icon name="leiste" size={17} />
    </button>
  </div>

  <ul class="ziele">
    {#each ZIELE as z (z.name)}
      <li>
        <button class="eintrag" class:aktiv={aktiv(z.name)} aria-current={aktiv(z.name) ? "page" : undefined} onclick={() => go(z.name)} title={zu ? z.label : undefined}>
          <Icon name={z.icon} size={18} />
          <span class="lbl">{z.label}</span>
        </button>
      </li>
    {/each}
  </ul>

  <div class="gruppe">
    <p class="gruppentitel" id="leiste-angepinnt">Angepinnt</p>
    {#if app.pinned.length === 0 && !zu}
      <p class="leer">Pinne Spiele auf ihrer Seite an, dann stehen sie hier.</p>
    {:else}
      <ul class="pins" aria-labelledby="leiste-angepinnt">
        {#each app.pinned as p (p.universeId)}
          <li>
            <button
              class="eintrag pin"
              class:aktiv={spielAktiv(p.universeId)}
              onclick={() => openGame(p)}
              onpointerenter={() => prefetch(p)}
              title={p.name}
              aria-label={p.name}
            >
              <span class="mini"><Bild universeId={p.universeId} name={p.name} /></span>
              <span class="pinname">{p.name}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <div class="unten">
    <button class="eintrag" class:aktiv={aktiv("einstellungen")} aria-current={aktiv("einstellungen") ? "page" : undefined} onclick={() => go("einstellungen")} title={zu ? "Einstellungen" : undefined}>
      <Icon name="zahnrad" size={18} />
      <span class="lbl">Einstellungen</span>
    </button>
  </div>
</nav>

<style>
  .leiste {
    display: flex;
    flex-direction: column;
    gap: 18px;
    width: 100%;
    padding: 18px 12px 12px;
    overflow: hidden;
    background: var(--s1);
    border-right: 1px solid var(--linie);
    min-height: 0;
  }
  .marke-oben { position: relative; display: flex; align-items: center; gap: 10px; padding: 2px 8px 6px; height: 36px; }
  .klappen { margin-left: auto; color: var(--t3); }
  .lbl, .wort, .pinname, .gruppentitel { transition: opacity 140ms ease; white-space: nowrap; }
  .leer { transition: opacity 140ms ease; }

  /* Eingeklappt: nur Symbole, Namen als Tooltip */
  .zu { padding-left: 10px; padding-right: 10px; }
  .zu .wort, .zu .lbl, .zu .pinname, .zu .leer { opacity: 0; pointer-events: none; width: 0; overflow: hidden; }
  .zu .gruppentitel { opacity: 0; height: 0; padding: 0; overflow: hidden; }
  .zu .marke-oben { flex-direction: column; height: auto; gap: 10px; padding: 2px 0 6px; align-items: center; }
  .zu .klappen { margin-left: 0; }
  .zu .eintrag { justify-content: center; padding: 0; gap: 0; }
  .zu .eintrag.aktiv::before { left: -10px; }
  .zu .pin { padding: 0; }
  .zeichen { display: block; }
  .wort { display: block; opacity: 0.96; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; }
  .eintrag {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    height: 38px;
    padding: 0 10px;
    border: 0;
    border-radius: var(--r);
    background: transparent;
    color: var(--t2);
    font: inherit;
    font-weight: 500;
    text-align: left;
    cursor: default;
    transition: background-color 120ms ease, color 120ms ease;
  }
  .eintrag:hover { background: var(--s2); color: var(--t1); }
  .eintrag.aktiv { background: var(--s3); color: var(--t1); font-weight: 600; }
  .eintrag.aktiv::before {
    content: "";
    position: absolute;
    left: -12px;
    top: 10px;
    bottom: 10px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--t1);
  }
  .gruppe { display: flex; flex-direction: column; gap: 6px; min-height: 0; flex: 1; }
  .gruppentitel { margin: 0; padding: 0 10px; font-size: 12px; font-weight: 600; color: var(--t3); }
  .leer { padding: 2px 10px; font-size: 12.5px; color: var(--t3); }
  .pins { overflow-y: auto; min-height: 0; scrollbar-width: thin; }
  .pin { height: 40px; gap: 10px; padding: 0 8px; }
  .mini { width: 26px; height: 26px; border-radius: 6px; overflow: hidden; flex: none; }
  .mini :global(.platz) { font-size: 10px; }
  .pinname { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .unten { border-top: 1px solid var(--linie); padding-top: 10px; }
</style>
