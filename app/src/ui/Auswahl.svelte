<script>
  // Eigene Auswahl statt <select>: Knopf + Liste im Stil der App, mit Tastatur bedienbar.
  import Icon from "./Icon.svelte";

  /** @type {{ value: string, options: {value: string, label: string}[], onchange: (v: string) => void, label: string, disabled?: boolean, oben?: boolean, breite?: string }} */
  let { value, options, onchange, label, disabled = false, oben = false, breite = "150px" } = $props();

  let offen = $state(false);
  let aktiv = $state(0);
  let wurzel;
  const id = "aw-" + Math.random().toString(36).slice(2, 8);
  const gewaehlt = $derived(options.find((o) => o.value === value) ?? options[0]);

  function oeffnen() {
    if (disabled) return;
    aktiv = Math.max(0, options.findIndex((o) => o.value === value));
    offen = true;
  }
  function waehlen(v) {
    offen = false;
    if (v !== value) onchange(v);
  }
  function taste(e) {
    if (!offen) {
      if (["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) {
        e.preventDefault();
        oeffnen();
      }
      return;
    }
    if (e.key === "ArrowDown") aktiv = Math.min(options.length - 1, aktiv + 1);
    else if (e.key === "ArrowUp") aktiv = Math.max(0, aktiv - 1);
    else if (e.key === "Enter" || e.key === " ") waehlen(options[aktiv].value);
    else if (e.key === "Escape" || e.key === "Tab") offen = false;
    else return;
    if (e.key !== "Tab") e.preventDefault();
  }
  function aussen(e) {
    if (offen && wurzel && !wurzel.contains(e.target)) offen = false;
  }
</script>

<svelte:window onpointerdown={aussen} />

<div class="auswahl" class:oben bind:this={wurzel} style="--breite: {breite}">
  <button
    type="button"
    class="feld knopffeld"
    class:offen
    {disabled}
    aria-label={label}
    aria-haspopup="listbox"
    aria-expanded={offen}
    aria-controls={id}
    onclick={() => (offen ? (offen = false) : oeffnen())}
    onkeydown={taste}
  >
    <span class="wert">{gewaehlt?.label ?? ""}</span>
    <span class="pfeil"><Icon name="rechts" size={14} /></span>
  </button>
  {#if offen}
    <ul class="liste" role="listbox" {id} aria-label={label}>
      {#each options as o, i (o.value)}
        <li
          role="option"
          aria-selected={o.value === value}
          class:aktiv={i === aktiv}
          onpointerenter={() => (aktiv = i)}
          onclick={() => waehlen(o.value)}
          onkeydown={() => {}}
        >
          <span>{o.label}</span>
          {#if o.value === value}<Icon name="haken" size={14} />{/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .auswahl { position: relative; width: var(--breite); }
  .knopffeld {
    display: flex; align-items: center; justify-content: space-between; gap: 10px;
    width: 100%; text-align: left; cursor: default; line-height: 1;
  }
  .knopffeld.offen { border-color: var(--t2); background: var(--s2); }
  .knopffeld:disabled { opacity: 0.5; }
  .wert { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .pfeil { display: flex; color: var(--t2); transform: rotate(90deg); transition: transform 160ms var(--ein); }
  .offen .pfeil { transform: rotate(-90deg); }
  .liste {
    position: absolute; left: 0; right: 0; top: calc(100% + 6px); z-index: 40;
    margin: 0; padding: 4px; list-style: none;
    background: var(--s3); border: 1px solid var(--linie-stark); border-radius: 10px;
    max-height: 280px; overflow-y: auto;
    transform-origin: top center;
    animation: auf 150ms var(--ein);
  }
  .oben .liste { top: auto; bottom: calc(100% + 6px); transform-origin: bottom center; }
  @keyframes auf { from { opacity: 0; transform: scale(0.96); } }
  li {
    display: flex; align-items: center; justify-content: space-between; gap: 8px;
    height: 34px; padding: 0 10px; border-radius: 6px;
    color: var(--t2); font-size: 13.5px; cursor: default;
  }
  li.aktiv { background: var(--s4); color: var(--t1); }
  li[aria-selected="true"] { color: var(--t1); font-weight: 600; }
</style>
