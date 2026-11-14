<script>
  // Mehr zum Spiel: Events, Game-Pässe und Abzeichen. Jeder Teil darf fehlen;
  // ohne Daten zeigt die Spielseite diesen Bereich gar nicht.
  import { onMount } from "svelte";
  import Icon from "./Icon.svelte";
  import { call } from "../lib/bridge.js";
  import { zahl, wann, anteil } from "../lib/format.js";

  /** @type {{ universeId: number }} */
  let { universeId } = $props();

  let daten = $state(null);
  let reiter = $state(null);
  let alle = $state(false);

  onMount(async () => {
    try {
      const d = await call("game_extras", { universeId });
      const jetzt = Date.now();
      // Nur laufende und kommende Events, die nächsten zuerst.
      d.events = (d.events ?? [])
        .filter((e) => !(Date.parse(e.end) < jetzt))
        .toSorted((a, b) => (Date.parse(a.start) || 0) - (Date.parse(b.start) || 0));
      daten = d;
      reiter = d.passes.length ? "passes" : d.badges.length ? "badges" : null;
    } catch {
      daten = { badges: [], passes: [], events: [] };
    }
  });

  const laeuft = (e) => Date.parse(e.start) <= Date.now();
  const liste = $derived(reiter === "passes" ? (daten?.passes ?? []) : (daten?.badges ?? []));
  const sichtbar = $derived(alle ? liste : liste.slice(0, 8));

  function wechseln(r) {
    reiter = r;
    alle = false;
  }
</script>

{#if daten?.events.length}
  <section class="events" aria-label="Events">
    <h2>Events</h2>
    <ul>
      {#each daten.events.slice(0, 3) as e (e.id + e.start)}
        <li>
          <span class="symbol"><Icon name="kalender" size={16} /></span>
          <div class="tx">
            <strong>{e.title}</strong>
            {#if e.subtitle}<span class="leise">{e.subtitle}</span>{/if}
          </div>
          {#if laeuft(e)}
            <span class="zeit zahl"><span class="punkt" aria-hidden="true"></span>{wann(e.end) ? `läuft bis ${wann(e.end)}` : "läuft"}</span>
          {:else}
            <span class="zeit zahl leise">ab {wann(e.start)}</span>
          {/if}
        </li>
      {/each}
    </ul>
  </section>
{/if}

{#if reiter}
  <section class="extras">
    <header>
      <div class="reiter" role="group" aria-label="Mehr zum Spiel">
        {#if daten.passes.length}
          <button class="chip" aria-pressed={reiter === "passes"} onclick={() => wechseln("passes")}>
            Game-Pässe <span class="zahl n">{daten.passes.length}</span>
          </button>
        {/if}
        {#if daten.badges.length}
          <button class="chip" aria-pressed={reiter === "badges"} onclick={() => wechseln("badges")}>
            Abzeichen <span class="zahl n">{daten.badges.length}{daten.badges.length >= 100 ? "+" : ""}</span>
          </button>
        {/if}
      </div>
    </header>

    <ul class="raster">
      {#each sichtbar as x (x.id)}
        <li title={x.description || x.name}>
          <span class="bild">{#if x.iconUrl}<img src={x.iconUrl} alt="" loading="lazy" />{/if}</span>
          <div class="tx">
            <strong>{x.name}</strong>
            {#if reiter === "passes"}
              <span class="leise zahl">{x.price != null ? `${zahl(x.price)} Robux` : "Nicht im Verkauf"}</span>
            {:else}
              <span class="leise zahl">
                {#if x.winRate != null}{anteil(x.winRate)} der Spieler{:else}{zahl(x.awarded)} vergeben{/if}
              </span>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
    {#if liste.length > 8}
      <button class="knopf flach klein" onclick={() => (alle = !alle)}>{alle ? "Weniger zeigen" : `Alle ${liste.length} zeigen`}</button>
    {/if}
  </section>
{/if}

<style>
  section { display: flex; flex-direction: column; gap: 12px; align-items: flex-start; }
  h2 { font-size: 17px; }
  ul { list-style: none; margin: 0; padding: 0; align-self: stretch; }

  .events ul { display: flex; flex-direction: column; border-radius: var(--r-l); background: var(--s1); overflow: hidden; max-width: var(--inhalt-max); }
  .events li { display: flex; align-items: center; gap: 12px; padding: 11px 16px 11px 14px; border-top: 1px solid var(--linie); }
  .events li:first-child { border-top: 0; }
  .symbol { display: grid; place-items: center; width: 32px; height: 32px; border-radius: var(--r); background: var(--s3); color: var(--t2); flex: none; }
  .zeit { margin-left: auto; display: flex; align-items: center; gap: 7px; font-size: 13px; white-space: nowrap; color: var(--t2); }
  .punkt { width: 7px; height: 7px; border-radius: 50%; background: var(--live); }

  .reiter { display: flex; gap: 8px; }
  .n { margin-left: 6px; opacity: 0.7; }
  .raster { display: grid; grid-template-columns: repeat(auto-fill, minmax(230px, 1fr)); gap: 4px 16px; }
  .raster li { display: flex; align-items: center; gap: 12px; padding: 7px 8px; margin: 0 -8px; border-radius: var(--r); min-width: 0; }
  .raster li:hover { background: var(--s1); }
  .bild { width: 44px; height: 44px; border-radius: var(--r); background: var(--s2); overflow: hidden; flex: none; box-shadow: inset 0 0 0 1px var(--rand-bild); }
  .bild img { display: block; width: 100%; height: 100%; object-fit: cover; }
  .tx { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
  .tx strong { font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .tx span { font-size: 12.5px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
