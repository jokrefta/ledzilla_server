import type { DisplayInfo, DisplayState } from "./types.js";
import { readCard, loadComponentsFromState } from "./card.js";
import { setAvailableFonts } from "./components.js";
import { log } from "./log.js";

const BASE = "/api";

/** Thin fetch wrapper shared by every module that talks to the API
 *  (this file, and files.ts for the /files endpoints). Logs every call
 *  to the activity log and normalizes 204/error handling. */
export async function apiCall(
  method: string,
  path: string,
  body?: object | string,
  isFormData = false,
): Promise<any> {
  const opts: RequestInit = { method };
  if (body !== undefined) {
    if (isFormData) {
      opts.body = body as FormData;
    } else if (typeof body === "string") {
      opts.headers = { "Content-Type": "text/plain" };
      opts.body = body;
    } else {
      opts.headers = { "Content-Type": "application/json" };
      opts.body = JSON.stringify(body);
    }
  }

  log(`${method} ${path}`, "info");
  let res: Response;
  try {
    res = await fetch(BASE + path, opts);
  } catch (err) {
    log(`Network error: ${(err as Error).message}`, "err");
    throw err;
  }

  if (res.status === 204) {
    log("204 No Content", "ok");
    return null;
  }

  const text = await res.text();
  if (!res.ok) {
    log(`${res.status} ${text}`, "err");
    throw new Error(`${method} ${path} -> ${res.status}: ${text}`);
  }

  log(`${res.status} OK`, "ok");

  // Some endpoints return JSON, some return a plain-text body (e.g.
  // GET /display/on-off-state -> "on"/"off"). Parse JSON when the server
  // says that's what it sent; otherwise hand back the raw text as-is so
  // callers that need it (rather than just a success signal) still get it.
  if (!text) return null;
  const contentType = res.headers.get("content-type") ?? "";
  return contentType.includes("application/json") ? JSON.parse(text) : text;
}

/** Reflects the last-known display power state in the sidebar indicator.
 *  `null` means "unknown" (e.g. the GET failed or hasn't run yet). */
function setDisplayPowerIndicator(state: "on" | "off" | null): void {
  const el = document.getElementById("display-power-status");
  if (!el) return;
  el.classList.remove("power-status-on", "power-status-off", "power-status-unknown");
  if (state === "on") {
    el.textContent = "● On";
    el.classList.add("power-status-on");
  } else if (state === "off") {
    el.textContent = "● Off";
    el.classList.add("power-status-off");
  } else {
    el.textContent = "● Unknown";
    el.classList.add("power-status-unknown");
  }
}

export const api = {
  async probe(): Promise<void> {
    const info: DisplayInfo | null = await apiCall("GET", "/info").catch(() => null);
    if (info) {
      document.getElementById("display-info")!.innerHTML =
        `${info.width} × ${info.height} px<br>API v${info.api_version}`;
      setAvailableFonts(info.available_fonts);
    }
  },

  async getState(): Promise<void> {
    const state: DisplayState | null = await apiCall("GET", "/state").catch(() => null);
    if (state) {
      log(`Pulled ${state.components?.length ?? 0} components`, "info");
      loadComponentsFromState(state.components ?? []);
    }
  },

  async pushState(): Promise<void> {
    const cards = [...document.querySelectorAll<HTMLElement>(".component-card")];
    const components = cards.map((card) => {
      const { type, fields } = readCard(card);
      return { type, ...fields };
    });
    await apiCall("POST", "/state", { components });
  },

  /** GET /display/on-off-state and update the sidebar indicator to match.
   *  Called on page load, and after every displayOn()/displayOff() so the
   *  indicator reflects what the server actually reports rather than just
   *  assuming the POST took effect. */
  async refreshDisplayPower(): Promise<void> {
    const raw: string | null = await apiCall("GET", "/display/on-off-state").catch(() => null);
    const state = raw?.trim();
    setDisplayPowerIndicator(state === "on" || state === "off" ? state : null);
  },

  async displayOn(): Promise<void> {
    await apiCall("POST", "/display/on-off-state", "on");
    await api.refreshDisplayPower();
  },

  async displayOff(): Promise<void> {
    await apiCall("POST", "/display/on-off-state", "off");
    await api.refreshDisplayPower();
  },
};
