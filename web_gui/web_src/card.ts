import { COMPONENT_TYPES } from "./components.js";
import { renderField, type FieldController } from "./fields.js";

// Each card's field controllers, keyed by field key. Kept out-of-band in a
// WeakMap rather than on the element itself, so the DOM stays plain.
const cardControllers = new WeakMap<HTMLElement, Map<string, FieldController>>();

export function createCard(typeId: string, startOpen = true): HTMLElement {
  const typeDef = COMPONENT_TYPES.find((t) => t.id === typeId);
  if (!typeDef) throw new Error(`Unknown component type: ${typeId}`);

  const count = document.querySelectorAll(`.component-card[data-type="${typeId}"]`).length + 1;

  const card = document.createElement("div");
  card.className = "component-card" + (startOpen ? " open" : "");
  card.dataset.type = typeId;

  const header = document.createElement("div");
  header.className = "card-header";
  header.innerHTML = `
    <span class="component-type-badge">${typeId}</span>
    <span class="component-name">${typeDef.label} ${count}</span>
    <span class="chevron">▶</span>`;
  header.addEventListener("click", () => card.classList.toggle("open"));

  const body = document.createElement("div");
  body.className = "card-body";

  const controllers = new Map<string, FieldController>();
  typeDef.fields.forEach((field) => {
    const controller = renderField(field);
    controllers.set(field.key, controller);

    const row = document.createElement("div");
    row.className = "field";
    const label = document.createElement("label");
    label.textContent = field.label;
    row.append(label, controller.element);
    body.appendChild(row);
  });
  cardControllers.set(card, controllers);

  const removeBtn = document.createElement("button");
  removeBtn.className = "btn btn-remove";
  removeBtn.textContent = "Remove";
  removeBtn.addEventListener("click", () => {
    card.remove();
    updateEmptyState();
  });
  body.appendChild(removeBtn);

  card.append(header, body);
  return card;
}

/** Reads a card's current field values, keyed by field key. Optional
 *  fields that are disabled are simply absent from the returned object. */
export function readCard(card: HTMLElement): { type: string; fields: Record<string, unknown> } {
  const controllers = cardControllers.get(card);
  const fields: Record<string, unknown> = {};
  controllers?.forEach((controller, key) => {
    const value = controller.getValue();
    if (value !== undefined) fields[key] = value;
  });
  return { type: card.dataset.type!, fields };
}

export function addComponent(typeId: string): void {
  document.getElementById("component-list")!.appendChild(createCard(typeId));
  updateEmptyState();
}

export function updateEmptyState(): void {
  const hasCards = document.querySelector(".component-card") !== null;
  document.getElementById("empty-state")!.style.display = hasCards ? "none" : "flex";
}

export function clearAllCards(): void {
  document.getElementById("component-list")!.innerHTML = "";
  updateEmptyState();
}

/** Rebuilds the component list from a pulled /state response.
 *  Unknown component types are skipped (logged to the console) rather
 *  than failing the whole load, since the spec is still evolving. */
export function loadComponentsFromState(components: Array<Record<string, unknown>>): void {
  clearAllCards();
  const list = document.getElementById("component-list")!;

  components.forEach((comp) => {
    const typeDef = COMPONENT_TYPES.find((t) => t.id === comp.type);
    if (!typeDef) {
      console.warn("Skipping unknown component type from pulled state:", comp.type);
      return;
    }
    const card = createCard(typeDef.id, false);
    list.appendChild(card);

    const controllers = cardControllers.get(card)!;
    typeDef.fields.forEach((field) => {
      const controller = controllers.get(field.key);
      const value = comp[field.key];
      if (controller?.setValue && value !== undefined) {
        controller.setValue(value as never);
      }
    });
  });

  updateEmptyState();
}
