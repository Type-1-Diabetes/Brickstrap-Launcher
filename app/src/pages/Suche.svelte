<script>
  // Suchergebnisse. Tippen wird entprellt, veraltete Antworten werden verworfen.
  import Spielkarte from "../ui/Spielkarte.svelte";
  import Hinweis from "../ui/Hinweis.svelte";
  import Icon from "../ui/Icon.svelte";
  import { call } from "../lib/bridge.js";
  import { fehlerText } from "../lib/errors.js";
  import { suche } from "../lib/suche.svelte.js";

  let ergebnis = $state(null);
  let fehler = $state(null);
  let laedt = $state(false);
  let mehrLaedt = $state(false);
  let lauf = 0;

  async function suchen(q) {
    const id = ++lauf;
    fehler = null;
    if (!q.trim()) {
      ergebnis = null;
      laedt = false;
      return;
    }
    laedt = true;
    try {
      const r = await call("search", { query: q, page: null });
      if (id === lauf) ergebnis = { ...r, query: q };
    } catch (e) {
      if (id === lauf) fehler = e;
    } finally {
      if (id === lauf) laedt = false;
    }
  }

  async function mehr() {
    if (!ergebnis?.nextPage) return;
    mehrLaedt = true;
    try {
      const r = await call("search", { query: ergebnis.query, page: ergebnis.nextPage });
      const ids = new Set(ergebnis.games.map((g) => g.universeId));
      ergebnis = { ...ergebnis, games: [...ergebnis.games, ...r.games.filter((g) => !ids.has(g.universeId))], nextPage: r.nextPage };
    } catch (e) {
      fehler = e;
    } finally {
      mehrLaedt = false;
    }
  }

  $effect(() => {
    const q = suche.query;
    const t = setTimeout(() => suchen(q), 350);
    return () => clearTimeout(t);
  });

  const f = $derived(fehler ? fehlerText(fehler) : null);
</script>

<div class="suche">
  <header>
    <h1>{suche.query.trim() ? `Ergebnisse für „${suche.query.trim()}“` : "Suchen"}</h1>
    {#if ergebnis && !laedt}<span class="leise zahl">{ergebnis.games.length} Spiele</span>{/if}
  </header>

  {#if !suche.query.trim()}
    <Hinweis icon="suche" titel="Wonach suchst du?" text="Gib oben einen Spielnamen ein. Eine Place-ID oder ein Link zur Spielseite funktioniert auch und führt direkt zum Spiel." />
  {:else if fehler && !ergebnis?.games?.length}
    <Hinweis art="fehler" titel={f.titel} text={f.hinweis}>
      <button class="knopf" onclick={() => suchen(suche.query)}><Icon name="neu" size={16} />Erneut suchen</button>
    </Hinweis>
  {:else if laedt && !ergebnis}
    <div class="raster" aria-hidden="true">
      {#each Array(12) as _, i (i)}<div class="skelett platzhalter"></div>{/each}
    </div>
  {:else if ergebnis && ergebnis.games.length === 0}
    <Hinweis icon="suche" titel="Nichts gefunden" text="Prüf die Schreibweise oder versuch einen kürzeren Begriff. Eine Place-ID findet das Spiel immer." />
  {:else if ergebnis}
    <div class="raster" class:veraltet={laedt}>
      {#each ergebnis.games as game (game.universeId)}
        <div class="zelle"><Spielkarte {game} /></div>
      {/each}
    </div>
    {#if ergebnis.nextPage}
      <div class="mehr">
        <button class="knopf" onclick={mehr} disabled={mehrLaedt}>{mehrLaedt ? "Lädt …" : "Mehr laden"}</button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .suche { display: flex; flex-direction: column; gap: 20px; }
  header { display: flex; align-items: baseline; gap: 14px; }
  .raster {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 24px 16px;
    transition: opacity 150ms ease;
  }
  .raster.veraltet { opacity: 0.55; }
  .zelle { content-visibility: auto; contain-intrinsic-size: 150px 230px; }
  .platzhalter { aspect-ratio: 1; border-radius: var(--r-l); }
  .mehr { display: flex; justify-content: center; }
</style>
