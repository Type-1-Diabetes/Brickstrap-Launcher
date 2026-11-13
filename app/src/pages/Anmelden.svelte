<script>
  // Anmeldung ist Pflicht (Entscheidung des Projektinhabers vom 13.11.2026).
  import Icon from "../ui/Icon.svelte";
  import zeichen from "../ui/zeichen.svg";
  import wortmarke from "../ui/wortmarke.svg";
  import { call } from "../lib/bridge.js";
  import { toast } from "../lib/app.svelte.js";
  import { fehlerText } from "../lib/errors.js";

  let wartet = $state(false);

  async function anmelden() {
    wartet = true;
    try {
      await call("auth_login");
    } catch (e) {
      wartet = false;
      const f = fehlerText(e);
      toast("Login-Fenster ließ sich nicht öffnen", "fehler", f.hinweis);
    }
  }
</script>

<div class="anmelden">
  <div class="karte">
    <div class="marke-oben">
      <img src={zeichen} alt="" width="40" height="40" />
      <img src={wortmarke} alt="Brickstrap" height="22" />
    </div>
    <h1>Melde dich mit deinem Roblox-Konto an</h1>
    <p class="lead">Brickstrap zeigt dir damit deine Freunde, ihre Server und dein Konto. Du meldest dich direkt auf der Seite von Roblox an.</p>
    <button class="knopf primaer gross breit" onclick={anmelden} disabled={wartet}>
      {wartet ? "Warte auf die Anmeldung …" : "Bei Roblox anmelden"}
    </button>
    {#if wartet}
      <p class="leise klein">Melde dich im neuen Fenster an. Es schließt sich danach von selbst. <button class="link" onclick={anmelden}>Fenster wieder öffnen</button></p>
    {/if}
    <ul class="punkte">
      <li><Icon name="haken" size={15} />Dein Passwort siehst nur du und Roblox, Brickstrap bekommt es nie.</li>
      <li><Icon name="haken" size={15} />Die Anmeldung liegt verschlüsselt im Windows-Anmeldeinformations-Manager.</li>
      <li><Icon name="haken" size={15} />Brickstrap spricht damit nur mit roblox.com, mit keinem anderen Server.</li>
    </ul>
  </div>
  <p class="fuss leise">Kein Produkt von Roblox Corporation.</p>
</div>

<style>
  .anmelden {
    height: 100vh; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 20px;
    padding: 24px; background: var(--s0);
  }
  .karte {
    width: min(460px, 100%); display: flex; flex-direction: column; gap: 18px;
    padding: 32px; border-radius: 16px; background: var(--s1); border: 1px solid var(--linie);
    animation: rein 260ms var(--ein);
  }
  @keyframes rein { from { opacity: 0; transform: translateY(8px); } }
  .marke-oben { display: flex; align-items: center; gap: 12px; margin-bottom: 6px; }
  h1 { font-size: 24px; }
  .lead { color: var(--t2); }
  .breit { width: 100%; }
  .klein { font-size: 12.5px; margin-top: -8px; }
  .link { padding: 0; border: 0; background: none; color: var(--t1); font: inherit; text-decoration: underline; text-underline-offset: 3px; cursor: default; }
  .punkte { list-style: none; margin: 4px 0 0; padding: 16px 0 0; border-top: 1px solid var(--linie); display: flex; flex-direction: column; gap: 10px; color: var(--t3); font-size: 13px; }
  .punkte li { display: flex; gap: 10px; align-items: flex-start; }
  .punkte :global(.icon) { margin-top: 2px; color: var(--t2); }
  .fuss { font-size: 12px; }
</style>
