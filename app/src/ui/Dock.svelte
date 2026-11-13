<script>
  // Leiste unten: was Roblox gerade macht, aktives FastFlag-Preset, Roblox starten.
  import Icon from "./Icon.svelte";
  import Auswahl from "./Auswahl.svelte";
  import { app, launch, starting, go, openGame, kopieren, letzterServer } from "../lib/app.svelte.js";
  import { flags, presetWaehlen, presetName } from "../lib/flags.svelte.js";
  import { call } from "../lib/bridge.js";
  import { dauer } from "../lib/format.js";

  let jetzt = $state(Math.floor(Date.now() / 1000));
  $effect(() => {
    const t = setInterval(() => (jetzt = Math.floor(Date.now() / 1000)), 15_000);
    return () => clearInterval(t);
  });

  const a = $derived(app.activity);
  const spielt = $derived(a.state === "playing" || a.state === "joining");

  const zuletzt = $derived(letzterServer());

  async function einladen() {
    if (!a.placeId) return;
    const link = await call("invite_link", { placeId: a.placeId, jobId: a.jobId ?? null });
    await kopieren(link, a.jobId ? "Link zu deinem Server kopiert" : "Link zur Spielseite kopiert");
  }
</script>

<footer class="dock">
  <div class="status">
    {#if spielt}
      <span class="lampe an" aria-hidden="true"></span>
      <div class="zeilen">
        <span class="titel">
          {#if a.state === "joining"}Tritt bei …{:else}Spielt {a.gameName ?? "Roblox"}{/if}
        </span>
        <span class="unter zahl">
          {#if a.since}seit {dauer(jetzt - a.since)}{/if}
          {#if a.region}<span title="Geschätzt aus der Adresse {a.serverIp}"> · Server in {a.region}</span>{:else if a.serverIp} · Server {a.serverIp}{/if}
        </span>
      </div>
      {#if a.universeId}
        <button class="knopf flach klein" onclick={() => openGame({ universeId: a.universeId, placeId: a.placeId, name: a.gameName ?? "" })}>Spielseite</button>
      {/if}
      <button class="knopf flach klein" onclick={einladen} title={a.jobId ? "Wer den Link öffnet, landet auf deinem Server" : "Link zur Spielseite"}>
        <Icon name="kopieren" size={14} />{a.jobId ? "Einladung kopieren" : "Link kopieren"}
      </button>
    {:else if !app.roblox.installed}
      <span class="lampe warnung" aria-hidden="true"></span>
      <div class="zeilen">
        <span class="titel">Roblox ist nicht installiert</span>
        <span class="unter">Installier Roblox einmal von roblox.com. Brickstrap startet es dann.</span>
      </div>
    {:else}
      <span class="lampe" aria-hidden="true"></span>
      <div class="zeilen">
        <span class="titel">Bereit</span>
        <span class="unter">
          {#if app.update?.updateAvailable}Roblox aktualisiert sich beim nächsten Start{:else if zuletzt}Zuletzt {zuletzt.name}{:else}Roblox ist geschlossen{/if}
        </span>
      </div>
      {#if zuletzt}
        <button class="knopf flach klein" onclick={() => launch(zuletzt, zuletzt.jobId)} disabled={!!starting.key} title="Zurück auf den Server von {zuletzt.name}">
          <Icon name="neu" size={14} />Wieder beitreten
        </button>
      {/if}
    {/if}
  </div>

  <div class="rechts">
    {#if flags.geladen}
      <div class="preset">
        <span class="leise">FastFlags</span>
        <Auswahl
          label="FastFlag-Preset"
          oben
          breite="160px"
          value={flags.preset}
          onchange={presetWaehlen}
          options={[{ value: "aus", label: "Aus" }, ...flags.presets.map((p) => ({ value: p.id, label: p.name })), ...(flags.preset === "eigene" ? [{ value: "eigene", label: "Eigene" }] : [])]}
        />
      </div>
      <button class="knopf flach klein rund" onclick={() => go("flags")} aria-label="FastFlags bearbeiten" title="{presetName(flags.preset)} bearbeiten">
        <Icon name="flags" size={16} />
      </button>
    {/if}
    {#if spielt}
      <button class="knopf start" disabled title="Roblox läuft schon. Beende es, um neu zu starten.">
        Roblox läuft
      </button>
    {:else}
      <button class="knopf primaer start" onclick={() => launch()} disabled={starting.key === "app" || !app.roblox.installed}>
        <Icon name="spielen" size={16} />
        {starting.key ? "Startet …" : "Roblox starten"}
      </button>
    {/if}
  </div>
</footer>

<style>
  .dock {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    height: var(--dock);
    padding: 0 10px 0 18px;
    background: var(--s1);
    border-top: 1px solid var(--linie);
  }
  .status { display: flex; align-items: center; gap: 12px; min-width: 0; }
  .lampe { width: 8px; height: 8px; border-radius: 50%; background: var(--t4); flex: none; }
  .lampe.an { background: var(--live); }
  .lampe.warnung { background: var(--warnung); }
  .zeilen { display: flex; flex-direction: column; min-width: 0; margin-right: 4px; }
  .titel { font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 340px; }
  .unter { font-size: 12.5px; color: var(--t3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .rechts { display: flex; align-items: center; gap: 8px; flex: none; }
  .preset { display: flex; align-items: center; gap: 10px; font-size: 13px; }
  .preset :global(.knopffeld) { height: 32px; }
  .start { height: 36px; padding: 0 16px; }
  .zeilen { flex-direction: row; align-items: baseline; gap: 10px; }
  /* Schmales Fenster: Platz für den Spielnamen statt für die Beschriftung. */
  @media (max-width: 1100px) {
    .preset .leise { display: none; }
    .titel { max-width: 220px; }
  }
</style>
