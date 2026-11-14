<script>
  // Suchergebnisse für Spiele oder Spieler. Tippen wird entprellt, veraltete Antworten werden verworfen.
  import Spielkarte from "../ui/Spielkarte.svelte";
  import Spielerzeile from "../ui/Spielerzeile.svelte";
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

  const spieler = $derived(suche.art === "spieler");
  // „@name“ sucht ohne das @
  const begriff = (q) => (suche.art === "spieler" ? q.trim().replace(/^@+/, "").trim() : q.trim());

  async function suchen(q, art) {
    const id = ++lauf;
    fehler = null;
    if (!q) {
      ergebnis = null;
      laedt = false;
      return;
    }
    laedt = true;
    try {
      const r = art === "spieler" ? await call("user_search", { query: q, cursor: null }) : await call("search", { query: q, page: null });
      if (id === lauf) ergebnis = { ...r, query: q, art };
    } catch (e) {
      if (id === lauf) {
        fehler = e;
        // Alte Treffer gehören zu einem anderen Begriff: nicht unter der neuen Überschrift zeigen.
        if (ergebnis?.query !== q || ergebnis?.art !== art) ergebnis = null;
      }
    } finally {
      if (id === lauf) laedt = false;
    }
  }

  async function mehr() {
    mehrLaedt = true;
    try {
      if (ergebnis.art === "spieler") {
        if (!ergebnis.nextCursor) return;
        const r = await call("user_search", { query: ergebnis.query, cursor: ergebnis.nextCursor });
        const ids = new Set(ergebnis.users.map((u) => u.userId));
        ergebnis = { ...ergebnis, users: [...ergebnis.users, ...r.users.filter((u) => !ids.has(u.userId))], nextCursor: r.nextCursor };
      } else {
        if (!ergebnis.nextPage) return;
        const r = await call("search", { query: ergebnis.query, page: ergebnis.nextPage });
        const ids = new Set(ergebnis.games.map((g) => g.universeId));
        ergebnis = { ...ergebnis, games: [...ergebnis.games, ...r.games.filter((g) => !ids.has(g.universeId))], nextPage: r.nextPage };
      }
    } catch (e) {
      fehler = e;
    } finally {
      mehrLaedt = false;
    }
  }

  $effect(() => {
    const q = begriff(suche.query);
    const art = suche.art;
    const t = setTimeout(() => suchen(q, art), 350);
    return () => clearTimeout(t);
  });

  const f = $derived(fehler ? fehlerText(fehler) : null);
  const q = $derived(begriff(suche.query));
  // Nur Treffer zur aktuellen Art zeigen, solange die neue Suche läuft.
  const passend = $derived(ergebnis?.art === suche.art ? ergebnis : null);
  const liste = $derived(passend ? (spieler ? passend.users : passend.games) : null);
  const weiter = $derived(passend ? (spieler ? passend.nextCursor : passend.nextPage) : null);
  const anzahl = (n) => (spieler ? `${n} Spieler` : `${n} ${n === 1 ? "Spiel" : "Spiele"}`);
</script>

<div class="suche">
  <header>
    <h1>{q ? `Ergebnisse für „${q}“` : "Suchen"}</h1>
    {#if liste && !laedt}<span class="leise zahl">{anzahl(liste.length)}</span>{/if}
  </header>

  <div class="art" role="radiogroup" aria-label="Was suchst du?">
    <button class="chip" role="radio" aria-checked={!spieler} onclick={() => (suche.art = "spiele")}>
      <Icon name="start" size={15} />Spiele
    </button>
    <button class="chip" role="radio" aria-checked={spieler} onclick={() => (suche.art = "spieler")}>
      <Icon name="person" size={15} />Spieler
    </button>
  </div>

  {#if !q}
    {#if spieler}
      <Hinweis icon="person" titel="Wen suchst du?" text="Gib oben einen Benutzernamen, Anzeigenamen oder eine Benutzer-ID ein. Ein Link zum Profil funktioniert auch." />
    {:else}
      <Hinweis icon="suche" titel="Wonach suchst du?" text="Gib oben einen Spielnamen ein. Eine Place-ID oder ein Link zur Spielseite führt direkt zum Spiel. Mit @ vor dem Namen suchst du Spieler." />
    {/if}
  {:else if fehler && !liste?.length}
    <Hinweis art="fehler" titel={f.titel} text={f.hinweis}>
      <button class="knopf" onclick={() => suchen(q, suche.art)}><Icon name="neu" size={16} />Erneut suchen</button>
    </Hinweis>
  {:else if laedt && !liste}
    {#if spieler}
      <div class="leute" aria-hidden="true">
        {#each Array(9) as _, i (i)}<div class="skelett zeile-platz"></div>{/each}
      </div>
    {:else}
      <div class="raster" aria-hidden="true">
        {#each Array(12) as _, i (i)}<div class="skelett platzhalter"></div>{/each}
      </div>
    {/if}
  {:else if liste && liste.length === 0}
    <Hinweis
      icon={spieler ? "person" : "suche"}
      titel="Nichts gefunden"
      text={spieler ? "Prüf die Schreibweise. Mit dem genauen Benutzernamen oder der ID findest du jedes Konto." : "Prüf die Schreibweise oder versuch einen kürzeren Begriff. Eine Place-ID findet das Spiel immer."}
    />
  {:else if liste}
    {#if spieler}
      <ul class="leute" class:veraltet={laedt}>
        {#each liste as u (u.userId)}
          <li><Spielerzeile user={u} /></li>
        {/each}
      </ul>
    {:else}
      <div class="raster" class:veraltet={laedt}>
        {#each liste as game (game.universeId)}
          <div class="zelle"><Spielkarte {game} /></div>
        {/each}
      </div>
    {/if}
    {#if weiter}
      <div class="mehr">
        <button class="knopf" onclick={mehr} disabled={mehrLaedt}>{mehrLaedt ? "Lädt …" : "Mehr laden"}</button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .suche { display: flex; flex-direction: column; gap: 20px; }
  header { display: flex; align-items: baseline; gap: 14px; }
  h1 { overflow-wrap: anywhere; }
  .art { display: flex; gap: 8px; margin-top: -6px; }
  .art .chip { gap: 7px; }
  .raster {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 24px 16px;
    transition: opacity 150ms ease;
  }
  .leute {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 4px 16px;
    transition: opacity 150ms ease;
  }
  .raster.veraltet, .leute.veraltet { opacity: 0.55; }
  .zelle { content-visibility: auto; contain-intrinsic-size: 150px 230px; }
  .platzhalter { aspect-ratio: 1; border-radius: var(--r-l); }
  .zeile-platz { height: 68px; border-radius: var(--r-l); }
  .mehr { display: flex; justify-content: center; }
</style>
