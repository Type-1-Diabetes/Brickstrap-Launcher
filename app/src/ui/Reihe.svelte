<script>
  // Waagerechte Reihe mit Spielen; Pfeile blättern um eine Bildschirmbreite.
  import Icon from "./Icon.svelte";
  import Spielkarte from "./Spielkarte.svelte";

  /** @type {{ titel: string, games: any[], aktion?: import('svelte').Snippet }} */
  let { titel, games, aktion } = $props();

  let spur;
  let links = $state(false);
  let rechts = $state(true);

  function messen() {
    if (!spur) return;
    links = spur.scrollLeft > 4;
    rechts = spur.scrollLeft + spur.clientWidth < spur.scrollWidth - 4;
  }
  function blaettern(richtung) {
    spur.scrollBy({ left: richtung * (spur.clientWidth - 80), behavior: "smooth" });
  }
  $effect(() => {
    if (games.length >= 0) queueMicrotask(messen);
  });
</script>

<section class="reihe">
  <header>
    <h2>{titel}</h2>
    <div class="steuer">
      {@render aktion?.()}
      {#if links || rechts}
        <button class="knopf flach klein rund" onclick={() => blaettern(-1)} disabled={!links} aria-label="Zurückblättern">
          <Icon name="zurueck" size={16} />
        </button>
        <button class="knopf flach klein rund" onclick={() => blaettern(1)} disabled={!rechts} aria-label="Weiterblättern">
          <Icon name="rechts" size={16} />
        </button>
      {/if}
    </div>
  </header>
  <div class="spur" bind:this={spur} onscroll={messen}>
    {#each games as game (game.universeId)}
      <div class="platz"><Spielkarte {game} /></div>
    {/each}
  </div>
</section>

<style>
  .reihe { display: flex; flex-direction: column; gap: 12px; }
  header { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .steuer { display: flex; align-items: center; gap: 4px; }
  .spur {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 152px;
    gap: 16px;
    overflow-x: auto;
    scroll-snap-type: x proximity;
    scrollbar-width: none;
    padding-bottom: 2px;
    /* Kacheln dürfen beim Anheben nicht oben abgeschnitten werden */
    padding-top: 3px;
    margin-top: -3px;
    /* bis zum Fensterrand laufen, damit angeschnittene Karten am Rand enden, nicht im Seitenabstand */
    margin-right: calc(-1 * var(--seite-rand));
    padding-right: var(--seite-rand);
    scroll-padding-inline: 0 var(--seite-rand);
  }
  .spur::-webkit-scrollbar { display: none; }
  .platz { scroll-snap-align: start; content-visibility: auto; contain-intrinsic-size: 152px 230px; }
</style>
