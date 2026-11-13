<script>
  import { app } from "../lib/app.svelte.js";
  import Icon from "./Icon.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each app.toasts as t (t.id)}
    <div class="toast {t.art}">
      <Icon name={t.art === "fehler" ? "warnung" : "haken"} size={16} />
      <div>
        <strong>{t.text}</strong>
        {#if t.hinweis}<p>{t.hinweis}</p>{/if}
      </div>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 20px;
    bottom: calc(var(--dock) + 16px);
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 8px;
    z-index: 50;
    pointer-events: none;
  }
  .toast {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    max-width: 380px;
    padding: 11px 14px;
    border-radius: 10px;
    background: var(--s3);
    color: var(--t1);
    border: 1px solid var(--linie-stark);
    transform-origin: bottom right;
    transition: opacity 180ms ease, transform 220ms var(--ein);
    @starting-style { opacity: 0; transform: translateY(8px) scale(0.97); }
  }
  .toast :global(.icon) { margin-top: 2px; color: var(--t2); }
  .toast.fehler :global(.icon) { color: var(--fehler); }
  strong { font-weight: 600; }
  p { color: var(--t2); font-size: 13px; margin-top: 2px; }
</style>
