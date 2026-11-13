<script>
  // Startseite: angepinnte Spiele groß, zuletzt gespielt, Reihen von Roblox.
  // Zuerst aus dem Cache zeichnen, dann im Hintergrund aktualisieren.
  import { onMount } from "svelte";
  import Reihe from "../ui/Reihe.svelte";
  import Bild from "../ui/Bild.svelte";
  import Icon from "../ui/Icon.svelte";
  import Hinweis from "../ui/Hinweis.svelte";
  import { app, openGame, prefetch, launch, starting, go } from "../lib/app.svelte.js";
  import { call } from "../lib/bridge.js";
  import { fehlerText } from "../lib/errors.js";
  import { seit, zahl } from "../lib/format.js";

  let feed = $state(null);
  let fehler = $state(null);
  let laedt = $state(true);
  let zuletzt = $state([]);
  let online = $state({});

  async function laden() {
    laedt = true;
    fehler = null;
    try {
      const frisch = await call("home_feed", { fresh: true });
      if (frisch) feed = frisch;
    } catch (e) {
      fehler = e;
    } finally {
      laedt = false;
    }
  }

  onMount(async () => {
    try {
      feed = await call("home_feed", { fresh: false });
    } catch {
      // Ohne Cache einfach frisch laden.
    }
    laden();
    if (app.pinned.length) {
      call("game_details", { universeIds: app.pinned.slice(0, 6).map((p) => p.universeId) })
        .then((list) => (online = Object.fromEntries(list.map((d) => [d.universeId, d.playing]))))
        .catch(() => {});
    }
    call("playtime_get")
      .then((p) => (zuletzt = (p?.games ?? []).toSorted((a, b) => b.lastPlayed - a.lastPlayed).slice(0, 12)))
      .catch(() => {});
  });

  const f = $derived(fehler ? fehlerText(fehler) : null);
</script>

<div class="start">
  <section class="angepinnt">
    <header>
      <h1>Angepinnt</h1>
      {#if app.pinned.length}
        <button class="knopf flach klein" onclick={() => go("bibliothek")}>Bibliothek <Icon name="rechts" size={14} /></button>
      {/if}
    </header>
    {#if app.pinned.length === 0}
      <Hinweis icon="pin" titel="Noch nichts angepinnt" text="Öffne ein Spiel und klick auf „Anpinnen“. Angepinnte Spiele stehen hier groß und in der Leiste links, ein Klick führt zur Serverliste.">
        <button class="knopf" onclick={() => go("suche")}><Icon name="suche" size={16} />Spiel suchen</button>
      </Hinweis>
    {:else}
      <div class="kacheln">
        {#each app.pinned.slice(0, 6) as p (p.universeId)}
          <article class="kachel" onpointerenter={() => prefetch(p)}>
            <button class="flaeche" onclick={() => openGame(p)} aria-label="{p.name} öffnen">
              <Bild universeId={p.universeId} name={p.name} kind="banner" eager />
            </button>
            <div class="fuss">
              <button class="name" onclick={() => openGame(p)}>
                <span class="nm">{p.name}</span>
                <span class="online zahl">
                  {#if online[p.universeId] != null}<span class="punkt" aria-hidden="true"></span>{zahl(online[p.universeId])} online{:else}&nbsp;{/if}
                </span>
              </button>
              <button
                class="knopf klein"
                onclick={() => launch(p)}
                disabled={starting.key === `${p.placeId}|`}
                aria-label="{p.name} spielen"
              >
                <Icon name="spielen" size={13} />Spielen
              </button>
            </div>
          </article>
        {/each}
      </div>
    {/if}
  </section>

  {#if zuletzt.length}
    <Reihe titel="Zuletzt gespielt" games={zuletzt.map((g) => ({ ...g, playing: 0 }))}>
      {#snippet aktion()}<span class="leise klein">{seit(zuletzt[0].lastPlayed)}</span>{/snippet}
    </Reihe>
  {/if}

  {#if feed?.sorts?.length}
    {#each feed.sorts as s (s.id)}
      <Reihe titel={s.title || "Empfohlen"} games={s.games} />
    {/each}
    {#if fehler}
      <p class="stand leise">Stand {seit(feed.fetchedAt)}. {f.titel}. <button class="link" onclick={laden}>Erneut versuchen</button></p>
    {/if}
  {:else if fehler}
    <Hinweis art="fehler" titel={f.titel} text={`${f.hinweis} Angepinnte Spiele und die Suche funktionieren trotzdem.`}>
      <button class="knopf" onclick={laden}><Icon name="neu" size={16} />Erneut laden</button>
    </Hinweis>
  {:else if laedt}
    {#each [0, 1] as i (i)}
      <div class="skelett-reihe" aria-hidden="true">
        <div class="skelett titelzeile"></div>
        <div class="karten">{#each Array(7) as _, j (j)}<div class="skelett karte"></div>{/each}</div>
      </div>
    {/each}
  {/if}
</div>

<style>
  .start { display: flex; flex-direction: column; gap: 36px; }
  .angepinnt { display: flex; flex-direction: column; gap: 16px; }
  .angepinnt header { display: flex; align-items: center; justify-content: space-between; }
  .kacheln {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(236px, 1fr));
    gap: 16px;
  }
  .kachel {
    display: flex;
    flex-direction: column;
    border-radius: var(--r-l);
    background: var(--s1);
    overflow: hidden;
    transition: background-color 160ms ease;
  }
  .kachel:hover { background: var(--s2); }
  .flaeche { display: block; padding: 0; border: 0; background: none; cursor: default; border-radius: 0; }
  .flaeche :global(.bild) { border-radius: 0; transition: opacity 160ms ease; }
  .flaeche:focus-visible { outline-offset: -2px; border-radius: var(--r-l) var(--r-l) 0 0; }
  .fuss { display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 12px 12px 12px 14px; }
  .name {
    display: flex; flex-direction: column; gap: 2px; min-width: 0;
    padding: 0; border: 0; background: none; color: var(--t1); font: inherit; text-align: left; cursor: default;
  }
  .nm { font-weight: 700; font-size: 15px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .online { display: flex; align-items: center; gap: 6px; font-size: 12.5px; color: var(--t3); }
  .punkt { width: 6px; height: 6px; border-radius: 50%; background: var(--live); }
  .klein { font-size: 12.5px; }
  .stand { font-size: 13px; }
  .link { padding: 0; border: 0; background: none; color: var(--t1); font: inherit; text-decoration: underline; text-underline-offset: 3px; text-decoration-color: var(--t4); cursor: default; }
  .skelett-reihe { display: flex; flex-direction: column; gap: 12px; }
  .titelzeile { width: 180px; height: 20px; }
  .karten { display: grid; grid-auto-flow: column; grid-auto-columns: 152px; gap: 16px; overflow: hidden; }
  .karte { height: 152px; border-radius: var(--r-l); }
</style>
