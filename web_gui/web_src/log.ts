export type LogKind = "ok" | "err" | "info" | "";

export function log(msg: string, kind: LogKind = ""): void {
  const el = document.getElementById("log");
  if (!el) return;
  const now = new Date().toLocaleTimeString("en-GB", { hour12: false });
  const row = document.createElement("div");
  row.className = "log-entry";

  const timeSpan = document.createElement("span");
  timeSpan.className = "log-time";
  timeSpan.textContent = now;

  const msgSpan = document.createElement("span");
  msgSpan.className = `log-${kind}`;
  msgSpan.textContent = msg;

  row.append(timeSpan, msgSpan);
  el.appendChild(row);
  el.scrollTop = el.scrollHeight;
}
