<script>
  // Leerer Zustand oder Fehler mit Erklärung und optionaler Aktion.
  import Icon from "./Icon.svelte";

  /** @type {{ icon?: string, titel: string, text?: string, art?: "leer"|"fehler", kompakt?: boolean, children?: import('svelte').Snippet }} */
  let { icon = "info", titel, text = "", art = "leer", kompakt = false, children } = $props();
</script>

<div class="hinweis {art}" class:kompakt role={art === "fehler" ? "alert" : undefined}>
  <span class="symbol"><Icon name={art === "fehler" ? "warnung" : icon} size={20} /></span>
  <div class="text">
    <h3>{titel}</h3>
    {#if text}<p>{text}</p>{/if}
    {#if children}<div class="aktion">{@render children()}</div>{/if}
  </div>
</div>

<style>
  .hinweis {
    display: flex;
    gap: 14px;
    align-items: flex-start;
    padding: 20px;
    border-radius: var(--r-l);
    background: var(--s1);
    max-width: 560px;
  }
  .hinweis.kompakt { padding: 14px 16px; }
  .symbol {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: var(--s3);
    color: var(--t2);
    flex: none;
  }
  .fehler .symbol { background: var(--fehler-flaeche); color: var(--fehler); }
  .text { display: flex; flex-direction: column; gap: 4px; padding-top: 2px; }
  p { color: var(--t3); max-width: 60ch; }
  .aktion { margin-top: 10px; display: flex; gap: 8px; flex-wrap: wrap; }
</style>
