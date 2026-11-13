<script>
  // FastFlag-Editor: nur Flags der Allowlist, Presets, Werte gelten beim nächsten Start.
  import Icon from "../ui/Icon.svelte";
  import Hinweis from "../ui/Hinweis.svelte";
  import { flags, ladeFlags, presetWaehlen, setzen, speichern } from "../lib/flags.svelte.js";
  import { toast, kopieren } from "../lib/app.svelte.js";
  import { seit } from "../lib/format.js";

  let importOffen = $state(false);
  let importText = $state("");
  let importFehler = $state("");

  $effect(() => {
    if (!flags.geladen) ladeFlags().catch(() => {});
  });

  const gruppen = $derived.by(() => {
    const m = new Map();
    for (const f of flags.allowlist) {
      if (!m.has(f.group)) m.set(f.group, []);
      m.get(f.group).push(f);
    }
    return [...m.entries()];
  });

  const erlaubt = $derived(new Set(flags.allowlist.map((f) => f.name)));
  const wirkungslos = $derived(Object.keys(flags.values).filter((n) => !erlaubt.has(n)));
  const anzahl = $derived(Object.keys(flags.values).filter((n) => erlaubt.has(n)).length);

  const datum = $derived(flags.allowlistDate ? new Date(flags.allowlistDate).toLocaleDateString("de-DE") : "");

  async function alsJson() {
    await kopieren(JSON.stringify($state.snapshot(flags.values), null, 2), "FastFlags als JSON kopiert");
  }

  function importieren() {
    importFehler = "";
    let obj;
    try {
      obj = JSON.parse(importText);
    } catch {
      importFehler = "Das ist kein gültiges JSON. Erwartet wird z. B. {\"FIntDebugForceMSAASamples\": \"4\"}.";
      return;
    }
    if (!obj || typeof obj !== "object" || Array.isArray(obj)) {
      importFehler = "Erwartet wird ein Objekt mit Flag-Namen und Werten.";
      return;
    }
    const werte = {};
    for (const [k, v] of Object.entries(obj)) werte[k] = String(v);
    flags.values = werte;
    flags.preset = "eigene";
    speichern(true);
    const ignoriert = Object.keys(werte).filter((n) => !erlaubt.has(n)).length;
    toast(
      `${Object.keys(werte).length - ignoriert} Flags übernommen`,
      "info",
      ignoriert ? `${ignoriert} stehen nicht auf der Allowlist und wirken nicht.` : "",
    );
    importOffen = false;
    importText = "";
  }

  function entfernen(name) {
    const next = { ...flags.values };
    delete next[name];
    flags.values = next;
    speichern();
  }

  const PRESET_AUS = { id: "aus", name: "Aus", description: "Roblox startet ohne eigene FastFlags." };
</script>

<div class="flags">
  <header class="kopf">
    <div>
      <h1>FastFlags</h1>
      <p class="lead">
        Mit FastFlags stellst du Grafik und Leistung feiner ein. Sie gelten ab dem nächsten Start von Roblox.
        Roblox beachtet nur Flags auf seiner Allowlist{datum ? ` (Stand ${datum})` : ""} und ignoriert alle anderen ohne Folgen.
      </p>
    </div>
    <div class="status leise" aria-live="polite">
      {#if !flags.gespeichert}Speichert …{:else if flags.lastApplied}Zuletzt geschrieben {seit(flags.lastApplied)}{:else}Gespeichert{/if}
    </div>
  </header>

  {#if !flags.geladen}
    <div class="skelett" style="height: 120px"></div>
  {:else}
    <section class="presets" aria-label="Presets">
      {#each [PRESET_AUS, ...flags.presets] as p (p.id)}
        <button class="preset" aria-pressed={flags.preset === p.id} onclick={() => presetWaehlen(p.id)}>
          <span class="pname">
            {p.name}
            {#if flags.preset === p.id}<Icon name="haken" size={15} />{/if}
          </span>
          <span class="pbeschr">{p.description}</span>
        </button>
      {/each}
      {#if flags.preset === "eigene"}
        <div class="preset eigene" aria-current="true">
          <span class="pname">Eigene <Icon name="haken" size={15} /></span>
          <span class="pbeschr zahl">{anzahl} Flags von dir angepasst</span>
        </div>
      {/if}
    </section>

    {#each gruppen as [gruppe, liste] (gruppe)}
      <section class="gruppe">
        <h2>{gruppe}</h2>
        <div class="karte">
          {#each liste as f (f.name)}
            {@const wert = flags.values[f.name]}
            <div class="zeile">
              <div class="text">
                <div class="label">
                  {f.label}
                  {#if f.experimental}<span class="marke warnung">Experimentell</span>{/if}
                </div>
                <p class="hint">{f.hint}</p>
                <code class="name waehlbar">{f.name}</code>
              </div>
              <div class="steuer">
                {#if f.kind === "bool"}
                  {#if wert === undefined}
                    <span class="standard" title="Kein eigener Wert: Roblox entscheidet">Standard</span>
                  {:else}
                    <button class="knopf flach klein rund" onclick={() => setzen(f.name, null)} aria-label="{f.label} auf Standard zurücksetzen" title="Auf Standard zurücksetzen">
                      <Icon name="x" size={14} />
                    </button>
                  {/if}
                  <span class="schalter">
                    <input type="checkbox" aria-label={f.label} checked={wert === "True"} onchange={(e) => setzen(f.name, e.currentTarget.checked ? "True" : "False")} />
                    <span></span>
                  </span>
                {:else if f.options}
                  <select class="feld" aria-label={f.label} value={wert ?? ""} onchange={(e) => setzen(f.name, e.currentTarget.value || null)}>
                    <option value="">Standard</option>
                    {#each f.options as o (o.value)}<option value={o.value}>{o.label}</option>{/each}
                  </select>
                {:else}
                  <input
                    class="feld zahlfeld"
                    type="number"
                    inputmode="numeric"
                    aria-label={f.label}
                    placeholder="Standard"
                    min={f.min ?? undefined}
                    max={f.max ?? undefined}
                    value={wert ?? ""}
                    onchange={(e) => {
                      const el = e.currentTarget;
                      setzen(f.name, el.value === "" ? null : Math.round(Number(el.value)));
                      el.value = flags.values[f.name] ?? "";
                    }}
                  />
                  {#if wert !== undefined}
                    <button class="knopf flach klein rund" onclick={() => setzen(f.name, null)} aria-label="{f.label} zurücksetzen" title="Auf Standard zurücksetzen">
                      <Icon name="x" size={14} />
                    </button>
                  {:else}
                    <span class="platzhalter" aria-hidden="true"></span>
                  {/if}
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/each}

    {#if wirkungslos.length}
      <section class="gruppe">
        <h2>Wirkungslos</h2>
        <p class="hint">Diese Flags stehen nicht auf der Allowlist. Brickstrap schreibt sie nicht, Roblox würde sie ohnehin ignorieren.</p>
        <div class="karte">
          {#each wirkungslos as n (n)}
            <div class="zeile">
              <div class="text"><code class="name waehlbar">{n}</code><p class="hint">Wert: {flags.values[n]}</p></div>
              <div class="steuer"><button class="knopf flach klein" onclick={() => entfernen(n)}>Entfernen</button></div>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <section class="gruppe">
      <h2>Teilen</h2>
      <div class="teilen">
        <button class="knopf" onclick={alsJson}><Icon name="kopieren" size={15} />Als JSON kopieren</button>
        <button class="knopf" aria-expanded={importOffen} onclick={() => (importOffen = !importOffen)}>JSON einfügen</button>
      </div>
      {#if importOffen}
        <div class="import">
          <textarea class="feld" rows="6" bind:value={importText} placeholder={'{\n  "FIntDebugForceMSAASamples": "4"\n}'} spellcheck="false"></textarea>
          {#if importFehler}<p class="fehlertext">{importFehler}</p>{/if}
          <div class="teilen">
            <button class="knopf primaer" onclick={importieren} disabled={!importText.trim()}>Übernehmen</button>
            <button class="knopf flach" onclick={() => (importOffen = false)}>Abbrechen</button>
          </div>
        </div>
      {/if}
    </section>

    {#if flags.allowlist.length === 0}
      <Hinweis art="fehler" titel="Allowlist fehlt" text="Brickstrap konnte die Liste erlaubter Flags nicht laden." />
    {/if}
  {/if}
</div>

<style>
  .flags { display: flex; flex-direction: column; gap: 28px; max-width: var(--inhalt-max); }
  .kopf { display: flex; justify-content: space-between; align-items: flex-start; gap: 24px; }
  .kopf > div { display: flex; flex-direction: column; gap: 8px; }
  .lead { color: var(--t3); max-width: 68ch; }
  .status { font-size: 12.5px; white-space: nowrap; padding-top: 8px; }

  .presets { display: grid; grid-template-columns: repeat(5, 1fr); gap: 10px; }
  @media (max-width: 1050px) { .presets { grid-template-columns: repeat(3, 1fr); } }
  .preset {
    display: flex; flex-direction: column; gap: 4px; align-items: flex-start;
    padding: 14px; border: 0; border-radius: var(--r-l);
    background: var(--s1); color: var(--t1); font: inherit; text-align: left; cursor: default;
    box-shadow: inset 0 0 0 1px var(--linie);
    transition: background-color 120ms ease, box-shadow 120ms ease;
  }
  .preset:hover { background: var(--s2); }
  .preset[aria-pressed="true"], .preset.eigene { box-shadow: inset 0 0 0 2px var(--t1); background: var(--s2); }
  .pname { display: flex; align-items: center; justify-content: space-between; gap: 8px; width: 100%; font-weight: 700; font-size: 15px; }
  .pbeschr { color: var(--t3); font-size: 12.5px; line-height: 1.4; }

  .gruppe { display: flex; flex-direction: column; gap: 10px; }
  .karte { border-radius: var(--r-l); background: var(--s1); overflow: hidden; }
  .zeile { display: flex; align-items: center; justify-content: space-between; gap: 24px; padding: 14px 16px 14px 18px; border-top: 1px solid var(--linie); }
  .zeile:first-child { border-top: 0; }
  .text { display: flex; flex-direction: column; gap: 3px; min-width: 0; }
  .label { display: flex; align-items: center; gap: 8px; font-weight: 600; }
  .hint { color: var(--t3); font-size: 13px; max-width: 64ch; }
  .name { font-family: var(--mono); font-size: 11.5px; color: var(--t3); overflow-wrap: anywhere; }
  .steuer { display: flex; align-items: center; gap: 6px; flex: none; }
  .steuer select { min-width: 150px; }
  .zahlfeld { width: 120px; text-align: right; }
  .platzhalter { width: 28px; }
  .standard { font-size: 12.5px; color: var(--t3); padding: 0 6px; }
  .teilen { display: flex; gap: 8px; }
  .import { display: flex; flex-direction: column; gap: 10px; }
  textarea.feld { height: auto; padding: 10px 12px; font-family: var(--mono); font-size: 13px; resize: vertical; }
  .fehlertext { color: var(--fehler); font-size: 13px; }
</style>
