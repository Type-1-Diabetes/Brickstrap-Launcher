<script>
  // Ein Spieler in einer Liste: Kopfbild, Anzeigename, @Benutzername. Klick öffnet das Profil.
  import Icon from "./Icon.svelte";
  import { openUser } from "../lib/app.svelte.js";

  /** @type {{ user: { userId: number, name: string, displayName: string, verified?: boolean, avatarUrl?: string|null } }} */
  let { user } = $props();
  let geladen = $state(false);
</script>

<button class="zeile" onclick={() => openUser(user)} title="Profil von {user.displayName || user.name} öffnen">
  <span class="ava" class:geladen>
    <span class="init" aria-hidden="true">{(user.displayName || user.name || "?").slice(0, 1).toUpperCase()}</span>
    {#if user.avatarUrl}<img src={user.avatarUrl} alt="" loading="lazy" decoding="async" onload={() => (geladen = true)} />{/if}
  </span>
  <span class="tx">
    <span class="nm">
      <span class="anzeige">{user.displayName || user.name}</span>
      {#if user.verified}<span class="geprueft" title="Verifiziert"><Icon name="haken" size={10} /></span>{/if}
    </span>
    <small>@{user.name}</small>
  </span>
  <span class="pfeil" aria-hidden="true"><Icon name="rechts" size={16} /></span>
</button>

<style>
  .zeile {
    display: flex; align-items: center; gap: 14px;
    padding: 8px 10px; margin: 0 -10px; width: calc(100% + 20px);
    border: 0; border-radius: var(--r-l); background: none;
    color: var(--t1); font: inherit; text-align: left; cursor: default;
    transition: background-color 120ms ease;
  }
  .zeile:hover { background: var(--s1); }
  .zeile:active .ava { transform: scale(0.97); }
  .zeile:focus-visible { outline: 2px solid var(--t1); outline-offset: -2px; }
  .ava {
    position: relative; width: 52px; height: 52px; flex: none; border-radius: 50%; overflow: hidden;
    background: var(--s2); box-shadow: inset 0 0 0 1px var(--rand-bild);
    transition: transform 140ms var(--ein);
  }
  .init { position: absolute; inset: 0; display: grid; place-items: center; color: var(--t4); font-weight: 800; font-size: 18px; }
  img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; opacity: 0; transition: opacity 220ms ease; }
  .geladen img { opacity: 1; }
  .tx { display: flex; flex-direction: column; gap: 1px; min-width: 0; flex: 1; }
  .nm { display: flex; align-items: center; gap: 6px; min-width: 0; }
  .anzeige { font-weight: 600; font-size: 15px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  small { color: var(--t3); font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .geprueft { display: grid; place-items: center; width: 15px; height: 15px; flex: none; border-radius: 50%; background: var(--t2); color: var(--s0); }
  .pfeil { color: var(--t3); opacity: 0; transform: translateX(-4px); transition: opacity 140ms ease, transform 160ms var(--ein); }
  .zeile:hover .pfeil, .zeile:focus-visible .pfeil { opacity: 1; transform: none; }
</style>
