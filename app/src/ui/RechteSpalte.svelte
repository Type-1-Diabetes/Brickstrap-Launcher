<script>
  // Rechte Spalte auf breiten Fenstern: Konto, was gerade läuft, Schnellstart, Spielzeit.
  import { onMount } from "svelte";
  import Konto from "./Konto.svelte";
  import Bild from "./Bild.svelte";
  import Icon from "./Icon.svelte";
  import { app, launch, starting, openGame, go } from "../lib/app.svelte.js";
  import { call } from "../lib/bridge.js";
  import { dauer, seit } from "../lib/format.js";

  let zeit = $state(null);
  onMount(() => {
    call("playtime_get").then((p) => (zeit = p)).catch(() => (zeit = { games: [] }));
  });

  const a = $derived(app.activity);
  const spielt = $derived(a.state === "playing" || a.state === "joining");
  const top = $derived((zeit?.games ?? []).slice(0, 4));
  const maxZeit = $derived(Math.max(1, ...top.map((g) => g.totalSecs)));
</script>

<aside class="rechts" aria-label="Übersicht">
  <section class="block">
    <Konto />
  </section>

  <section class="block">
    <h2>Gerade</h2>
    {#if spielt}
      <button class="jetzt" onclick={() => a.universeId && openGame({ universeId: a.universeId, placeId: a.placeId, name: a.gameName ?? "" })}>
        <span class="icon">{#if a.iconUrl}<img src={a.iconUrl} alt="" />{/if}</span>
        <span class="tx">
          <strong>{a.state === "joining" ? "Tritt bei …" : (a.gameName ?? "Roblox")}</strong>
          <small><span class="punkt"></span>läuft</small>
        </span>
      </button>
    {:else}
      <p class="leise klein">Roblox ist geschlossen.</p>
    {/if}
  </section>

  <section class="block">
    <div class="kopf"><h2>Schnellstart</h2>{#if app.pinned.length}<button class="knopf flach klein" onclick={() => go("bibliothek")}>Alle</button>{/if}</div>
    {#if app.pinned.length === 0}
      <p class="leise klein">Angepinnte Spiele lassen sich hier mit einem Klick starten.</p>
    {:else}
      <ul>
        {#each app.pinned.slice(0, 5) as p (p.universeId)}
          <li>
            <button class="zeile" onclick={() => openGame(p)}>
              <span class="mini"><Bild universeId={p.universeId} name={p.name} /></span>
              <span class="nm">{p.name}</span>
            </button>
            <button class="knopf klein rund" onclick={() => launch(p)} disabled={!!starting.key} aria-label="{p.name} spielen" title="Spielen">
              <Icon name="spielen" size={13} />
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  {#if top.length}
    <section class="block">
      <h2>Spielzeit</h2>
      <ul class="zeiten">
        {#each top as g (g.universeId + "-" + g.placeId)}
          <li>
            <div class="z1"><span class="nm">{g.name}</span><span class="zahl leise">{dauer(g.totalSecs)}</span></div>
            <span class="spur"><span class="fuellung" style="transform: scaleX({g.totalSecs / maxZeit})"></span></span>
          </li>
        {/each}
      </ul>
      <p class="leise klein">Zuletzt gespielt {seit(top[0].lastPlayed)}</p>
    </section>
  {/if}
</aside>

<style>
  .rechts {
    display: flex; flex-direction: column; gap: 10px;
    width: 296px; padding: 16px 14px; overflow-y: auto;
    background: var(--s0); border-left: 1px solid var(--linie);
  }
  .block { background: var(--s1); border-radius: var(--r-l); padding: 14px; display: flex; flex-direction: column; gap: 10px; }
  h2 { font-size: 13px; font-stretch: 100%; color: var(--t2); }
  .kopf { display: flex; align-items: center; justify-content: space-between; margin: -4px 0; }
  .klein { font-size: 12.5px; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; }
  li { display: flex; align-items: center; gap: 6px; }
  .zeile, .jetzt {
    flex: 1; min-width: 0; display: flex; align-items: center; gap: 10px;
    padding: 5px 6px; margin-left: -6px; border: 0; border-radius: var(--r);
    background: none; color: var(--t1); font: inherit; text-align: left; cursor: default;
    transition: background-color 120ms ease;
  }
  .zeile:hover, .jetzt:hover { background: var(--s2); }
  .mini, .icon { width: 30px; height: 30px; border-radius: 7px; overflow: hidden; flex: none; background: var(--s3); }
  .icon { width: 40px; height: 40px; }
  .icon img { width: 100%; height: 100%; object-fit: cover; }
  .mini :global(.platz) { font-size: 11px; }
  .nm { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 500; }
  .tx { display: flex; flex-direction: column; min-width: 0; }
  .tx strong { font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  small { color: var(--t3); font-size: 12px; display: flex; align-items: center; gap: 6px; }
  .punkt { width: 6px; height: 6px; border-radius: 50%; background: var(--live); }
  .zeiten { gap: 10px; }
  .zeiten li { flex-direction: column; align-items: stretch; gap: 5px; }
  .z1 { display: flex; justify-content: space-between; gap: 8px; font-size: 13px; }
  .spur { display: block; height: 4px; border-radius: 2px; background: var(--s3); overflow: hidden; }
  .fuellung { display: block; height: 100%; background: var(--t2); transform-origin: left; }
</style>
