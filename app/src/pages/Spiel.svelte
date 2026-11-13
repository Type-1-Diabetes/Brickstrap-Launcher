<script>
  // Spielseite: Kopf mit Spielen/Anpinnen, darunter der Server-Picker.
  import { onMount } from "svelte";
  import Bild from "../ui/Bild.svelte";
  import Icon from "../ui/Icon.svelte";
  import Hinweis from "../ui/Hinweis.svelte";
  import Serverliste from "../ui/Serverliste.svelte";
  import { launch, starting, togglePin, isPinned, kopieren } from "../lib/app.svelte.js";
  import { call, openExternal } from "../lib/bridge.js";
  import { fehlerText } from "../lib/errors.js";
  import { zahl, dauer, seit } from "../lib/format.js";

  /** @type {{ params: { universeId: number, placeId: number, name: string } }} */
  let { params } = $props();

  let detail = $state(null);
  let fehler = $state(null);
  let mehrText = $state(false);
  let meineZeit = $state(null);

  const game = $derived({
    universeId: params.universeId,
    placeId: detail?.placeId || params.placeId,
    name: detail?.name || params.name,
  });

  onMount(async () => {
    try {
      const [d] = await call("game_details", { universeIds: [params.universeId] });
      detail = d ?? null;
      if (!d) fehler = { code: "FORMAT", message: "" };
    } catch (e) {
      fehler = e;
    }
  });

  onMount(() => {
    call("playtime_get")
      .then((p) => (meineZeit = p?.games?.find((g) => g.universeId === params.universeId) ?? null))
      .catch(() => {});
  });

  async function linkKopieren() {
    const link = await call("invite_link", { placeId: game.placeId });
    await kopieren(link, "Link zur Spielseite kopiert");
  }

  const pinned = $derived(isPinned(game.universeId));
  const f = $derived(fehler ? fehlerText(fehler) : null);
</script>

<div class="spiel">
  <section class="kopf">
    <div class="titelbild">
      <Bild universeId={game.universeId} name={game.name} kind="banner" eager />
    </div>
    <div class="info">
      <div class="titel">
        <h1>{game.name}</h1>
        {#if detail}
          <p class="ersteller">
            von <span class="silber">{detail.creatorName}</span>
            {#if detail.creatorVerified}<span class="geprueft" title="Verifiziert"><Icon name="haken" size={11} /></span>{/if}
          </p>
        {:else if !fehler}
          <div class="skelett" style="width: 140px; height: 16px"></div>
        {/if}
        {#if meineZeit}
          <p class="meinezeit zahl"><Icon name="uhr" size={14} />Du hast {dauer(meineZeit.totalSecs)} gespielt · zuletzt {seit(meineZeit.lastPlayed)}</p>
        {/if}
      </div>

      {#if detail}
        <dl class="werte zahl">
          <div><dt>Online</dt><dd><span class="punkt" aria-hidden="true"></span>{zahl(detail.playing)}</dd></div>
          <div><dt>Besuche</dt><dd>{zahl(detail.visits)}</dd></div>
          <div><dt>Favoriten</dt><dd>{zahl(detail.favorites)}</dd></div>
          <div><dt>Pro Server</dt><dd>{detail.maxPlayers}</dd></div>
        </dl>
      {/if}

      <div class="aktionen">
        <button class="knopf primaer gross" onclick={() => launch(game)} disabled={!game.placeId || starting.key === `${game.placeId}|`}>
          <Icon name="spielen" size={16} />Spielen
        </button>
        <button class="knopf gross" aria-pressed={pinned} onclick={() => togglePin(game)}>
          <Icon name={pinned ? "pinFill" : "pin"} size={16} />{pinned ? "Angepinnt" : "Anpinnen"}
        </button>
        <button class="knopf flach gross rund" onclick={linkKopieren} aria-label="Link kopieren" title="Link zur Spielseite kopieren">
          <Icon name="kopieren" size={17} />
        </button>
        <button class="knopf flach gross rund" onclick={() => openExternal(`https://www.roblox.com/games/${game.placeId}`)} aria-label="Im Browser öffnen" title="Spielseite im Browser öffnen">
          <Icon name="extern" size={17} />
        </button>
      </div>
      <p class="tipp leise">„Spielen“ wählt einen Server für dich. Unten suchst du dir selbst einen aus.</p>
    </div>
  </section>

  {#if fehler && !detail}
    <Hinweis art="fehler" kompakt titel={f.titel} text="Die Details fehlen. „Spielen“ funktioniert trotzdem." />
  {/if}

  {#if detail?.description}
    <section class="beschreibung">
      <p class="waehlbar" class:offen={mehrText}>{detail.description}</p>
      {#if detail.description.length > 220}
        <button class="knopf flach klein" onclick={() => (mehrText = !mehrText)}>{mehrText ? "Weniger" : "Mehr lesen"}</button>
      {/if}
    </section>
  {/if}

  {#if game.placeId}
    <Serverliste {game} />
  {/if}
</div>

<style>
  .spiel { display: flex; flex-direction: column; gap: 28px; padding-top: 4px; }
  .kopf { display: grid; grid-template-columns: minmax(300px, 440px) 1fr; gap: 28px; align-items: start; }
  .titelbild { border-radius: var(--r-l); overflow: hidden; box-shadow: inset 0 0 0 1px var(--rand-bild); }
  .info { display: flex; flex-direction: column; gap: 18px; min-width: 0; padding-top: 2px; }
  .titel { display: flex; flex-direction: column; gap: 6px; }
  h1 { font-size: 30px; overflow-wrap: anywhere; }
  .meinezeit { display: flex; align-items: center; gap: 7px; color: var(--t2); font-size: 13px; }
  .ersteller { color: var(--t3); display: flex; align-items: center; gap: 6px; }
  .geprueft { display: grid; place-items: center; width: 16px; height: 16px; border-radius: 50%; background: var(--t2); color: var(--s0); }
  .werte { display: flex; flex-wrap: wrap; gap: 8px 28px; margin: 0; }
  .werte div { display: flex; flex-direction: column; gap: 2px; }
  dt { font-size: 12px; color: var(--t3); }
  dd { margin: 0; font-size: 18px; font-weight: 700; font-stretch: 108%; display: flex; align-items: center; gap: 7px; }
  .punkt { width: 7px; height: 7px; border-radius: 50%; background: var(--live); }
  .aktionen { display: flex; flex-wrap: wrap; gap: 8px; }
  .tipp { font-size: 12.5px; margin-top: -8px; }
  .beschreibung { display: flex; flex-direction: column; align-items: flex-start; gap: 6px; max-width: 80ch; }
  .beschreibung p {
    color: var(--t2);
    white-space: pre-line;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .beschreibung p.offen { display: block; }
  @media (max-width: 1100px) {
    .kopf { grid-template-columns: 300px 1fr; gap: 22px; }
    h1 { font-size: 24px; }
  }
</style>
