<script>
  // Freunde, die gerade online sind. Spielende zuerst; Klick öffnet das Profil,
  // der Knopf tritt dem Server des Freundes bei (nur wenn Roblox es erlaubt).
  import { onMount } from "svelte";
  import Icon from "./Icon.svelte";
  import { launch, starting, openUser } from "../lib/app.svelte.js";
  import { call } from "../lib/bridge.js";

  const SICHTBAR = 6;
  let liste = $state(null);
  let fehler = $state(false);
  let grund = $state("");
  let alle = $state(false);

  async function laden(fresh = false) {
    try {
      liste = await call("friends_online", { fresh });
      fehler = false;
    } catch (e) {
      fehler = true;
      grund = e?.message ?? "";
      liste ??= [];
    }
  }

  onMount(() => {
    laden();
    // Jede Minute neu, aber nur, wenn das Fenster sichtbar ist.
    const t = setInterval(() => !document.hidden && laden(), 60_000);
    const sichtbar = () => !document.hidden && laden();
    document.addEventListener("visibilitychange", sichtbar);
    return () => {
      clearInterval(t);
      document.removeEventListener("visibilitychange", sichtbar);
    };
  });

  const zeige = $derived(alle ? (liste ?? []) : (liste ?? []).slice(0, SICHTBAR));
  const zeile2 = (f) =>
    f.status === "spielt" ? (f.gameName ? f.gameName : "Spielt gerade") : f.status === "studio" ? "In Roblox Studio" : "Online";
  const spiel = (f) => ({ universeId: f.universeId, placeId: f.placeId, name: f.gameName ?? "" });
</script>

<section class="block" aria-label="Freunde online">
  <div class="kopf">
    <h2>Freunde</h2>
    {#if liste?.length}<span class="anzahl leise">{liste.length} online</span>{/if}
  </div>

  {#if liste === null}
    <ul aria-hidden="true">
      {#each [0, 1, 2] as i (i)}<li class="platzhalter"><span class="ava"></span><span class="balken"></span></li>{/each}
    </ul>
  {:else if fehler && liste.length === 0}
    <p class="leise klein" title={grund || undefined}>Freunde gerade nicht erreichbar. <button class="link" onclick={() => laden(true)}>Erneut versuchen</button></p>
  {:else if liste.length === 0}
    <p class="leise klein">Gerade ist niemand online.</p>
  {:else}
    <ul>
      {#each zeige as f (f.userId)}
        <li>
          <button class="zeile" onclick={() => openUser(f)} title="Profil von {f.displayName || f.name} öffnen">
            <span class="ava">
              {#if f.avatarUrl}<img src={f.avatarUrl} alt="" loading="lazy" />{:else}<span class="init">{(f.displayName || f.name || "?").slice(0, 1).toUpperCase()}</span>{/if}
              <span class="status" class:spielt={f.status === "spielt"}></span>
            </span>
            <span class="tx">
              <span class="nm">{f.displayName || f.name}</span>
              <small>{zeile2(f)}</small>
            </span>
          </button>
          {#if f.jobId && f.placeId}
            <button
              class="knopf klein rund"
              onclick={() => launch(spiel(f), f.jobId)}
              disabled={!!starting.key}
              aria-label="Server von {f.displayName || f.name} beitreten"
              title="Beitreten"
            >
              <Icon name="spielen" size={13} />
            </button>
          {/if}
        </li>
      {/each}
    </ul>
    {#if liste.length > SICHTBAR}
      <button class="knopf flach klein mehr" onclick={() => (alle = !alle)}>
        {alle ? "Weniger" : `Alle ${liste.length} zeigen`}
      </button>
    {/if}
  {/if}
</section>

<style>
  .block { background: var(--s1); border-radius: var(--r-l); padding: 14px; display: flex; flex-direction: column; gap: 10px; }
  h2 { font-size: 13px; font-stretch: 100%; color: var(--t2); }
  .kopf { display: flex; align-items: baseline; justify-content: space-between; }
  .anzahl { font-size: 12px; font-variant-numeric: tabular-nums; }
  .klein { font-size: 12.5px; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; }
  li { display: flex; align-items: center; gap: 6px; }
  .zeile {
    flex: 1; min-width: 0; display: flex; align-items: center; gap: 10px;
    padding: 5px 6px; margin-left: -6px; border: 0; border-radius: var(--r);
    background: none; color: var(--t1); font: inherit; text-align: left; cursor: default;
    transition: background-color 120ms ease;
  }
  .zeile:hover { background: var(--s2); }
  .ava { position: relative; width: 32px; height: 32px; flex: none; border-radius: 50%; background: var(--s3); }
  .init { display: grid; place-items: center; width: 100%; height: 100%; font-size: 13px; font-weight: 600; color: var(--t2); }
  .ava img { width: 100%; height: 100%; border-radius: 50%; object-fit: cover; }
  /* Punkt unten rechts: grün = spielt, grau = nur online */
  .status {
    position: absolute; right: -1px; bottom: -1px; width: 10px; height: 10px; border-radius: 50%;
    background: var(--t3); box-shadow: 0 0 0 2px var(--s1);
  }
  .status.spielt { background: var(--live); }
  .tx { display: flex; flex-direction: column; min-width: 0; }
  .nm { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 600; font-size: 13.5px; }
  small { color: var(--t3); font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .mehr { align-self: flex-start; margin: -4px 0 0 -6px; }
  .link { border: 0; background: none; padding: 0; color: var(--t1); font: inherit; text-decoration: underline; text-underline-offset: 2px; cursor: default; }
  .platzhalter { gap: 10px; padding: 5px 0; }
  .balken { height: 10px; width: 60%; border-radius: 5px; background: var(--s3); }
</style>
