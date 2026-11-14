import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";
import { isDemo } from "./lib/bridge.js";

// Im Programm (nicht in der Browser-Vorschau): WebView2 zeigt sonst sein eigenes Kontextmenü.
// Dessen „Bild in neuem Fenster öffnen“ startet den Bildbetrachter von Edge mit Edge-Tipps.
if (!isDemo) {
  addEventListener("contextmenu", (e) => {
    const t = e.target;
    const feld = t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement;
    const auswahl = String(getSelection() ?? "").length > 0;
    // Felder und markierter Text behalten „Kopieren“ und „Einfügen“.
    if (!feld && !auswahl) e.preventDefault();
  });
  // Bilder nicht herausziehen (sonst öffnet sich das Bild in einem neuen Fenster).
  addEventListener("dragstart", (e) => {
    if (e.target instanceof HTMLImageElement) e.preventDefault();
  });
  // Mittelklick auf Links öffnet kein zweites WebView-Fenster.
  addEventListener("auxclick", (e) => {
    if (e.button === 1 && e.target instanceof Element && e.target.closest("a")) e.preventDefault();
  });
}

mount(App, { target: document.getElementById("app") });
