<script>
  // Großer Update-Dialog über der ganzen App. Erscheint beim Start, zwischendurch
  // (Suche alle 3 Stunden) und nach „Nach Updates suchen“.
  import { upd, updateInstallieren, updateSpaeter } from "../lib/update.svelte.js";
  import zeichen from "./zeichen.svg";

  let knopf = $state();
  $effect(() => {
    if (upd.offen) queueMicrotask(() => knopf?.focus());
  });

  const mb = (b) => (b / 1048576).toLocaleString("de-DE", { maximumFractionDigits: 1 }) + " MB";

  // Release-Text ohne Markdown-Zeichen, als ruhiger Fließtext.
  const notizen = $derived(
    (upd.info?.notes ?? "")
      .replace(/\r/g, "")
      .replace(/^#{1,6}\s*/gm, "")
      .replace(/\*\*(.+?)\*\*/g, "$1")
      .replace(/`([^`]+)`/g, "$1")
      .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
      .replace(/^\s*[-*]\s+/gm, "• ")
      .replace(/\n{3,}/g, "\n\n")
      .trim(),
  );

  function taste(e) {
    if (e.key === "Escape" && upd.offen) updateSpaeter();
  }
</script>

<svelte:window onkeydown={taste} />

{#if upd.offen && upd.info}
  <div class="abdeckung">
    <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="upd-titel" aria-describedby="upd-sub">
      <img class="zeichen" src={zeichen} alt="" />
      <h2 id="upd-titel">Brickstrap {upd.info.version} ist da</h2>
      <p id="upd-sub" class="sub">
        Du nutzt {upd.info.current}{#if upd.info.size}{" · "}Download {mb(upd.info.size)}{/if}
      </p>

      {#if notizen}
        <div class="notizen" role="region" aria-label="Neu in dieser Version">{notizen}</div>
      {/if}

      {#if upd.laedt || upd.fertig}
        <div class="lauf" role="status" aria-live="polite">
          <span class="spur"><span class="fuellung" style="transform: scaleX({upd.fertig ? 1 : upd.fortschritt})"></span></span>
          <span class="leise zahl">
            {#if upd.fertig}Fertig. Brickstrap startet neu …
            {:else if upd.gesamt}Lädt {mb(upd.geladen)} von {mb(upd.gesamt)}
            {:else}Verbindet …{/if}
          </span>
        </div>
      {/if}

      {#if upd.fehler}
        <p class="fehler" role="alert"><strong>{upd.fehler.titel}</strong>{#if upd.fehler.hinweis}{" "}{upd.fehler.hinweis}{/if}</p>
      {/if}

      <div class="knoepfe">
        <button bind:this={knopf} class="knopf primaer gross" onclick={updateInstallieren} disabled={upd.laedt || upd.fertig}>
          {upd.fehler ? "Erneut versuchen" : upd.laedt ? "Wird aktualisiert …" : "Jetzt aktualisieren"}
        </button>
        <button class="knopf flach gross" onclick={updateSpaeter} disabled={upd.laedt || upd.fertig}>Später</button>
      </div>
      <p class="fuss leise">Brickstrap prüft die Datei vor dem Austausch und startet danach von selbst neu.</p>
    </div>
  </div>
{/if}

<style>
  .abdeckung {
    position: fixed; inset: 0; z-index: 100;
    display: grid; place-items: center; padding: 24px;
    background: var(--abdeckung);
    animation: einblenden 180ms var(--ein) both;
  }
  .dialog {
    width: min(520px, 100%); max-height: calc(100vh - 48px); overflow-y: auto;
    display: flex; flex-direction: column; gap: 14px;
    padding: 28px; border-radius: 16px;
    background: var(--s1); border: 1px solid var(--linie-stark);
    box-shadow: 0 24px 64px -12px #000c;
    animation: heben 220ms var(--ein) both;
  }
  .zeichen { width: 48px; height: 48px; border-radius: 11px; }
  h2 { font-size: 24px; font-weight: 700; letter-spacing: -0.01em; line-height: 1.15; text-wrap: balance; }
  .sub { margin-top: -8px; color: var(--t3); font-size: 13.5px; font-variant-numeric: tabular-nums; }
  .notizen {
    max-height: 200px; overflow-y: auto; overscroll-behavior: contain;
    padding: 12px 14px; border-radius: var(--r);
    background: var(--s2); color: var(--t2);
    font-size: 13px; line-height: 1.55; white-space: pre-wrap;
  }
  .lauf { display: flex; flex-direction: column; gap: 6px; }
  .spur { display: block; height: 6px; border-radius: 3px; background: var(--s3); overflow: hidden; }
  .fuellung { display: block; height: 100%; background: var(--primaer); transform-origin: left; transition: transform 120ms linear; }
  .zahl { font-size: 12.5px; font-variant-numeric: tabular-nums; }
  .fehler { color: var(--fehler); font-size: 13px; line-height: 1.45; }
  .knoepfe { display: flex; gap: 8px; margin-top: 4px; }
  .knoepfe .primaer { flex: 1; }
  .fuss { font-size: 12px; }
  @keyframes einblenden { from { opacity: 0; } }
  @keyframes heben { from { opacity: 0; transform: translateY(8px) scale(0.98); } }
  @media (prefers-reduced-motion: reduce) {
    .abdeckung, .dialog { animation: none; }
  }
</style>
