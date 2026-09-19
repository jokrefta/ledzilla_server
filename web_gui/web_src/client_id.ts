const STORAGE_KEY = "ledzilla-client-id";

/** Stable per-browser client ID, sent as the Ledzilla-Client-ID header on
 *  state-modifying requests. Generated once and cached in localStorage so
 *  it survives reloads (a page refresh isn't a "new" client). */
export function getClientId(): string {
  let id = localStorage.getItem(STORAGE_KEY);
  if (!id) {
    id = `web-${Math.random().toString(36).slice(2, 9)}`;
    localStorage.setItem(STORAGE_KEY, id);
  }
  return id;
}

// Whether we've attempted a write yet this page load. The "did someone
// else touch it since?" check is meaningless before our own first write,
// so it's skipped until this flips true.
let hasWrittenThisSession = false;

export function isFirstWriteThisSession(): boolean {
  return !hasWrittenThisSession;
}

export function markWrittenThisSession(): void {
  hasWrittenThisSession = true;
}
