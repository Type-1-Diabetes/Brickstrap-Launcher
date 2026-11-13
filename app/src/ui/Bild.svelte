<script>
  // Spielbild mit ruhigem Platzhalter (Anfangsbuchstaben), bis die URL da ist.
  import { app, wantImage, coverOf } from "../lib/app.svelte.js";

  /** @type {{ universeId: number, name: string, kind?: "icon"|"banner", eager?: boolean }} */
  let { universeId, name, kind = "icon", eager = false } = $props();

  const store = $derived(kind === "banner" ? app.banners : app.icons);
  // Banner angepinnter Spiele: eigenes Titelbild, falls gesetzt
  const url = $derived((kind === "banner" && coverOf(universeId)) || store[universeId]);
  let geladen = $state(false);

  $effect(() => {
    wantImage(universeId, kind);
  });

  const initialen = $derived(
    (name ?? "?")
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((w) => w[0])
      .join("")
      .toUpperCase(),
  );
</script>

<div class="bild {kind}" class:geladen>
  <span class="platz" aria-hidden="true">{initialen}</span>
  {#if url}
    <img
      src={url}
      alt=""
      loading={eager ? "eager" : "lazy"}
      decoding="async"
      draggable="false"
      onload={() => (geladen = true)}
    />
  {/if}
</div>

<style>
  .bild {
    position: relative;
    overflow: hidden;
    background: var(--s2);
    border-radius: inherit;
    aspect-ratio: 1;
    width: 100%;
  }
  .bild.banner { aspect-ratio: 16 / 9; }
  .platz {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    color: var(--t4);
    font-weight: 800;
    font-stretch: 118%;
    font-size: 22px;
    letter-spacing: -0.02em;
  }
  img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0;
    transition: opacity 220ms ease;
  }
  .geladen img { opacity: 1; }
</style>
