<script>
  // Alle Vorschaubilder eines Spiels. Bei angepinnten Spielen lässt sich eins als Titelbild wählen
  // oder ein eigenes Bild hochladen.
  import { onMount } from "svelte";
  import Icon from "./Icon.svelte";
  import { isPinned, coverOf, setCover, bildVorbereiten, toast } from "../lib/app.svelte.js";
  import { call } from "../lib/bridge.js";

  /** @type {{ game: { universeId: number }, gross: string | null, onzeigen: (url: string) => void }} */
  let { game, gross, onzeigen } = $props();

  let bilder = $state(null);
  let datei = $state();

  onMount(() => {
    call("game_media", { universeId: game.universeId })
      .then((b) => (bilder = b))
      .catch(() => (bilder = []));
  });

  const pinned = $derived(isPinned(game.universeId));
  const cover = $derived(coverOf(game.universeId));

  async function hochladen(e) {
    const f = e.currentTarget.files?.[0];
    e.currentTarget.value = "";
    if (!f) return;
    try {
      await setCover(game.universeId, await bildVorbereiten(f));
    } catch {
      toast("Bild nicht verwendbar", "fehler", "Nimm ein PNG, JPG oder WebP.");
    }
  }
</script>

{#if bilder === null || bilder.length || pinned}
  <section class="galerie">
    <header>
      <h2>Bilder</h2>
      {#if pinned}
        <div class="aktionen">
          {#if cover}
            <button class="knopf flach klein" onclick={() => setCover(game.universeId, null)}>Standard-Titelbild</button>
          {/if}
          <button class="knopf klein" onclick={() => datei.click()}><Icon name="hochladen" size={14} />Eigenes Titelbild</button>
          <input bind:this={datei} type="file" accept="image/png,image/jpeg,image/webp" hidden onchange={hochladen} />
        </div>
      {/if}
    </header>
    <div class="spur">
      {#if bilder === null}
        {#each Array(4) as _, i (i)}<div class="skelett platz"></div>{/each}
      {:else}
        {#if cover?.startsWith("data:")}
          <div class="bild eigen" class:gewaehlt={gross === cover}>
            <button class="flaeche" onclick={() => onzeigen(cover)} aria-label="Eigenes Titelbild ansehen"><img src={cover} alt="" /></button>
            <span class="marke">Eigenes Titelbild</span>
          </div>
        {/if}
        {#each bilder as url, i (url)}
          <div class="bild" class:gewaehlt={gross === url}>
            <button class="flaeche" onclick={() => onzeigen(url)} aria-label="Bild {i + 1} groß ansehen">
              <img src={url} alt="" loading="lazy" decoding="async" />
            </button>
            {#if pinned}
              {#if cover === url}
                <span class="marke">Titelbild</span>
              {:else}
                <button class="knopf klein setzen" onclick={() => setCover(game.universeId, url)}>Als Titelbild</button>
              {/if}
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  </section>
{/if}

<style>
  .galerie { display: flex; flex-direction: column; gap: 12px; }
  header { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .aktionen { display: flex; gap: 6px; }
  .spur {
    display: grid; grid-auto-flow: column; grid-auto-columns: 220px; gap: 12px;
    overflow-x: auto; scrollbar-width: thin; padding: 3px 0 6px;
  }
  .platz { aspect-ratio: 16 / 9; border-radius: var(--r); }
  .bild { position: relative; border-radius: var(--r); overflow: hidden; aspect-ratio: 16 / 9; background: var(--s2); }
  .flaeche { display: block; width: 100%; height: 100%; padding: 0; border: 0; background: none; cursor: default; }
  img { width: 100%; height: 100%; object-fit: cover; display: block; transition: transform 220ms var(--ein); }
  .bild::after { content: ""; position: absolute; inset: 0; border-radius: inherit; box-shadow: inset 0 0 0 1px var(--rand-bild); pointer-events: none; transition: box-shadow 140ms ease; }
  .bild.gewaehlt::after { box-shadow: inset 0 0 0 2px var(--t1); }
  @media (hover: hover) { .bild:hover img { transform: scale(1.03); } }
  .marke { position: absolute; left: 8px; bottom: 8px; background: var(--abdeckung); color: var(--t1); }
  .setzen { position: absolute; right: 8px; bottom: 8px; opacity: 0; transform: translateY(4px); transition: opacity 140ms ease, transform 160ms var(--ein); }
  .bild:hover .setzen, .setzen:focus-visible { opacity: 1; transform: none; }
</style>
