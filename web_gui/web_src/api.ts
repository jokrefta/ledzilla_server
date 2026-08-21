import type { DisplayInfo, DisplayState } from "./types.js";
import { readCard, loadComponentsFromState } from "./card.js";
import { setAvailableFonts } from "./components.js";
import { log } from "./log.js";

const BASE = "/api";

/** Thin fetch wrapper shared by every module that talks to the API
 *  (this file, and files.ts for the /files endpoints). Logs every call
 *  to the activity log and normalizes 204/error handling. */
export async function apiCall(method: string, path: string, body?: object, isFormData = false): Promise<any> {
  const opts: RequestInit = { method };
  if (body !== undefined) {
    if (isFormData) {
      opts.body = body as FormData;
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

  // Not every successful response is JSON (e.g. PUT /files/<name> returns a
  // plain-text confirmation message on 201). Only attempt to parse bodies
  // that actually claim to be JSON; anything else is treated as "no useful
  // payload" rather than throwing a SyntaxError out of this call.
  const contentType = res.headers.get("content-type") ?? "";
  if (!text || !contentType.includes("application/json")) {
    return null;
  }
  return JSON.parse(text);
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

  async displayOn(): Promise<void> {
    await apiCall("POST", "/display/on");
  },

  async displayOff(): Promise<void> {
    await apiCall("POST", "/display/off");
  },
};
