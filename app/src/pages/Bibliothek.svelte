<script>
  // Bibliothek: angepinnte Spiele (sortierbar) und Spielzeit aus der Logdatei.
  import { onMount } from "svelte";
  import Bild from "../ui/Bild.svelte";
  import Icon from "../ui/Icon.svelte";
  import Hinweis from "../ui/Hinweis.svelte";
  import { app, openGame, launch, starting, togglePin, prefetch, go } from "../lib/app.svelte.js";
  import { call } from "../lib/bridge.js";
  import { dauer, seit } from "../lib/format.js";

  let zeit = $state(null);

  onMount(() => {
    call("playtime_get").then((p) => (zeit = p)).catch(() => (zeit = { games: [], recent: [] }));
  });

  async function verschieben(i, d) {
    const j = i + d;
    if (j < 0 || j >= app.pinned.length) return;
    const order = app.pinned.map((p) => p.universeId);
    [order[i], order[j]] = [order[j], order[i]];
    app.pinned = await call("pin_order", { order });
  }

  const maxZeit = $derived(Math.max(1, ...(zeit?.games ?? []).map((g) => g.totalSecs)));
  const gesamt = $derived((zeit?.games ?? []).reduce((s, g) => s + g.totalSecs, 0));
</script>

<div class="bib">
  <section>
    <header><h1>Bibliothek</h1></header>
    <h2>Angepinnt</h2>
    {#if app.pinned.length === 0}
      <Hinweis icon="pin" titel="Noch nichts angepinnt" text="Auf jeder Spielseite gibt es „Anpinnen“. Angepinnte Spiele stehen auf der Startseite und in der Leiste.">
        <button class="knopf" onclick={() => go("suche")}><Icon name="suche" size={16} />Spiel suchen</button>
      </Hinweis>
    {:else}
      <ol class="liste">
        {#each app.pinned as p, i (p.universeId)}
          <li class="eintrag" onpointerenter={() => prefetch(p)}>
            <span class="rang zahl leise">{i + 1}</span>
            <button class="bild" onclick={() => openGame(p)} aria-label="{p.name} öffnen"><Bild universeId={p.universeId} name={p.name} /></button>
            <button class="name" onclick={() => openGame(p)}>{p.name}</button>
            <div class="werkzeuge">
              <button class="knopf flach klein rund" onclick={() => verschieben(i, -1)} disabled={i === 0} aria-label="Nach oben" title="Nach oben">
                <span class="hoch"><Icon name="rechts" size={15} /></span>
              </button>
              <button class="knopf flach klein rund" onclick={() => verschieben(i, 1)} disabled={i === app.pinned.length - 1} aria-label="Nach unten" title="Nach unten">
                <span class="runter"><Icon name="rechts" size={15} /></span>
              </button>
              <button class="knopf flach klein" onclick={() => togglePin(p)}>Lösen</button>
              <button class="knopf klein" onclick={() => launch(p)} disabled={starting.key === `${p.placeId}|`}><Icon name="spielen" size={13} />Spielen</button>
            </div>
          </li>
        {/each}
      </ol>
    {/if}
  </section>

  <section>
    <div class="zeitkopf">
      <h2>Spielzeit</h2>
      {#if gesamt}<span class="leise zahl">{dauer(gesamt)} insgesamt</span>{/if}
    </div>
    {#if zeit && zeit.games.length === 0}
      <Hinweis icon="uhr" titel="Noch keine Spielzeit erfasst" text="Brickstrap liest beim Spielen die Logdatei von Roblox mit und zählt hier, wie lange du was spielst. Es bleibt alles auf deinem PC." />
    {:else if zeit}
      <ol class="liste zeiten">
        {#each zeit.games as g (g.universeId + "-" + g.placeId)}
          <li class="eintrag">
            <button class="bild" onclick={() => g.universeId && openGame(g)} aria-label="{g.name} öffnen" disabled={!g.universeId}><Bild universeId={g.universeId} name={g.name} /></button>
            <div class="zeitinfo">
              <div class="zeile1">
                <span class="nm">{g.name || "Unbekanntes Spiel"}</span>
                <span class="zahl">{dauer(g.totalSecs)}</span>
              </div>
              <span class="spur"><span class="fuellung" style="transform: scaleX({g.totalSecs / maxZeit})"></span></span>
              <span class="leise klein zahl">{g.sessions} Sitzungen · zuletzt {seit(g.lastPlayed)}</span>
            </div>
          </li>
        {/each}
      </ol>
    {:else}
      <div class="skelett" style="height: 120px"></div>
    {/if}
  </section>
</div>

<style>
  .bib { display: flex; flex-direction: column; gap: 36px; max-width: var(--inhalt-max); }
  section { display: flex; flex-direction: column; gap: 14px; }
  header { margin-bottom: 10px; }
  .zeitkopf { display: flex; align-items: baseline; gap: 12px; }
  .liste { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; border-radius: var(--r-l); background: var(--s1); overflow: hidden; }
  .eintrag { display: flex; align-items: center; gap: 14px; padding: 10px 12px 10px 14px; border-top: 1px solid var(--linie); transition: background-color 100ms ease; }
  .eintrag:first-child { border-top: 0; }
  .eintrag:hover { background: var(--s2); }
  .rang { width: 18px; text-align: right; font-weight: 600; }
  .bild { width: 48px; height: 48px; padding: 0; border: 0; border-radius: 9px; overflow: hidden; flex: none; background: none; cursor: default; }
  .bild :global(.platz) { font-size: 14px; }
  .name { flex: 1; min-width: 0; padding: 0; border: 0; background: none; color: var(--t1); font: inherit; font-weight: 600; font-size: 15px; text-align: left; cursor: default; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .werkzeuge { display: flex; align-items: center; gap: 4px; }
  .hoch { display: inline-flex; transform: rotate(-90deg); }
  .runter { display: inline-flex; transform: rotate(90deg); }
  .zeitinfo { flex: 1; display: flex; flex-direction: column; gap: 6px; min-width: 0; }
  .zeile1 { display: flex; justify-content: space-between; gap: 12px; font-weight: 600; }
  .nm { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .spur { display: block; height: 4px; border-radius: 2px; background: var(--s3); overflow: hidden; }
  .fuellung { display: block; height: 100%; background: var(--t2); transform-origin: left; animation: fuellen 480ms var(--ein) both; }
  @keyframes fuellen { from { transform: scaleX(0); } }
  .klein { font-size: 12.5px; }
</style>
