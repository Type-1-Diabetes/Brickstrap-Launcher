<script>
  // Spielzeit nach Zeitraum: Summe, Balken je Tag und die Spiele des Zeitraums.
  // Rechnet aus den Sitzungen der letzten 30 Tage; „Gesamt“ nutzt die fertige Summe aus Rust.
  import Bild from "./Bild.svelte";
  import { openGame } from "../lib/app.svelte.js";
  import { dauer, seit, tagBeginn, tagKurz, tagDatum, tagVoll } from "../lib/format.js";

  /** @type {{ zeit: { games: any[], month: any[] } }} */
  let { zeit } = $props();

  let raum = $state("7");
  const RAEUME = [["7", "7 Tage"], ["30", "30 Tage"], ["alle", "Gesamt"]];

  const tage = $derived.by(() => {
    if (raum === "alle") return [];
    const n = Number(raum);
    const heute = tagBeginn(Date.now() / 1000);
    const liste = [];
    for (let i = n - 1; i >= 0; i--) {
      // Mittags rechnen, damit die Zeitumstellung keinen Tag verschluckt.
      const t = tagBeginn(heute + 43200 - i * 86400);
      liste.push({ t, secs: 0 });
    }
    const index = new Map(liste.map((d) => [d.t, d]));
    for (const s of zeit.month ?? []) {
      const d = index.get(tagBeginn(s.start));
      if (d) d.secs += s.secs;
    }
    return liste;
  });

  const spiele = $derived.by(() => {
    if (raum === "alle") return zeit.games;
    const ab = tage[0]?.t ?? 0;
    const map = new Map();
    for (const s of zeit.month ?? []) {
      if (s.start < ab) continue;
      const key = s.universeId || `p${s.placeId}`;
      const g = map.get(key) ?? { universeId: s.universeId, placeId: s.placeId, name: s.name, totalSecs: 0, sessions: 0, lastPlayed: 0 };
      g.totalSecs += s.secs;
      g.sessions += 1;
      g.lastPlayed = Math.max(g.lastPlayed, s.end);
      map.set(key, g);
    }
    return [...map.values()].toSorted((a, b) => b.totalSecs - a.totalSecs);
  });

  const summe = $derived(spiele.reduce((n, g) => n + g.totalSecs, 0));
  const maxTag = $derived(Math.max(1, ...tage.map((d) => d.secs)));
  const maxSpiel = $derived(Math.max(1, ...spiele.map((g) => g.totalSecs)));
  const aktiveTage = $derived(tage.filter((d) => d.secs > 0).length);
  // Bei 30 Tagen nur jeden fünften Tag beschriften.
  const beschriftet = (i) => raum === "7" || (tage.length - 1 - i) % 5 === 0;
</script>

<div class="kopf">
  <h2>Spielzeit</h2>
  <div class="raeume" role="group" aria-label="Zeitraum">
    {#each RAEUME as [wert, name] (wert)}
      <button class="chip" aria-pressed={raum === wert} onclick={() => (raum = wert)}>{name}</button>
    {/each}
  </div>
</div>

<div class="tafel">
  <div class="summe">
    <strong class="zahl">{summe ? dauer(summe) : "0 Min."}</strong>
    <span class="leise zahl">
      {raum === "alle"
        ? `insgesamt in ${spiele.length} ${spiele.length === 1 ? "Spiel" : "Spielen"}`
        : `an ${aktiveTage} von ${tage.length} Tagen` + (aktiveTage ? ` · im Schnitt ${dauer(summe / tage.length)} am Tag` : "")}
    </span>
  </div>

  {#if tage.length}
    <ol class="balken" class:eng={raum === "30"} aria-label="Spielzeit je Tag">
      {#each tage as d, i (d.t)}
        <li title="{tagVoll(d.t)}: {d.secs ? dauer(d.secs) : 'nicht gespielt'}">
          <span class="saeule"><span class="fuellung" class:leer={!d.secs} style="transform: scaleY({d.secs ? Math.max(0.04, d.secs / maxTag) : 0.012})"></span></span>
          <span class="tag zahl">{beschriftet(i) ? (raum === "7" ? tagKurz(d.t) : tagDatum(d.t)) : ""}</span>
        </li>
      {/each}
    </ol>
  {/if}
</div>

{#if spiele.length}
  <ol class="liste">
    {#each spiele as g (g.universeId + "-" + g.placeId)}
      <li class="eintrag">
        <button class="bild" onclick={() => g.universeId && openGame(g)} aria-label="{g.name} öffnen" disabled={!g.universeId}><Bild universeId={g.universeId} name={g.name} /></button>
        <div class="zeitinfo">
          <div class="zeile1">
            <span class="nm">{g.name || "Unbekanntes Spiel"}</span>
            <span class="zahl">{dauer(g.totalSecs)}</span>
          </div>
          <span class="spur"><span class="lauf" style="transform: scaleX({g.totalSecs / maxSpiel})"></span></span>
          <span class="leise klein zahl">{g.sessions} {g.sessions === 1 ? "Sitzung" : "Sitzungen"} · zuletzt {seit(g.lastPlayed)}</span>
        </div>
      </li>
    {/each}
  </ol>
{:else}
  <p class="leise nichts">In diesem Zeitraum hast du nicht gespielt.</p>
{/if}

<style>
  .kopf { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; }
  .raeume { display: flex; gap: 6px; }
  .tafel { display: flex; flex-direction: column; gap: 18px; padding: 18px 18px 14px; border-radius: var(--r-l); background: var(--s1); }
  .summe { display: flex; align-items: baseline; gap: 12px; flex-wrap: wrap; }
  .summe strong { font-size: 26px; font-weight: 700; font-stretch: 108%; }
  .summe span { font-size: 13px; }

  .balken { list-style: none; margin: 0; padding: 0; display: flex; align-items: flex-end; gap: 10px; height: 132px; }
  .balken.eng { gap: 3px; }
  .balken li { flex: 1; min-width: 0; height: 100%; display: flex; flex-direction: column; gap: 8px; }
  .saeule { flex: 1; display: flex; align-items: flex-end; border-radius: 4px; }
  .fuellung { display: block; width: 100%; height: 100%; border-radius: 4px 4px 1px 1px; background: var(--t2); transform-origin: bottom; animation: wachsen 420ms var(--ein) both; transition: background-color 120ms ease; }
  .fuellung.leer { background: var(--s4); border-radius: 1px; }
  .balken li:hover .fuellung:not(.leer) { background: var(--t1); }
  @keyframes wachsen { from { transform: scaleY(0); } }
  .tag { height: 14px; font-size: 11.5px; line-height: 14px; color: var(--t3); text-align: center; white-space: nowrap; }
  .balken.eng .tag { overflow: visible; display: flex; justify-content: center; }

  .liste { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; border-radius: var(--r-l); background: var(--s1); overflow: hidden; }
  .eintrag { display: flex; align-items: center; gap: 14px; padding: 10px 14px; border-top: 1px solid var(--linie); transition: background-color 100ms ease; }
  .eintrag:first-child { border-top: 0; }
  .eintrag:hover { background: var(--s2); }
  .bild { width: 48px; height: 48px; padding: 0; border: 0; border-radius: 9px; overflow: hidden; flex: none; background: none; cursor: default; }
  .bild :global(.platz) { font-size: 14px; }
  .zeitinfo { flex: 1; display: flex; flex-direction: column; gap: 6px; min-width: 0; }
  .zeile1 { display: flex; justify-content: space-between; gap: 12px; font-weight: 600; }
  .nm { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .spur { display: block; height: 4px; border-radius: 2px; background: var(--s3); overflow: hidden; }
  .lauf { display: block; height: 100%; background: var(--t2); transform-origin: left; transition: transform 240ms var(--ein); }
  .klein { font-size: 12.5px; }
  .nichts { padding: 4px 2px; }
  @media (prefers-reduced-motion: reduce) {
    .fuellung { animation: none; }
    .lauf { transition: none; }
  }
</style>
