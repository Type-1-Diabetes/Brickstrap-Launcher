<script>
  // Einstellungen aus Roblox' eigener Datei (GlobalBasicSettings_13.xml).
  // Schreiben nur, wenn Roblox geschlossen ist, sonst überschreibt Roblox die Datei beim Beenden.
  import { onMount } from "svelte";
  import Icon from "../ui/Icon.svelte";
  import Hinweis from "../ui/Hinweis.svelte";
  import { app, toast } from "../lib/app.svelte.js";
  import { call } from "../lib/bridge.js";
  import { fehlerText } from "../lib/errors.js";

  let view = $state(null);
  let fehler = $state(null);
  let entwurf = $state({});
  let speichert = $state(false);
  let weitereOffen = $state(false);

  async function laden() {
    fehler = null;
    try {
      view = await call("basic_get");
      entwurf = {};
    } catch (e) {
      fehler = e;
    }
  }
  onMount(laden);

  // Neu laden, wenn Roblox geschlossen wird.
  let warLaufend = false;
  $effect(() => {
    const laeuft = app.activity.state !== "idle";
    if (warLaufend && !laeuft) laden();
    warLaufend = laeuft;
  });

  const bekannt = $derived((view?.entries ?? []).filter((e) => e.label && e.editable));
  const weitere = $derived((view?.entries ?? []).filter((e) => !(e.label && e.editable)));
  const aenderungen = $derived(Object.keys(entwurf).length);
  const laeuft = $derived(view?.running || app.activity.state !== "idle");

  const wert = (e) => entwurf[e.name] ?? e.value;

  function setze(e, v) {
    const s = String(v);
    if (s === e.value) {
      const { [e.name]: _, ...rest } = entwurf;
      entwurf = rest;
    } else entwurf = { ...entwurf, [e.name]: s };
  }

  async function speichern() {
    speichert = true;
    try {
      const changes = Object.entries(entwurf).map(([name, value]) => ({
        name,
        value,
        kind: view.entries.find((e) => e.name === name)?.kind ?? "string",
      }));
      await call("basic_set", { changes });
      toast("Roblox-Einstellungen gespeichert");
      await laden();
    } catch (e) {
      const f = fehlerText(e);
      toast(f.titel, "fehler", f.hinweis);
    } finally {
      speichert = false;
    }
  }

  const f = $derived(fehler ? fehlerText(fehler) : null);
  const fpsText = (v) => (Number(v) === 0 ? "Ohne Limit" : `${v} FPS`);
</script>

<div class="roblox">
  <header>
    <h1>Roblox-Einstellungen</h1>
    <p class="lead">Die Werte aus dem Spielmenü von Roblox, direkt in seiner Einstellungsdatei. Manche lassen sich hier feiner setzen als im Spiel.</p>
  </header>

  {#if laeuft}
    <Hinweis art="fehler" kompakt titel="Roblox läuft" text="Schließ Roblox, bevor du hier etwas änderst. Sonst überschreibt Roblox deine Änderungen beim Beenden." />
  {/if}

  {#if fehler}
    <Hinweis art="fehler" titel={f.titel} text={f.hinweis}>
      <button class="knopf" onclick={laden}><Icon name="neu" size={16} />Erneut lesen</button>
    </Hinweis>
  {:else if !view}
    <div class="skelett" style="height: 240px"></div>
  {:else if !view.exists}
    <Hinweis icon="regler" titel="Noch keine Einstellungsdatei" text="Roblox legt sie beim ersten Start an. Starte Roblox einmal, schließ es wieder und öffne dann diese Seite." />
  {:else}
    <section class="karte" class:gesperrt={laeuft}>
      {#each bekannt as e (e.name)}
        <div class="zeile">
          <div class="text">
            <span class="label">{e.label}</span>
            {#if e.hint}<p class="hint">{e.hint}</p>{/if}
          </div>
          <div class="steuer">
            {#if e.kind === "bool"}
              <span class="schalter">
                <input type="checkbox" aria-label={e.label} disabled={laeuft} checked={wert(e) === "true"} onchange={(ev) => setze(e, ev.currentTarget.checked ? "true" : "false")} />
                <span></span>
              </span>
            {:else if e.name === "FramerateCap"}
              <select class="feld" aria-label={e.label} disabled={laeuft} value={String(wert(e))} onchange={(ev) => setze(e, ev.currentTarget.value)}>
                {#each ["30", "60", "75", "120", "144", "165", "240", "0"] as o (o)}<option value={o}>{fpsText(o)}</option>{/each}
                {#if !["30", "60", "75", "120", "144", "165", "240", "0"].includes(String(wert(e)))}<option value={String(wert(e))}>{fpsText(wert(e))}</option>{/if}
              </select>
            {:else if e.min !== null && e.max !== null && e.max - e.min <= 10}
              <input class="regler" type="range" aria-label={e.label} disabled={laeuft} min={e.min} max={e.max} step={e.step ?? 1} value={wert(e)} oninput={(ev) => setze(e, ev.currentTarget.value)} />
              <span class="wert zahl">{e.kind === "token" && Number(wert(e)) === 0 && e.name === "SavedQualityLevel" ? "Auto" : wert(e)}</span>
            {:else}
              <input class="feld zahlfeld" type="number" aria-label={e.label} disabled={laeuft} min={e.min ?? undefined} max={e.max ?? undefined} step={e.step ?? (e.kind === "float" ? 0.01 : 1)} value={wert(e)} onchange={(ev) => setze(e, ev.currentTarget.value)} />
            {/if}
          </div>
        </div>
      {/each}
    </section>

    {#if weitere.length}
      <section class="weitere">
        <button class="knopf flach" aria-expanded={weitereOffen} onclick={() => (weitereOffen = !weitereOffen)}>
          <span class="pfeil" class:offen={weitereOffen}><Icon name="rechts" size={15} /></span>
          Weitere Werte aus der Datei ({weitere.length})
        </button>
        {#if weitereOffen}
          <div class="karte">
            {#each weitere as e (e.name)}
              <div class="zeile">
                <div class="text"><code class="waehlbar">{e.name}</code><span class="hint">{e.kind}</span></div>
                <div class="steuer">
                  {#if e.editable}
                    <input class="feld zahlfeld" aria-label={e.name} disabled={laeuft} value={wert(e)} onchange={(ev) => setze(e, ev.currentTarget.value)} />
                  {:else}
                    <span class="leise zahl waehlbar">{e.value}</span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    {/if}

    <p class="pfad leise">Datei: <code class="waehlbar">{view.path}</code>. Vor der ersten Änderung legt Brickstrap daneben eine Sicherung an.</p>
  {/if}

  {#if aenderungen}
    <div class="speichernleiste">
      <span><strong class="zahl">{aenderungen}</strong> {aenderungen === 1 ? "Änderung" : "Änderungen"}</span>
      <button class="knopf flach" onclick={() => (entwurf = {})}>Verwerfen</button>
      <button class="knopf primaer" onclick={speichern} disabled={speichert || laeuft}>{speichert ? "Speichert …" : "Speichern"}</button>
    </div>
  {/if}
</div>

<style>
  .roblox { display: flex; flex-direction: column; gap: 20px; max-width: var(--inhalt-max); padding-bottom: 60px; }
  header { display: flex; flex-direction: column; gap: 8px; }
  .lead { color: var(--t3); max-width: 68ch; }
  .karte { border-radius: var(--r-l); background: var(--s1); overflow: hidden; }
  .karte.gesperrt { opacity: 0.6; }
  .zeile { display: flex; align-items: center; justify-content: space-between; gap: 24px; padding: 14px 16px 14px 18px; border-top: 1px solid var(--linie); min-height: 64px; }
  .zeile:first-child { border-top: 0; }
  .text { display: flex; flex-direction: column; gap: 3px; min-width: 0; }
  .label { font-weight: 600; }
  .hint { color: var(--t3); font-size: 13px; }
  .steuer { display: flex; align-items: center; gap: 12px; flex: none; }
  .steuer select { min-width: 150px; }
  .zahlfeld { width: 120px; text-align: right; }
  .wert { min-width: 36px; text-align: right; font-weight: 600; }
  .regler { width: 180px; accent-color: #fff; }
  code { font-family: var(--mono); font-size: 12.5px; color: var(--t2); }
  .weitere { display: flex; flex-direction: column; gap: 10px; align-items: flex-start; }
  .weitere .karte { width: 100%; }
  .pfeil { display: inline-flex; transition: transform 160ms var(--ein); }
  .pfeil.offen { transform: rotate(90deg); }
  .pfad { font-size: 12.5px; }
  .speichernleiste {
    position: sticky;
    bottom: 12px;
    display: flex;
    align-items: center;
    gap: 10px;
    align-self: flex-end;
    padding: 8px 8px 8px 16px;
    border-radius: 12px;
    background: var(--s3);
    box-shadow: 0 10px 30px #00000090, 0 0 0 1px #ffffff12;
    animation: auftauchen 200ms var(--ein);
  }
  .speichernleiste span { margin-right: 6px; }
  @keyframes auftauchen { from { opacity: 0; transform: translateY(8px); } }
</style>
