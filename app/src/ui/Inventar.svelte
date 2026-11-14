<script>
  // Öffentliches Inventar eines Spielers. Abzeichen lassen sich nach Spiel filtern:
  // Dann zeigt die Liste alle Abzeichen des Spiels, erhaltene zuerst, fehlende blass.
  import Icon from "./Icon.svelte";
  import Hinweis from "./Hinweis.svelte";
  import Bild from "./Bild.svelte";
  import { app, openGame } from "../lib/app.svelte.js";
  import { call } from "../lib/bridge.js";
  import { fehlerText } from "../lib/errors.js";
  import { datum, anteil, assetArt } from "../lib/format.js";

  /** @type {{ userId: number, name: string, offen: boolean|null }} */
  let { userId, name, offen } = $props();

  const KATEGORIEN = [
    { id: "abzeichen", titel: "Abzeichen" },
    { id: "accessoires", titel: "Accessoires" },
    { id: "kleidung", titel: "Kleidung" },
    { id: "emotes", titel: "Emotes" },
    { id: "gamepaesse", titel: "Game-Pässe" },
  ];

  let kategorie = $state("abzeichen");
  /** Je Kategorie: { items, games, next, laedt, fehler } */
  let daten = $state({});
  let filter = $state("");
  let spielId = $state("");
  let spielDaten = $state(null);
  let zeigen = $state("alle");

  const aktuell = $derived(daten[kategorie] ?? null);

  async function laden(kat, weiter = false) {
    const alt = daten[kat];
    if (alt?.laedt || (weiter && !alt?.next)) return;
    daten[kat] = { items: alt?.items ?? [], games: alt?.games ?? {}, next: alt?.next ?? null, laedt: true, fehler: null };
    try {
      const cursor = weiter ? alt.next : null;
      const d = daten[kat];
      if (kat === "abzeichen") {
        const r = await call("user_badges", { userId, cursor });
        for (const g of r.games) d.games[g.universeId] = g;
        d.items = [...d.items, ...r.badges];
        d.next = r.nextCursor;
      } else {
        const r = await call("user_inventory", { userId, category: kat, cursor });
        d.items = [...d.items, ...r.items];
        d.next = r.nextCursor;
      }
    } catch (e) {
      daten[kat].fehler = e;
    } finally {
      daten[kat].laedt = false;
    }
  }

  $effect(() => {
    if (offen !== false && !daten[kategorie]) laden(kategorie);
  });

  // Spiele für den Filter: aus den geladenen Abzeichen (mit Anzahl), dazu die angepinnten.
  const spiele = $derived.by(() => {
    const d = daten.abzeichen;
    if (!d) return [];
    const n = {};
    for (const b of d.items) if (b.universeId) n[b.universeId] = (n[b.universeId] ?? 0) + 1;
    return Object.entries(n)
      .map(([id, anzahl]) => ({ ...(d.games[id] ?? { universeId: Number(id), name: `Spiel ${id}` }), anzahl }))
      .toSorted((a, b) => b.anzahl - a.anzahl || a.name.localeCompare(b.name, "de"));
  });
  const angepinnt = $derived(app.pinned.filter((p) => !spiele.some((s) => s.universeId === p.universeId)));
  const gewaehlt = $derived(
    spielId ? (spiele.find((s) => String(s.universeId) === spielId) ?? app.pinned.find((s) => String(s.universeId) === spielId) ?? null) : null,
  );

  // Alle Abzeichen des gewählten Spiels mit Datum (erhalten oder nicht).
  let spielLauf = 0;
  async function spielLaden(id) {
    const lauf = ++spielLauf;
    spielDaten = { liste: null, fehler: null };
    try {
      const liste = await call("user_game_badges", { userId, universeId: Number(id) });
      if (lauf === spielLauf) spielDaten = { liste, fehler: null };
    } catch (e) {
      if (lauf === spielLauf) spielDaten = { liste: null, fehler: e };
    }
  }

  function spielWaehlen(e) {
    spielId = e.currentTarget.value;
    zeigen = "alle";
    if (spielId) spielLaden(spielId);
    else spielDaten = null;
  }

  function kategorieWaehlen(id) {
    kategorie = id;
    filter = "";
  }

  const passt = (text) => !filter.trim() || text.toLowerCase().includes(filter.trim().toLowerCase());

  const sichtbar = $derived.by(() => {
    if (kategorie === "abzeichen" && spielId) {
      const l = spielDaten?.liste ?? [];
      return l
        .filter((b) => (zeigen === "erhalten" ? b.awardedAt : zeigen === "fehlen" ? !b.awardedAt : true))
        .filter((b) => passt(b.name))
        .toSorted((a, b) => (b.awardedAt ? 1 : 0) - (a.awardedAt ? 1 : 0));
    }
    const items = aktuell?.items ?? [];
    if (kategorie === "abzeichen") return items.filter((b) => passt(b.name) || passt(aktuell.games[b.universeId]?.name ?? ""));
    return items.filter((i) => passt(i.name));
  });

  const erhalten = $derived(spielDaten?.liste?.filter((b) => b.awardedAt).length ?? 0);
  const gesamt = $derived(spielDaten?.liste?.length ?? 0);
  const fF = $derived(aktuell?.fehler ? fehlerText(aktuell.fehler) : null);
  const sF = $derived(spielDaten?.fehler ? fehlerText(spielDaten.fehler) : null);
  const titelVon = (id) => KATEGORIEN.find((k) => k.id === id)?.titel ?? "";
</script>

<section class="inventar" aria-label="Inventar">
  <h2>Inventar</h2>

  {#if offen === false}
    <Hinweis icon="info" kompakt titel="Inventar ist privat" text="{name} hat das Inventar nicht freigegeben. Abzeichen und Gegenstände bleiben verborgen." />
  {:else}
    <div class="reiter" role="radiogroup" aria-label="Kategorie">
      {#each KATEGORIEN as k (k.id)}
        <button class="chip" role="radio" aria-checked={kategorie === k.id} onclick={() => kategorieWaehlen(k.id)}>{k.titel}</button>
      {/each}
    </div>

    <div class="werkzeug">
      {#if kategorie === "abzeichen"}
        <label class="wahl">
          <span class="leise">Spiel</span>
          <select class="feld" value={spielId} onchange={spielWaehlen} disabled={!spiele.length && !app.pinned.length}>
            <option value="">Alle Spiele</option>
            {#if spiele.length}
              <optgroup label="Aus den geladenen Abzeichen">
                {#each spiele as s (s.universeId)}<option value={String(s.universeId)}>{s.name} ({s.anzahl})</option>{/each}
              </optgroup>
            {/if}
            {#if angepinnt.length}
              <optgroup label="Angepinnt">
                {#each angepinnt as s (s.universeId)}<option value={String(s.universeId)}>{s.name}</option>{/each}
              </optgroup>
            {/if}
          </select>
        </label>
      {/if}
      <label class="filter">
        <Icon name="suche" size={15} />
        <input
          class="feld"
          type="search"
          placeholder={kategorie === "abzeichen" && !spielId ? "Abzeichen oder Spiel filtern" : `${titelVon(kategorie)} filtern`}
          bind:value={filter}
          spellcheck="false"
          autocomplete="off"
        />
      </label>
    </div>

    {#if kategorie === "abzeichen" && gewaehlt}
      <div class="spielkopf">
        <button class="spielname" onclick={() => openGame(gewaehlt)} title="Spielseite öffnen">
          <span class="spielbild"><Bild universeId={gewaehlt.universeId} name={gewaehlt.name} /></span>
          <strong>{gewaehlt.name}</strong>
          <Icon name="rechts" size={15} />
        </button>
        {#if spielDaten?.liste}
          <div class="fortschritt zahl">
            <span><strong>{erhalten}</strong> von {gesamt} erhalten</span>
            <span class="balken" aria-hidden="true"><span style="transform: scaleX({gesamt ? erhalten / gesamt : 0})"></span></span>
          </div>
          <div class="zeigen" role="radiogroup" aria-label="Anzeigen">
            <button class="chip klein" role="radio" aria-checked={zeigen === "alle"} onclick={() => (zeigen = "alle")}>Alle</button>
            <button class="chip klein" role="radio" aria-checked={zeigen === "erhalten"} onclick={() => (zeigen = "erhalten")}>Erhalten</button>
            <button class="chip klein" role="radio" aria-checked={zeigen === "fehlen"} onclick={() => (zeigen = "fehlen")}>Fehlen</button>
          </div>
        {/if}
      </div>
    {/if}

    {#if kategorie === "abzeichen" && spielId && sF}
      <Hinweis art="fehler" kompakt titel={sF.titel} text={sF.hinweis}>
        <button class="knopf" onclick={() => spielLaden(spielId)}><Icon name="neu" size={16} />Erneut laden</button>
      </Hinweis>
    {:else if (kategorie === "abzeichen" && spielId && !spielDaten?.liste) || (!(kategorie === "abzeichen" && spielId) && (!aktuell || (aktuell.laedt && !aktuell.items.length)))}
      <ul class="raster" aria-hidden="true">
        {#each Array(9) as _, i (i)}<li class="platz"><span class="skelett bild"></span><span class="skelett zeile"></span></li>{/each}
      </ul>
    {:else if !spielId && aktuell?.fehler && !aktuell.items.length}
      <Hinweis art="fehler" kompakt titel={fF.titel} text={kategorie === "abzeichen" ? fF.hinweis : `${titelVon(kategorie)} lassen sich gerade nicht laden. ${fF.hinweis}`}>
        <button class="knopf" onclick={() => laden(kategorie)}><Icon name="neu" size={16} />Erneut laden</button>
      </Hinweis>
    {:else if sichtbar.length === 0}
      <p class="leise leer">
        {#if filter.trim()}Nichts passt zu „{filter.trim()}“.
        {:else if spielId}Dieses Spiel hat keine Abzeichen.
        {:else}{name} hat hier nichts.{/if}
      </p>
    {:else}
      <ul class="raster">
        {#each sichtbar as x (x.id)}
          <li class:fehlt={spielId && kategorie === "abzeichen" && !x.awardedAt} title={x.description || x.name}>
            <span class="bild">{#if x.iconUrl}<img src={x.iconUrl} alt="" loading="lazy" decoding="async" />{/if}</span>
            <span class="tx">
              <strong>{x.name}</strong>
              {#if kategorie === "abzeichen" && spielId}
                <span class="leise zahl">
                  {x.awardedAt
                    ? `Erhalten${datum(x.awardedAt) ? ` am ${datum(x.awardedAt)}` : ""}`
                    : `Fehlt${x.winRate != null ? ` · ${anteil(x.winRate)} der Spieler` : ""}`}
                </span>
              {:else if kategorie === "abzeichen"}
                {#if aktuell.games[x.universeId]}
                  <button class="spiel-link" onclick={() => { spielId = String(x.universeId); zeigen = "alle"; spielLaden(spielId); }} title="Nur Abzeichen dieses Spiels">
                    {aktuell.games[x.universeId].name}
                  </button>
                {:else}
                  <span class="leise">Spiel unbekannt</span>
                {/if}
              {:else}
                <span class="leise">{assetArt(x.kind)}</span>
              {/if}
            </span>
          </li>
        {/each}
      </ul>
    {/if}

    {#if !(kategorie === "abzeichen" && spielId) && aktuell?.next}
      <div class="mehr">
        <button class="knopf" onclick={() => laden(kategorie, true)} disabled={aktuell.laedt}>{aktuell.laedt ? "Lädt …" : "Mehr laden"}</button>
        {#if kategorie === "abzeichen"}<span class="leise">Mehr Abzeichen bringen mehr Spiele in den Filter.</span>{/if}
      </div>
    {/if}
  {/if}
</section>

<style>
  .inventar { display: flex; flex-direction: column; gap: 14px; }
  h2 { font-size: 17px; }
  .reiter, .zeigen { display: flex; flex-wrap: wrap; gap: 8px; }
  .chip.klein { height: 26px; padding: 0 10px; font-size: 12.5px; }

  .werkzeug { display: flex; flex-wrap: wrap; gap: 10px; align-items: center; }
  .wahl { display: flex; align-items: center; gap: 10px; }
  .wahl .leise { font-size: 13px; }
  .wahl select { width: 280px; max-width: 100%; }
  .filter { position: relative; display: flex; align-items: center; flex: 0 1 300px; min-width: 200px; color: var(--t3); }
  .filter :global(.icon) { position: absolute; left: 11px; pointer-events: none; }
  .filter input { width: 100%; padding-left: 34px; }
  .filter input::-webkit-search-cancel-button { display: none; }

  .spielkopf {
    display: flex; flex-wrap: wrap; align-items: center; gap: 12px 24px;
    padding: 12px 16px 12px 12px; border-radius: var(--r-l); background: var(--s1);
  }
  .spielname {
    display: flex; align-items: center; gap: 12px; min-width: 0;
    padding: 0; border: 0; background: none; color: var(--t1); font: inherit; text-align: left; cursor: default;
  }
  .spielname strong { font-weight: 700; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .spielname :global(.icon) { color: var(--t3); transition: transform 160ms var(--ein); }
  .spielname:hover :global(.icon) { transform: translateX(2px); color: var(--t1); }
  .spielname .spielbild { display: block; width: 40px; height: 40px; flex: none; border-radius: var(--r); overflow: hidden; box-shadow: inset 0 0 0 1px var(--rand-bild); }
  .fortschritt { display: flex; align-items: center; gap: 12px; color: var(--t2); font-size: 13px; }
  .fortschritt strong { color: var(--t1); font-weight: 700; }
  .balken { width: 140px; height: 6px; border-radius: 3px; background: var(--s3); overflow: hidden; }
  .balken span {
    display: block; height: 100%; background: var(--t2); transform-origin: left;
    animation: wachsen 420ms var(--ein);
  }
  @keyframes wachsen { from { transform: scaleX(0); } }
  .zeigen { margin-left: auto; }

  .raster { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 4px 16px; }
  .raster li {
    display: flex; align-items: center; gap: 12px; padding: 7px 8px; margin: 0 -8px; border-radius: var(--r); min-width: 0;
    content-visibility: auto; contain-intrinsic-size: 240px 62px;
  }
  .raster li:not(.platz):hover { background: var(--s1); }
  .bild { width: 48px; height: 48px; border-radius: var(--r); background: var(--s2); overflow: hidden; flex: none; box-shadow: inset 0 0 0 1px var(--rand-bild); }
  .bild img { display: block; width: 100%; height: 100%; object-fit: cover; }
  .fehlt .bild { opacity: 0.4; filter: grayscale(1); }
  .fehlt strong { color: var(--t3); }
  .tx { display: flex; flex-direction: column; gap: 1px; min-width: 0; align-items: flex-start; }
  .tx strong { font-weight: 600; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .tx > span, .spiel-link { font-size: 12.5px; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .spiel-link {
    padding: 0; border: 0; background: none; font: inherit; font-size: 12.5px; color: var(--t3); text-align: left; cursor: default;
    text-decoration: underline; text-decoration-color: transparent; text-underline-offset: 2px;
    transition: color 120ms ease, text-decoration-color 120ms ease;
  }
  .spiel-link:hover { color: var(--t1); text-decoration-color: currentColor; }
  .platz .zeile { height: 12px; width: 60%; }
  .leer { padding: 6px 0; }
  .mehr { display: flex; align-items: center; gap: 14px; flex-wrap: wrap; }
  .mehr .leise { font-size: 13px; }
</style>
