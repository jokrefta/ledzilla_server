import { apiCall } from "./api.js";
import { log } from "./log.js";

// The current file listing, kept here (not in components.ts) since this
// module owns it. components.ts reads it through getAvailableFiles().
let files: string[] = [];

export function getAvailableFiles(): string[] {
  return files;
}

async function uploadFile(
  name: string,
  file: File,
  animated: boolean,
  width?: number,
  height?: number,
  resizeFilter?: string,
): Promise<void> {
  const form = new FormData();
  form.append("file", file);
  form.append("animated", animated ? "true" : "false");
  if (width !== undefined) form.append("width", String(width));
  if (height !== undefined) form.append("height", String(height));
  if (resizeFilter) form.append("resize_filter", resizeFilter);
  await apiCall("PUT", `/files/${encodeURIComponent(name)}`, form, true);
}

async function deleteFile(name: string): Promise<void> {
  await apiCall("DELETE", `/files/${encodeURIComponent(name)}`);
}

function renderFileRow(name: string): HTMLElement {
  const row = document.createElement("div");
  row.className = "file-row";

  const nameSpan = document.createElement("span");
  nameSpan.className = "file-row-name";
  nameSpan.textContent = name;

  const deleteBtn = document.createElement("button");
  deleteBtn.className = "btn btn-remove";
  deleteBtn.textContent = "Delete";
  deleteBtn.addEventListener("click", async () => {
    if (!confirm(`Delete "${name}" from the device?`)) return;
    try {
      await deleteFile(name);
      await refreshFileList();
    } catch (err) {
      log(`Failed to delete "${name}": ${(err as Error).message}`, "err");
    }
  });

  row.append(nameSpan, deleteBtn);
  return row;
}

export async function refreshFileList(): Promise<void> {
  const result: { files: string[] } | null = await apiCall("GET", "/files").catch(() => null);
  files = result?.files ?? [];

  const list = document.getElementById("file-list");
  if (!list) return;
  list.innerHTML = "";
  if (files.length === 0) {
    const empty = document.createElement("div");
    empty.className = "file-list-empty";
    empty.textContent = "No files uploaded";
    list.appendChild(empty);
  } else {
    files.forEach((name) => list.appendChild(renderFileRow(name)));
  }
}

export function initFileManager(): void {
  const form = document.getElementById("file-upload-form") as HTMLFormElement;
  const fileInput = document.getElementById("upload-file") as HTMLInputElement;
  const nameInput = document.getElementById("upload-name") as HTMLInputElement;
  const animatedInput = document.getElementById("upload-animated") as HTMLInputElement;
  const widthInput = document.getElementById("upload-width") as HTMLInputElement;
  const heightInput = document.getElementById("upload-height") as HTMLInputElement;
  const filterSelect = document.getElementById("upload-filter") as HTMLSelectElement;

  // Prefill the name field (and guess "animated") from the chosen file.
  fileInput.addEventListener("change", () => {
    const f = fileInput.files?.[0];
    if (f) {
      if (!nameInput.value) nameInput.value = f.name;
      animatedInput.checked = /\.gif$/i.test(f.name);
    }
  });

  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    const file = fileInput.files?.[0];
    const name = nameInput.value.trim();
    if (!file || !name) return;

    try {
      await uploadFile(
        name,
        file,
        animatedInput.checked,
        widthInput.value ? Number(widthInput.value) : undefined,
        heightInput.value ? Number(heightInput.value) : undefined,
        filterSelect.value || undefined,
      );
      form.reset();
    } catch (err) {
      log(`Upload of "${name}" failed: ${(err as Error).message}`, "err");
    }

    // Refresh regardless of outcome: even a failed upload may be worth
    // re-checking (e.g. partial write), and this guarantees the list never
    // silently goes stale just because the try block above threw.
    await refreshFileList();
  });
}
