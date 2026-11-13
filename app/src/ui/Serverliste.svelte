<script>
  // Server-Picker: sortierbare Tabelle öffentlicher Server mit Ein-Klick-Beitritt.
  // Die erste Seite kommt oft schon aus dem Vorladen (Hover auf der Kachel).
  import Icon from "./Icon.svelte";
  import Hinweis from "./Hinweis.svelte";
  import { app, firstServers, launch, starting, saveConfig, toast } from "../lib/app.svelte.js";
  import { call } from "../lib/bridge.js";
  import { fehlerText } from "../lib/errors.js";
  import { seit } from "../lib/format.js";

  /** @type {{ game: { universeId: number, placeId: number, name: string } }} */
  let { game } = $props();

  const SORTEN = [
    { id: "voll", label: "Meiste Spieler" },
    { id: "leer", label: "Wenigste Spieler" },
    { id: "ping", label: "Niedrigster Ping" },
  ];

  let sorte = $state("voll");
  let ohneVolle = $state(app.config?.hideFullServers ?? true);
  let server = $state([]);
  let cursor = $state(null);
  let laedt = $state(true);
  let mehrLaedt = $state(false);
  let fehler = $state(null);
  let stand = $state(0);
  let lauf = 0;

  // Roblox sortiert nur nach Spielerzahl; Ping sortieren wir selbst.
  const order = $derived(sorte === "leer" ? "Asc" : "Desc");

  async function laden(neu = false) {
    const id = ++lauf;
    laedt = true;
    fehler = null;
    try {
      const p = neu
        ? call("servers", { placeId: game.placeId, cursor: null, order, excludeFull: ohneVolle })
        : firstServers(game.placeId, order, ohneVolle);
      const r = await p;
      if (id !== lauf) return;
      server = r.servers;
      cursor = r.nextCursor;
      stand = Math.floor(Date.now() / 1000);
    } catch (e) {
      if (id === lauf) fehler = e;
    } finally {
      if (id === lauf) laedt = false;
    }
  }

  async function mehr() {
    if (!cursor) return;
    mehrLaedt = true;
    try {
      const r = await call("servers", { placeId: game.placeId, cursor, order, excludeFull: ohneVolle });
      const ids = new Set(server.map((s) => s.id));
      server = [...server, ...r.servers.filter((s) => !ids.has(s.id))];
      cursor = r.nextCursor;
    } catch (e) {
      const f = fehlerText(e);
      toast(f.titel, "fehler", f.hinweis);
    } finally {
      mehrLaedt = false;
    }
  }

  $effect(() => {
    // neu laden, wenn Sortierung (Spielerzahl) oder Filter wechselt
    if (order && typeof ohneVolle === "boolean") laden();
  });

  function filterUmschalten() {
    ohneVolle = !ohneVolle;
    saveConfig({ hideFullServers: ohneVolle }).catch(() => {});
  }

  const sichtbar = $derived(
    sorte === "ping" ? server.toSorted((a, b) => (a.ping ?? 9999) - (b.ping ?? 9999)) : server,
  );

  const kleinster = $derived(
    server.filter((s) => s.playing < s.maxPlayers).reduce((m, s) => (!m || s.playing < m.playing ? s : m), null),
  );
  const schnellster = $derived(
    server.filter((s) => s.playing < s.maxPlayers && s.ping != null).reduce((m, s) => (!m || s.ping < m.ping ? s : m), null),
  );

  // Signalbalken 1–3 aus dem Server-Ping (Messpunkt laut Roblox, nicht dein eigener Ping).
  const balken = (ping) => (ping == null ? 0 : ping < 70 ? 3 : ping < 140 ? 2 : 1);

  async function idKopieren(id) {
    await navigator.clipboard.writeText(id);
    toast("Server-ID kopiert");
  }

  const f = $derived(fehler ? fehlerText(fehler) : null);
  const kurzId = (id) => id.slice(0, 8);
</script>

<section class="server">
  <header class="kopf">
    <div class="titel">
      <h2>Server</h2>
      {#if !laedt || server.length}
        <span class="leise zahl">
          {server.length}{cursor ? "+" : ""} geladen{#if stand}&nbsp;· Stand {seit(stand)}{/if}
        </span>
      {/if}
    </div>
    <div class="schnell">
      <button class="knopf klein" disabled={!kleinster || !!starting.key} onclick={() => launch(game, kleinster.id)} title="Freier Server mit den wenigsten Spielern">
        <Icon name="personen" size={15} />Kleinster Server
      </button>
      <button class="knopf klein" disabled={!schnellster || !!starting.key} onclick={() => launch(game, schnellster.id)} title="Freier Server mit dem niedrigsten Server-Ping">
        <Icon name="blitz" size={15} />Bester Ping
      </button>
    </div>
  </header>

  <div class="werkzeug">
    <div class="sorten" role="radiogroup" aria-label="Sortierung">
      {#each SORTEN as s (s.id)}
        <button class="chip" role="radio" aria-checked={sorte === s.id} onclick={() => (sorte = s.id)}>{s.label}</button>
      {/each}
    </div>
    <div class="rechts">
      <label class="filter">
        <span class="schalter"><input type="checkbox" checked={ohneVolle} onchange={filterUmschalten} /><span></span></span>
        Volle ausblenden
      </label>
      <button class="knopf flach klein" onclick={() => laden(true)} disabled={laedt} aria-label="Serverliste neu laden">
        <span class="dreh" class:aktiv={laedt}><Icon name="neu" size={15} /></span>Neu laden
      </button>
    </div>
  </div>

  {#if fehler && !server.length}
    <Hinweis art="fehler" titel={f.titel} text={f.hinweis}>
      <button class="knopf" onclick={() => laden(true)}><Icon name="neu" size={16} />Erneut laden</button>
    </Hinweis>
  {:else if laedt && !server.length}
    <div class="tabelle" aria-hidden="true">
      {#each Array(8) as _, i (i)}<div class="zeile platz"><div class="skelett" style="height: 14px; width: {40 + ((i * 37) % 45)}%"></div></div>{/each}
    </div>
  {:else if !server.length}
    <Hinweis icon="personen" titel="Keine passenden Server" text={ohneVolle ? "Alle Server sind gerade voll. Blende volle Server ein oder starte mit „Spielen“, dann wartet Roblox auf einen Platz." : "Gerade läuft kein öffentlicher Server. „Spielen“ startet einen neuen."} />
  {:else}
    <div class="tabelle" class:veraltet={laedt} role="table" aria-label="Öffentliche Server">
      <div class="zeile titelzeile" role="row">
        <span role="columnheader">Spieler</span>
        <span role="columnheader">Auslastung</span>
        <span role="columnheader" title="Ping des Servers laut Roblox. Dein eigener Ping kann abweichen.">Server-Ping</span>
        <span role="columnheader">FPS</span>
        <span role="columnheader">Server-ID</span>
        <span role="columnheader"><span class="unsichtbar">Aktion</span></span>
      </div>
      {#each sichtbar as s, i (s.id)}
        {@const voll = s.playing >= s.maxPlayers}
        <div class="zeile" class:voll role="row" style="--i: {Math.min(i, 12)}">
          <span class="spieler zahl" role="cell"><strong>{s.playing}</strong><span class="leise">/{s.maxPlayers}</span></span>
          <span class="auslastung" role="cell">
            <span class="spur"><span class="fuellung" style="transform: scaleX({Math.min(1, s.playing / Math.max(1, s.maxPlayers))})"></span></span>
          </span>
          <span class="ping zahl" role="cell">
            <span class="signal" data-stufe={balken(s.ping)} aria-hidden="true"><i></i><i></i><i></i></span>
            {s.ping != null ? `${s.ping} ms` : "–"}
          </span>
          <span class="fps zahl leise" role="cell">{s.fps != null ? Math.round(s.fps) : "–"}</span>
          <span class="id" role="cell">
            <button class="idknopf" onclick={() => idKopieren(s.id)} title="Server-ID kopieren: {s.id}">
              <code>{kurzId(s.id)}</code><Icon name="kopieren" size={13} />
            </button>
          </span>
          <span class="aktion" role="cell">
            <button
              class="knopf klein beitreten"
              disabled={voll || !!starting.key}
              onclick={() => launch(game, s.id)}
            >
              {voll ? "Voll" : starting.key === `${game.placeId}|${s.id}` ? "Startet …" : "Beitreten"}
            </button>
          </span>
        </div>
      {/each}
    </div>
    {#if cursor}
      <div class="mehr">
        <button class="knopf" onclick={mehr} disabled={mehrLaedt}>{mehrLaedt ? "Lädt …" : "Weitere Server laden"}</button>
      </div>
    {:else if server.length >= 600}
      <p class="leise klein">Roblox liefert höchstens etwa 700 Server je Spiel.</p>
    {/if}
  {/if}
</section>

<style>
  .server { display: flex; flex-direction: column; gap: 14px; }
  .kopf { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; }
  .titel { display: flex; align-items: baseline; gap: 12px; }
  .titel .leise { font-size: 13px; }
  .schnell { display: flex; gap: 8px; }
  .werkzeug { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; }
  .sorten { display: flex; gap: 6px; flex-wrap: wrap; }
  .rechts { display: flex; align-items: center; gap: 14px; }
  .filter { display: flex; align-items: center; gap: 10px; color: var(--t2); font-size: 13px; }
  .dreh { display: inline-flex; }
  .dreh.aktiv { animation: drehen 0.8s linear infinite; }
  @keyframes drehen { to { transform: rotate(360deg); } }

  .tabelle {
    display: flex;
    flex-direction: column;
    border-radius: var(--r-l);
    background: var(--s1);
    overflow: clip;
    transition: opacity 150ms ease;
  }
  .tabelle.veraltet { opacity: 0.6; }
  .zeile {
    display: grid;
    grid-template-columns: 92px minmax(120px, 1.4fr) 128px 56px minmax(110px, 1fr) 112px;
    align-items: center;
    gap: 16px;
    min-height: 48px;
    padding: 0 12px 0 18px;
    border-top: 1px solid var(--linie);
    content-visibility: auto;
    contain-intrinsic-size: auto 48px;
    transition: background-color 100ms ease;
  }
  .zeile:first-child, .titelzeile + .zeile { border-top: 0; }
  .zeile:not(.titelzeile):hover { background: var(--s2); }
  .zeile.platz { grid-template-columns: 1fr; }
  .titelzeile {
    position: sticky;
    top: 0;
    z-index: 1;
    border-bottom: 1px solid var(--linie);
    min-height: 38px;
    font-size: 12px;
    font-weight: 600;
    color: var(--t3);
    background: var(--s1);
  }
  .spieler strong { font-size: 15px; font-weight: 700; }
  .voll .spieler strong, .voll .ping { color: var(--t3); }

  .spur { display: block; height: 6px; border-radius: 3px; background: var(--s3); overflow: hidden; }
  .fuellung {
    display: block;
    height: 100%;
    background: var(--t2);
    transform-origin: left center;
    animation: fuellen 420ms var(--ein) both;
    animation-delay: calc(var(--i) * 18ms);
  }
  .voll .fuellung { background: var(--t4); }
  @keyframes fuellen { from { transform: scaleX(0); } }

  .ping { display: flex; align-items: center; gap: 8px; }
  .signal { display: inline-flex; align-items: flex-end; gap: 2px; height: 12px; }
  .signal i { width: 3px; border-radius: 1px; background: var(--s4); }
  .signal i:nth-child(1) { height: 5px; }
  .signal i:nth-child(2) { height: 8px; }
  .signal i:nth-child(3) { height: 12px; }
  .signal[data-stufe="1"] i:nth-child(-n + 1),
  .signal[data-stufe="2"] i:nth-child(-n + 2),
  .signal[data-stufe="3"] i { background: var(--t1); }

  .idknopf {
    display: inline-flex; align-items: center; gap: 6px;
    padding: 3px 6px; margin-left: -6px; border: 0; border-radius: 5px;
    background: none; color: var(--t3); font: inherit; cursor: default;
    transition: background-color 100ms ease, color 100ms ease;
  }
  .idknopf:hover { background: var(--s3); color: var(--t1); }
  .idknopf :global(.icon) { opacity: 0; transition: opacity 100ms ease; }
  .zeile:hover .idknopf :global(.icon), .idknopf:focus-visible :global(.icon) { opacity: 1; }
  code { font-family: var(--mono); font-size: 12.5px; }

  .aktion { display: flex; justify-content: flex-end; }
  .beitreten { min-width: 96px; }
  .zeile:hover .beitreten:not(:disabled) { --bg: var(--primaer); --bg-h: var(--primaer-hover); --fg: var(--auf-primaer); }

  .mehr { display: flex; justify-content: center; }
  .klein { font-size: 12.5px; }
  .unsichtbar { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }
</style>
