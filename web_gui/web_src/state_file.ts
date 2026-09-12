import type { DisplayState } from "./types.js";
import { buildStateFromCards, loadComponentsFromState } from "./card.js";
import { log } from "./log.js";

/** Triggers a browser "Save As" for arbitrary JSON data. Not tied to
 *  display state specifically — just a generic file-download helper. */
function downloadJson(filename: string, data: unknown): void {
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  // Deferred so the click has time to be processed before the object URL
  // backing it is invalidated.
  setTimeout(() => URL.revokeObjectURL(url), 0);
}

/** Minimal runtime validation of a parsed JSON value as a DisplayState.
 *  We don't validate individual component fields here — that's the same
 *  "unknown type? skip it" leniency loadComponentsFromState already
 *  applies, and the server is the real source of truth for full
 *  validation (POST /state will reject anything malformed). This just
 *  guards against loading an unrelated JSON file (wrong shape entirely). */
function parseDisplayState(data: unknown): DisplayState {
  if (typeof data !== "object" || data === null || !("components" in data)) {
    throw new Error('expected an object with a "components" array');
  }
  const components = (data as { components: unknown }).components;
  if (!Array.isArray(components)) {
    throw new Error('"components" must be an array');
  }
  components.forEach((c, i) => {
    if (typeof c !== "object" || c === null || typeof (c as any).type !== "string") {
      throw new Error(`component at index ${i} is missing a "type" string`);
    }
  });
  return { components } as DisplayState;
}

function timestampedFilename(): string {
  // e.g. ledzilla-state-2026-09-12T140501.json — sortable, filesystem-safe.
  const stamp = new Date().toISOString().replace(/[:.]/g, "").slice(0, 15);
  return `ledzilla-state-${stamp}.json`;
}

export function downloadCurrentState(): void {
  const state = buildStateFromCards();
  downloadJson(timestampedFilename(), state);
  log(`Downloaded state (${state.components.length} components)`, "ok");
}

async function loadStateFile(file: File): Promise<void> {
  const text = await file.text();
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (err) {
    throw new Error(`not valid JSON: ${(err as Error).message}`);
  }
  const state = parseDisplayState(parsed);
  loadComponentsFromState(state.components);
  log(`Loaded ${state.components.length} components from "${file.name}"`, "ok");
}

/** Wires up the hidden file input used for "Load state from file".
 *  Mirrors the pattern in files.ts's initFileManager(). */
export function initStateFileControls(): void {
  const downloadBtn = document.getElementById("btn-download-state");
  const loadBtn = document.getElementById("btn-load-state");
  const fileInput = document.getElementById("load-state-file") as HTMLInputElement | null;
  if (!downloadBtn || !loadBtn || !fileInput) return;

  downloadBtn.addEventListener("click", () => downloadCurrentState());

  // The visible button just proxies to the hidden file input, so we get
  // the native file picker without having to style an <input type=file>.
  loadBtn.addEventListener("click", () => fileInput.click());

  fileInput.addEventListener("change", async () => {
    const file = fileInput.files?.[0];
    // Reset immediately so choosing the *same* filename again still fires
    // a "change" event next time.
    fileInput.value = "";
    if (!file) return;

    try {
      await loadStateFile(file);
    } catch (err) {
      log(`Failed to load state from "${file.name}": ${(err as Error).message}`, "err");
    }
  });
}
