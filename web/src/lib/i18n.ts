// The interface language (REQ-WEBUI-015). Text is written in English and
// wrapped in t(); the core translates it from the catalogues the egui build
// uses, and anything not translated there stays English.
import { createEffect, createRoot, createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { state } from "./bridge";

const [table, setTable] = createSignal<Record<string, string>>({});
const known = new Set<string>();
const pending = new Set<string>();
let queued = false;
const inTauri = "__TAURI_INTERNALS__" in window;

function ask(texts: string[]) {
  if (!inTauri || texts.length === 0) return;
  const locale = state.settings.locale;
  void invoke<string[]>("translate", { texts }).then((out) => {
    if (state.settings.locale !== locale) return;
    setTable((t) => {
      const next = { ...t };
      texts.forEach((text, i) => (next[text] = out[i] ?? text));
      return next;
    });
  });
}

function flush() {
  queued = false;
  const texts = [...pending];
  pending.clear();
  ask(texts);
}

/** `text` in the interface language; `{name}` placeholders take `values`. */
export function t(text: string, values?: Record<string, string | number>): string {
  if (!known.has(text)) {
    known.add(text);
    if (state.settings.locale !== "en") {
      pending.add(text);
      if (!queued) {
        queued = true;
        queueMicrotask(flush);
      }
    }
  }
  let out = state.settings.locale === "en" ? text : (table()[text] ?? text);
  if (values) for (const [k, v] of Object.entries(values)) out = out.replaceAll(`{${k}}`, String(v));
  return out;
}

// A new language asks again for everything seen so far.
createRoot(() => {
  let last = "";
  createEffect(() => {
    const locale = state.settings.locale;
    if (locale === last) return;
    last = locale;
    setTable({});
    document.documentElement.lang = locale;
    if (locale !== "en") ask([...known]);
  });
});
