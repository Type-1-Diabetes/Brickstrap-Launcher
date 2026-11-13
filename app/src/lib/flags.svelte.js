// FastFlags: gemeinsamer Zustand für Editor-Seite und Preset-Auswahl im Dock.

import { call } from "./bridge.js";
import { toast } from "./app.svelte.js";
import { fehlerText } from "./errors.js";

export const flags = $state({
  geladen: false,
  allowlist: [],
  values: {},
  preset: "aus",
  presets: [],
  allowlistDate: "",
  appliedTo: [],
  lastApplied: null,
  gespeichert: true,
});

export async function ladeFlags() {
  const [view, presets] = await Promise.all([call("flags_get"), call("flags_presets")]);
  flags.allowlist = view?.allowlist ?? [];
  flags.values = view?.values ?? {};
  // Altes Preset „Schwacher Laptop“ ist in „Max FPS“ aufgegangen (gleiche Werte).
  flags.preset = view?.preset === "schwacher-laptop" ? "max-fps" : view?.preset || "aus";
  flags.allowlistDate = view?.allowlistDate ?? "";
  flags.appliedTo = view?.appliedTo ?? [];
  flags.lastApplied = view?.lastApplied ?? null;
  flags.presets = presets ?? [];
  flags.geladen = true;
}

let timer;
/** Speichert verzögert, damit schnelle Änderungen nur einmal schreiben. */
export function speichern(sofort = false) {
  flags.gespeichert = false;
  clearTimeout(timer);
  const run = async () => {
    try {
      await call("flags_set", { values: $state.snapshot(flags.values), preset: flags.preset });
      flags.gespeichert = true;
    } catch (e) {
      const f = fehlerText(e);
      toast("FastFlags nicht gespeichert", "fehler", f.hinweis);
      // Anzeige wieder auf den gespeicherten Stand bringen.
      ladeFlags().catch(() => {});
    }
  };
  if (sofort) run();
  else timer = setTimeout(run, 400);
}

/** Preset übernehmen: ersetzt alle Werte. "aus" leert die Liste. */
export function presetWaehlen(id) {
  flags.preset = id;
  if (id === "aus") flags.values = {};
  else {
    const p = flags.presets.find((x) => x.id === id);
    if (p) flags.values = { ...p.values };
  }
  speichern(true);
}

/** Einzelnen Wert setzen; macht aus einem Preset „Eigene“. */
export function setzen(name, value) {
  const f = flags.allowlist.find((x) => x.name === name);
  const next = { ...flags.values };
  if (value === null || value === undefined || value === "" || Number.isNaN(value)) delete next[name];
  else if (f?.kind === "int" && typeof value === "number") {
    // Gleiche Grenzen wie im Rust-Teil, damit Anzeige und Datei übereinstimmen.
    next[name] = String(Math.min(f.max ?? Infinity, Math.max(f.min ?? -Infinity, value)));
  } else next[name] = String(value);
  // Nur eine Grafik-API gleichzeitig
  if (f?.exclusive && value === "True") {
    for (const o of flags.allowlist) if (o.exclusive === f.exclusive && o.name !== name) delete next[o.name];
  }
  flags.values = next;
  if (flags.preset !== "eigene") flags.preset = "eigene";
  speichern();
}

export const presetName = (id) =>
  id === "aus" ? "Aus" : id === "eigene" ? "Eigene" : (flags.presets.find((p) => p.id === id)?.name ?? id);
