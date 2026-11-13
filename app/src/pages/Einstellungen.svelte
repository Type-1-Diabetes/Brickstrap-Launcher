<script>
  // Einstellungen von Brickstrap selbst.
  import Icon from "../ui/Icon.svelte";
  import { app, saveConfig, toast } from "../lib/app.svelte.js";
  import { onMount } from "svelte";
  import { call, openExternal, isDemo } from "../lib/bridge.js";

  let appId = $state(app.config?.discordAppId ?? "");
  let eingebaut = $state(false);
  onMount(() => call("discord_builtin").then((b) => (eingebaut = b)).catch(() => {}));

  async function setze(patch) {
    try {
      await saveConfig(patch);
    } catch {
      toast("Einstellung nicht gespeichert", "fehler");
    }
  }

  const discordBereit = $derived(app.config?.discord && (eingebaut || app.config?.discordAppId));
</script>

<div class="einst">
  <h1>Einstellungen</h1>

  {#if app.config}
    <section>
      <h2>Discord-Status</h2>
      <div class="karte">
        <div class="zeile">
          <div class="text">
            <span class="label">Spiel als Discord-Status zeigen</span>
            <p class="hint">Deine Freunde sehen, was du spielst und seit wann. Discord muss dafür am PC laufen.</p>
          </div>
          <span class="schalter"><input type="checkbox" aria-label="Discord-Status" checked={app.config.discord} onchange={(e) => setze({ discord: e.currentTarget.checked })} /><span></span></span>
        </div>
        <div class="zeile">
          <div class="text">
            <span class="label">Knopf „Server beitreten“</span>
            <p class="hint">Freunde können über deinen Status direkt deinem Server beitreten.</p>
          </div>
          <span class="schalter"><input type="checkbox" aria-label="Knopf Server beitreten" disabled={!app.config.discord} checked={app.config.discordJoinButton} onchange={(e) => setze({ discordJoinButton: e.currentTarget.checked })} /><span></span></span>
        </div>
        <div class="zeile">
          <div class="text">
            <span class="label">{eingebaut ? "Eigene Discord-Anwendung (optional)" : "Discord-Anwendungs-ID"}</span>
            <p class="hint">
              {#if eingebaut}
                Leer lassen, dann erscheint „Brickstrap“ im Status. Mit einer eigenen ID aus dem Discord Developer Portal steht dort der Name deiner Anwendung.
              {:else}
                In dieser Version fehlt noch die Discord-Anwendung von Brickstrap. Leg im Discord Developer Portal eine eigene an und trag ihre ID hier ein. Ohne ID bleibt der Status aus.
              {/if}
            </p>
            {#if app.config.discord && !eingebaut && !app.config.discordAppId}<span class="marke warnung">Status aus: keine ID</span>{/if}
            {#if discordBereit}<span class="marke">Aktiv</span>{/if}
          </div>
          <div class="steuer">
            <input class="feld idfeld zahl" inputmode="numeric" placeholder="z. B. 1234567890" aria-label="Discord-Anwendungs-ID" value={appId} oninput={(e) => { appId = e.currentTarget.value.replace(/\D/g, ""); e.currentTarget.value = appId; }} onchange={() => setze({ discordAppId: appId })} />
            <button class="knopf flach klein rund" onclick={() => openExternal("https://discord.com/developers/applications")} aria-label="Developer Portal öffnen" title="Developer Portal öffnen"><Icon name="extern" size={15} /></button>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>Verhalten</h2>
      <div class="karte">
        <div class="zeile">
          <div class="text">
            <span class="label">Beim Schließen im Infobereich bleiben</span>
            <p class="hint">Das Fenster schließt sich, Brickstrap läuft im Infobereich neben der Uhr weiter. So öffnet es sich schneller und zählt weiter deine Spielzeit.</p>
          </div>
          <span class="schalter"><input type="checkbox" aria-label="Im Infobereich bleiben" checked={app.config.closeToTray} onchange={(e) => setze({ closeToTray: e.currentTarget.checked })} /><span></span></span>
        </div>
        <div class="zeile">
          <div class="text">
            <span class="label">Nach dem Start von Roblox ausblenden</span>
            <p class="hint">Das Fenster verschwindet, sobald Roblox startet. Ein Klick auf das Symbol im Infobereich holt es zurück.</p>
          </div>
          <span class="schalter"><input type="checkbox" aria-label="Nach dem Start ausblenden" checked={app.config.hideOnLaunch} onchange={(e) => setze({ hideOnLaunch: e.currentTarget.checked })} /><span></span></span>
        </div>
        <div class="zeile">
          <div class="text">
            <span class="label">Volle Server ausblenden</span>
            <p class="hint">Standard für die Serverliste. Auf jeder Spielseite umschaltbar.</p>
          </div>
          <span class="schalter"><input type="checkbox" aria-label="Volle Server ausblenden" checked={app.config.hideFullServers} onchange={(e) => setze({ hideFullServers: e.currentTarget.checked })} /><span></span></span>
        </div>
      </div>
    </section>

    <section>
      <h2>Über Brickstrap</h2>
      <div class="karte ueber">
        <p><strong>Brickstrap {__APP_VERSION__}</strong>{#if isDemo} · Vorschau im Browser mit Demo-Daten{/if}</p>
        <p class="hint">Brickstrap startet den offiziellen Roblox-Client nur von außen, per Link oder über das Programm von Roblox. Es ändert nichts am Client, greift nicht auf seinen Speicher zu und braucht kein Login.</p>
        <p class="hint">Kein Produkt von Roblox Corporation. Roblox ist eine Marke der Roblox Corporation.</p>
        <p class="hint">Schrift: Archivo (SIL Open Font License).</p>
      </div>
    </section>
  {/if}
</div>

<style>
  .einst { display: flex; flex-direction: column; gap: 28px; max-width: var(--inhalt-max); }
  section { display: flex; flex-direction: column; gap: 10px; }
  .karte { border-radius: var(--r-l); background: var(--s1); overflow: hidden; }
  .zeile { display: flex; align-items: center; justify-content: space-between; gap: 24px; padding: 14px 16px 14px 18px; border-top: 1px solid var(--linie); }
  .zeile:first-child { border-top: 0; }
  .text { display: flex; flex-direction: column; gap: 4px; align-items: flex-start; min-width: 0; }
  .label { font-weight: 600; }
  .hint { color: var(--t3); font-size: 13px; max-width: 64ch; }
  .steuer { display: flex; align-items: center; gap: 6px; flex: none; }
  .idfeld { width: 200px; }
  .ueber { display: flex; flex-direction: column; gap: 8px; padding: 16px 18px; }
</style>
