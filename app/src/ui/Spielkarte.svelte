<script>
  // Spiel in einer Reihe oder im Raster: quadratisches Bild, Name, Spieler online.
  import Bild from "./Bild.svelte";
  import Icon from "./Icon.svelte";
  import { openGame, prefetch, isPinned } from "../lib/app.svelte.js";
  import { zahl, bewertung } from "../lib/format.js";

  /** @type {{ game: any }} */
  let { game } = $props();

  let hoverTimer;
  function enter() {
    // Signatur: kurz verweilen lädt die Serverliste vor.
    hoverTimer = setTimeout(() => prefetch(game), 180);
  }
  function leave() {
    clearTimeout(hoverTimer);
  }
  const quote = $derived(bewertung(game.upVotes, game.downVotes));
</script>

<button
  class="karte"
  onclick={() => openGame(game)}
  onpointerenter={enter}
  onpointerleave={leave}
  onfocus={() => prefetch(game)}
  title={game.name}
>
  <div class="bildrahmen">
    <Bild universeId={game.universeId} name={game.name} />
    {#if isPinned(game.universeId)}
      <span class="angepinnt" title="Angepinnt"><Icon name="pinFill" size={13} /></span>
    {/if}
  </div>
  <span class="name">{game.name}</span>
  <span class="meta zahl">
    {#if game.playing}
      <span class="punkt" aria-hidden="true"></span>{zahl(game.playing)} online
    {/if}
    {#if quote !== null}
      <span class="quote"><Icon name="daumen" size={12} />{quote} %</span>
    {/if}
  </span>
</button>

<style>
  .karte {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    padding: 0;
    border: 0;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: default;
  }
  .bildrahmen {
    position: relative;
    border-radius: var(--r-l);
    overflow: hidden;
    transition: transform 200ms var(--ein);
  }
  .bildrahmen::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    box-shadow: inset 0 0 0 1px #ffffff12;
    transition: box-shadow 160ms ease;
  }
  @media (hover: hover) and (pointer: fine) {
    .karte:hover .bildrahmen { transform: translateY(-2px); }
    .karte:hover .bildrahmen::after { box-shadow: inset 0 0 0 2px var(--t2); }
  }
  .karte:active .bildrahmen { transform: scale(0.98); }
  .karte:focus-visible { outline: none; }
  .karte:focus-visible .bildrahmen::after { box-shadow: inset 0 0 0 2px var(--t1); }
  .angepinnt {
    position: absolute;
    top: 8px;
    right: 8px;
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 6px;
    background: #0b0b0dcc;
    color: var(--t1);
  }
  .name {
    font-weight: 600;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: -4px;
    color: var(--t3);
    font-size: 12.5px;
  }
  .punkt {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--live);
    margin-right: -4px;
  }
  .quote { display: inline-flex; align-items: center; gap: 4px; }
</style>
