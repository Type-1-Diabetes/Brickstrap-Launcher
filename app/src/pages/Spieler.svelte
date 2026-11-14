<script>
  // Profil eines Spielers: Avatar, Status, Zahlen, was er trägt, eigene Spiele und das Inventar.
  import { onMount } from "svelte";
  import Icon from "../ui/Icon.svelte";
  import Hinweis from "../ui/Hinweis.svelte";
  import Reihe from "../ui/Reihe.svelte";
  import Inventar from "../ui/Inventar.svelte";
  import { app, launch, starting, openGame, kopieren } from "../lib/app.svelte.js";
  import { call, openExternal } from "../lib/bridge.js";
  import { fehlerText } from "../lib/errors.js";
  import { zahl, monatJahr, assetArt } from "../lib/format.js";

  /** @type {{ params: { userId: number, name?: string } }} */
  let { params } = $props();

  let p = $state(null);
  let fehler = $state(null);
  let mehrText = $state(false);
  let bildDa = $state(false);

  async function laden() {
    fehler = null;
    try {
      p = await call("user_profile", { userId: params.userId });
    } catch (e) {
      fehler = e;
    }
  }
  onMount(laden);

  const name = $derived(p?.displayName || params.name || "Spieler");
  const ich = $derived(app.auth?.userId === params.userId);
  const st = $derived(p?.presence ?? null);
  const spiel = $derived(st?.universeId ? { universeId: st.universeId, placeId: st.placeId, name: st.gameName ?? "" } : null);
  const statusText = $derived(
    !st ? null
      : st.status === "spielt" ? (st.gameName ? `Spielt ${st.gameName}` : "Spielt gerade")
      : st.status === "studio" ? "In Roblox Studio"
      : st.status === "online" ? "Online"
      : "Offline",
  );
  const f = $derived(fehler ? fehlerText(fehler) : null);
</script>

<div class="profil">
  {#if fehler && !p}
    <h1>{name}</h1>
    <Hinweis art="fehler" titel={f.titel} text={f.hinweis}>
      <button class="knopf" onclick={laden}><Icon name="neu" size={16} />Erneut laden</button>
    </Hinweis>
  {:else}
    <section class="kopf">
      <div class="avatar" class:da={bildDa}>
        <span class="init" aria-hidden="true">{name.slice(0, 1).toUpperCase()}</span>
        {#if p?.avatarUrl}
          <img src={p.avatarUrl} alt="Avatar von {name}" decoding="async" onload={() => (bildDa = true)} />
        {/if}
      </div>

      <div class="info">
        <div class="titel">
          <h1>
            <span>{name}</span>
            {#if p?.verified}<span class="geprueft" title="Verifiziert"><Icon name="haken" size={12} /></span>{/if}
          </h1>
          {#if p}
            <p class="namen">
              <span class="silber waehlbar">@{p.name}</span>
              {#if ich}<span class="marke">Das bist du</span>{/if}
              {#if p.banned}<span class="marke warnung">Gesperrt</span>{/if}
            </p>
          {:else}
            <div class="skelett" style="width: 120px; height: 16px"></div>
          {/if}
          {#if statusText}
            <p class="status" class:an={st.status !== "offline"}>
              <span class="punkt" class:spielt={st.status === "spielt"} aria-hidden="true"></span>{statusText}
            </p>
          {/if}
        </div>

        {#if p}
          <dl class="werte zahl">
            <div><dt>Freunde</dt><dd>{zahl(p.friends)}</dd></div>
            <div><dt>Follower</dt><dd>{zahl(p.followers)}</dd></div>
            <div><dt>Folgt</dt><dd>{zahl(p.following)}</dd></div>
            {#if monatJahr(p.created)}<div><dt>Dabei seit</dt><dd>{monatJahr(p.created)}</dd></div>{/if}
          </dl>
        {:else}
          <div class="werte" aria-hidden="true">
            {#each [0, 1, 2, 3] as i (i)}<div class="skelett" style="width: 64px; height: 38px"></div>{/each}
          </div>
        {/if}

        <div class="aktionen">
          {#if spiel && st.jobId && spiel.placeId}
            <button class="knopf primaer gross" onclick={() => launch(spiel, st.jobId)} disabled={!!starting.key} title="Auf denselben Server wie {name}">
              <Icon name="spielen" size={16} />Beitreten
            </button>
          {/if}
          {#if spiel}
            <button class="knopf gross" onclick={() => openGame(spiel)}>
              <Icon name="start" size={16} />Spiel ansehen
            </button>
          {/if}
          {#if p}
            <button class="knopf flach gross rund" onclick={() => kopieren(p.name, `@${p.name} kopiert`)} aria-label="Benutzernamen kopieren" title="Benutzernamen kopieren">
              <Icon name="kopieren" size={17} />
            </button>
          {/if}
          <button class="knopf flach gross rund" onclick={() => openExternal(`https://www.roblox.com/users/${params.userId}/profile`)} aria-label="Im Browser öffnen" title="Profil im Browser öffnen">
            <Icon name="extern" size={17} />
          </button>
        </div>
      </div>
    </section>

    {#if p?.description}
      <section class="beschreibung">
        <p class="waehlbar" class:offen={mehrText}>{p.description}</p>
        {#if p.description.length > 220}
          <button class="knopf flach klein" onclick={() => (mehrText = !mehrText)}>{mehrText ? "Weniger" : "Mehr lesen"}</button>
        {/if}
      </section>
    {/if}

    {#if p?.wearing.length}
      <section class="traegt">
        <h2>Trägt gerade</h2>
        <ul>
          {#each p.wearing as w (w.id)}
            <li title={w.name}>
              <span class="bild">{#if w.iconUrl}<img src={w.iconUrl} alt="" loading="lazy" />{/if}</span>
              <span class="nm">{w.name}</span>
              <span class="leise art">{assetArt(w.kind)}</span>
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    {#if p?.games.length}
      <Reihe titel="Spiele von {name}" games={p.games} />
    {/if}

    {#if p}
      <Inventar userId={params.userId} {name} offen={p.inventoryPublic} />
    {/if}
  {/if}
</div>

<style>
  .profil { display: flex; flex-direction: column; gap: 28px; padding-top: 4px; }
  .kopf { display: grid; grid-template-columns: 260px 1fr; gap: 28px; align-items: start; }
  .avatar {
    position: relative;
    aspect-ratio: 1;
    border-radius: var(--r-l);
    background: var(--s1);
    overflow: hidden;
  }
  .init {
    position: absolute; inset: 0; display: grid; place-items: center;
    color: var(--t4); font-weight: 800; font-stretch: 118%; font-size: 64px;
  }
  .avatar img {
    position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain;
    opacity: 0; transform: translateY(6px);
    transition: opacity 220ms ease, transform 250ms var(--ein);
  }
  .avatar.da img { opacity: 1; transform: none; }
  .avatar.da .init { opacity: 0; }

  .info { display: flex; flex-direction: column; gap: 18px; min-width: 0; padding-top: 2px; }
  .titel { display: flex; flex-direction: column; gap: 6px; }
  h1 { font-size: 30px; display: flex; align-items: center; gap: 10px; min-width: 0; }
  h1 span:first-child { overflow-wrap: anywhere; }
  .geprueft { display: grid; place-items: center; width: 20px; height: 20px; flex: none; border-radius: 50%; background: var(--t2); color: var(--s0); }
  .namen { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .status { display: flex; align-items: center; gap: 8px; color: var(--t3); font-size: 13.5px; }
  .status.an { color: var(--t2); }
  .punkt { width: 8px; height: 8px; border-radius: 50%; background: var(--t4); flex: none; }
  .status.an .punkt { background: var(--t2); }
  .status .punkt.spielt { background: var(--live); }

  .werte { display: flex; flex-wrap: wrap; gap: 8px 28px; margin: 0; }
  .werte div { display: flex; flex-direction: column; gap: 2px; }
  dt { font-size: 12px; color: var(--t3); }
  dd { margin: 0; font-size: 18px; font-weight: 700; font-stretch: 108%; }
  .aktionen { display: flex; flex-wrap: wrap; gap: 8px; }

  .beschreibung { display: flex; flex-direction: column; align-items: flex-start; gap: 6px; max-width: 80ch; margin-top: -8px; }
  .beschreibung p {
    color: var(--t2);
    white-space: pre-line;
    overflow-wrap: anywhere;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .beschreibung p.offen { display: block; }

  .traegt { display: flex; flex-direction: column; gap: 12px; }
  h2 { font-size: 17px; }
  .traegt ul {
    list-style: none; margin: 0; padding: 0;
    display: grid; grid-template-columns: repeat(auto-fill, minmax(112px, 1fr)); gap: 16px 14px;
  }
  .traegt li { display: flex; flex-direction: column; gap: 6px; min-width: 0; }
  .bild {
    aspect-ratio: 1; border-radius: var(--r-l); background: var(--s1); overflow: hidden;
    box-shadow: inset 0 0 0 1px var(--rand-bild);
  }
  .bild img { display: block; width: 100%; height: 100%; object-fit: cover; }
  .nm { font-weight: 600; font-size: 13px; line-height: 1.3; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .art { font-size: 12px; margin-top: -4px; }

  @media (max-width: 1100px) {
    .kopf { grid-template-columns: 200px 1fr; gap: 22px; }
    h1 { font-size: 24px; }
  }
</style>
