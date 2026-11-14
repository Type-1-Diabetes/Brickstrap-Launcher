// Updates von Brickstrap: Zustand für den großen Update-Dialog.
// Rust sucht beim Start und alle 3 Stunden (Ereignis `update`); der Knopf in den
// Einstellungen sucht sofort. „Später“ blendet dieselbe Version bis zum Neustart aus.
import { call, on } from "./bridge.js";
import { fehlerText } from "./errors.js";
import { toast } from "./app.svelte.js";

export const upd = $state({
  info: null, // { version, current, notes, size }
  offen: false,
  sucht: false,
  laedt: false,
  fortschritt: 0, // 0…1
  geladen: 0,
  gesamt: 0,
  fehler: null,
  fertig: false,
});

let spaeter = null;

function zeigen(info, manuell) {
  if (!info) return;
  if (!manuell && spaeter === info.version) return;
  upd.info = info;
  upd.fehler = null;
  upd.offen = true;
}

export function updateStart() {
  on("update", (info) => zeigen(info, false));
  on("update-progress", (p) => {
    upd.geladen = p.done;
    upd.gesamt = p.total;
    upd.fortschritt = p.total ? Math.min(1, p.done / p.total) : 0;
  });
}

/** Knopf „Nach Updates suchen“. */
export async function updateSuchen() {
  if (upd.sucht) return;
  upd.sucht = true;
  try {
    const info = await call("update_check", { force: true });
    if (info) zeigen(info, true);
    else toast("Du hast die neueste Version");
  } catch (e) {
    const f = fehlerText(e);
    toast("Suche nach Updates fehlgeschlagen", "fehler", f.hinweis || f.titel);
  } finally {
    upd.sucht = false;
  }
}

export async function updateInstallieren() {
  if (upd.laedt) return;
  upd.laedt = true;
  upd.fehler = null;
  upd.fortschritt = 0;
  try {
    await call("update_install");
    upd.fertig = true; // die App beendet sich gleich selbst
  } catch (e) {
    // Rust liefert „CODE|Text“ mit einem Text, der genau dieses Update betrifft.
    const roh = String(e?.message ?? e ?? "");
    const text = roh.includes("|") ? roh.split("|").slice(1).join("|") : "";
    upd.fehler = { titel: "Update fehlgeschlagen.", hinweis: text || fehlerText(e).hinweis || "Versuch es gleich noch einmal." };
  } finally {
    upd.laedt = false;
  }
}

export function updateSpaeter() {
  if (upd.laedt || upd.fertig) return;
  spaeter = upd.info?.version ?? null;
  upd.offen = false;
}
