<script>
  // Vorschaubilder eines Spiels als Diashow im Titelbild. Bei angepinnten Spielen:
  // aktuelles Bild als Titelbild setzen oder ein eigenes hochladen.
  import { onMount } from "svelte";
  import Icon from "./Icon.svelte";
  import Bild from "./Bild.svelte";
  import { isPinned, coverOf, setCover, bildVorbereiten, toast } from "../lib/app.svelte.js";
  import { call } from "../lib/bridge.js";

  /** @type {{ game: { universeId: number, name: string } }} */
  let { game } = $props();

  let bilder = $state([]);
  let i = $state(0);
  let pause = $state(false);
  let datei = $state();

  onMount(() => {
    call("game_media", { universeId: game.universeId }).then((b) => (bilder = b)).catch(() => {});
    const t = setInterval(() => {
      if (!pause && liste.length > 1) i = (i + 1) % liste.length;
    }, 5000);
    return () => clearInterval(t);
  });

  const cover = $derived(coverOf(game.universeId));
  const pinned = $derived(isPinned(game.universeId));
  // Eigenes Titelbild zuerst, dann die Bilder von Roblox
  const liste = $derived([...(cover && !bilder.includes(cover) ? [cover] : []), ...bilder]);
  const aktuell = $derived(liste[i] ?? null);

  const gehe = (d) => (i = (i + d + liste.length) % liste.length);

  async function hochladen(e) {
    const f = e.currentTarget.files?.[0];
    e.currentTarget.value = "";
    if (!f) return;
    try {
      await setCover(game.universeId, await bildVorbereiten(f));
      i = 0;
    } catch {
      toast("Bild nicht verwendbar", "fehler", "Nimm ein PNG, JPG oder WebP.");
    }
  }
</script>

<div class="dia" role="group" aria-label="Bilder von {game.name}" onpointerenter={() => (pause = true)} onpointerleave={() => (pause = false)}>
  {#if liste.length === 0}
    <Bild universeId={game.universeId} name={game.name} kind="banner" eager />
  {:else}
    {#each liste as url, k (url)}
      <img src={url} alt="" class:sichtbar={k === i} loading={k === 0 ? "eager" : "lazy"} decoding="async" />
    {/each}
  {/if}

  {#if liste.length > 1}
    <button class="pfeil links" onclick={() => gehe(-1)} aria-label="Vorheriges Bild"><Icon name="zurueck" size={18} /></button>
    <button class="pfeil rechts" onclick={() => gehe(1)} aria-label="Nächstes Bild"><Icon name="rechts" size={18} /></button>
    <div class="punkte">
      {#each liste as _, k (k)}
        <button class:an={k === i} onclick={() => (i = k)} aria-label="Bild {k + 1}"></button>
      {/each}
    </div>
  {/if}

  {#if pinned}
    <div class="aktionen">
      {#if aktuell && aktuell === cover}
        <span class="marke">Titelbild</span>
        <button class="knopf klein" onclick={() => setCover(game.universeId, null)}>Zurücksetzen</button>
      {:else if aktuell}
        <button class="knopf klein" onclick={() => setCover(game.universeId, aktuell)}>Als Titelbild</button>
      {/if}
      <button class="knopf klein rund" onclick={() => datei.click()} aria-label="Eigenes Titelbild hochladen" title="Eigenes Titelbild hochladen">
        <Icon name="hochladen" size={15} />
      </button>
      <input bind:this={datei} type="file" accept="image/png,image/jpeg,image/webp" hidden onchange={hochladen} />
    </div>
  {/if}
</div>

<style>
  .dia { position: relative; aspect-ratio: 16 / 9; overflow: hidden; background: var(--s2); }
  img {
    position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover;
    opacity: 0; transform: scale(1.02);
    transition: opacity 600ms ease, transform 5600ms linear;
  }
  img.sichtbar { opacity: 1; transform: scale(1); }
  .pfeil {
    position: absolute; top: 50%; translate: 0 -50%;
    display: grid; place-items: center; width: 34px; height: 34px; padding: 0;
    border: 0; border-radius: 50%; background: var(--abdeckung); color: var(--t1); cursor: default;
    opacity: 0; transition: opacity 140ms ease, background-color 120ms ease;
  }
  .pfeil:hover { background: var(--s3); }
  .links { left: 10px; }
  .rechts { right: 10px; }
  .dia:hover .pfeil, .pfeil:focus-visible { opacity: 1; }
  .punkte { position: absolute; left: 50%; bottom: 10px; translate: -50% 0; display: flex; gap: 6px; padding: 5px 8px; border-radius: 10px; background: var(--abdeckung); }
  .punkte button { width: 6px; height: 6px; padding: 0; border: 0; border-radius: 3px; background: var(--t3); cursor: default; transition: width 200ms var(--ein), background-color 150ms ease; }
  .punkte button.an { width: 16px; background: var(--t1); }
  .aktionen { position: absolute; top: 10px; right: 10px; display: flex; gap: 6px; align-items: center; opacity: 0; transition: opacity 150ms ease; }
  .dia:hover .aktionen, .aktionen:focus-within { opacity: 1; }
  .aktionen .knopf { --bg: var(--abdeckung); --bg-h: var(--s3); }
  .marke { background: var(--abdeckung); color: var(--t1); }
  @media (prefers-reduced-motion: reduce) { img { transition: opacity 1ms; transform: none; } }
</style>
