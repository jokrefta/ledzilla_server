import { COMPONENT_TYPES } from "./components.js";
import { addComponent, updateEmptyState } from "./card.js";
import { api } from "./api.js";
import { log } from "./log.js";
import { initFileManager, refreshFileList } from "./files.js";

function init(): void {
  const picker = document.getElementById("add-component")!;
  COMPONENT_TYPES.forEach((t) => {
    const btn = document.createElement("button");
    btn.className = "type-option";
    btn.textContent = `+ ${t.label}`;
    btn.addEventListener("click", () => addComponent(t.id));
    picker.appendChild(btn);
  });

  document.getElementById("btn-display-on")!.addEventListener("click", () => api.displayOn());
  document.getElementById("btn-display-off")!.addEventListener("click", () => api.displayOff());
  document.getElementById("btn-push-state")!.addEventListener("click", () => api.pushState());
  document.getElementById("btn-pull-state")!.addEventListener("click", () => api.getState());

  initFileManager();

  updateEmptyState();
  log("Ready", "info");
  api.probe();
  api.refreshDisplayPower();
  refreshFileList();
}

init();
