<script>
  // Angemeldetes Roblox-Konto (aus den Dateien von Roblox, ohne Login bei Brickstrap).
  import { app, openUser } from "../lib/app.svelte.js";
  /** @type {{ kompakt?: boolean }} */
  let { kompakt = false } = $props();
  const a = $derived(app.account);
</script>

<button class="konto" class:kompakt disabled={!a} onclick={() => a && openUser(a)} title={a ? `Angemeldet als @${a.name}. Klick öffnet dein Profil.` : "Kein Roblox-Konto erkannt"}>
  <span class="avatar">
    {#if a?.avatarUrl}<img src={a.avatarUrl} alt="" />{:else}<span>{(a?.displayName || a?.name || "?").slice(0, 1).toUpperCase()}</span>{/if}
  </span>
  <span class="text">
    {#if a}
      <strong>{a.displayName || a.name}</strong>
      <small>@{a.name}</small>
    {:else}
      <strong>Kein Konto erkannt</strong>
      <small>Starte Roblox einmal</small>
    {/if}
  </span>
</button>

<style>
  .konto {
    display: flex; align-items: center; gap: 10px; min-width: 0; width: calc(100% + 12px);
    margin: -6px; padding: 6px; border: 0; border-radius: var(--r); background: none;
    color: var(--t1); font: inherit; text-align: left; cursor: default;
    transition: background-color 120ms ease;
  }
  .konto:not(:disabled):hover { background: var(--s2); }
  .konto:disabled { opacity: 1; }
  .avatar {
    width: 36px; height: 36px; flex: none; border-radius: 50%; overflow: hidden;
    display: grid; place-items: center; background: var(--s3); color: var(--t2); font-weight: 700;
  }
  .kompakt .avatar { width: 28px; height: 28px; font-size: 12px; }
  img { width: 100%; height: 100%; object-fit: cover; }
  .text { display: flex; flex-direction: column; min-width: 0; line-height: 1.25; }
  strong, small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  strong { font-weight: 600; }
  small { color: var(--t3); font-size: 12px; }
  .kompakt strong { font-size: 13px; }
</style>
