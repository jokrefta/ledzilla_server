import { COMPONENT_TYPES } from "./components.js";
import { renderField, type FieldController } from "./fields.js";
import type { FieldDef, FieldRow } from "./types.js";
import { log } from "./log.js";

// Each card's field controllers, keyed by field key. Kept out-of-band in a
// WeakMap rather than on the element itself, so the DOM stays plain.
const cardControllers = new WeakMap<HTMLElement, Map<string, FieldController>>();

/** Flattens a component type's field rows (which may pair two fields
 *  together for side-by-side layout) back into a plain list of fields.
 *  Used anywhere layout doesn't matter and every field just needs visiting. */
function flattenFields(rows: FieldRow[]): FieldDef[] {
  return rows.flatMap((row) => (Array.isArray(row) ? row : [row]));
}

/** Renders a single field (label + input) as one ".field" element and
 *  registers its controller. Shared by both single and paired rows. */
function renderFieldElement(field: FieldDef, controllers: Map<string, FieldController>): HTMLElement {
  const controller = renderField(field);
  controllers.set(field.key, controller);

  const el = document.createElement("div");
  el.className = "field";
  const label = document.createElement("label");
  label.textContent = field.label;
  el.append(label, controller.element);
  return el;
}

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
    <span class="component-name">${typeDef.label} ${count}</span>`;
  header.addEventListener("click", () => card.classList.toggle("open"));

  // Component order in the list determines draw order in the pushed state
  // (pushState() serializes cards in DOM order), so reordering here is
  // just moving DOM nodes around — no changes needed to serialization.
  const headerActions = document.createElement("div");
  headerActions.className = "card-header-actions";

  const moveUpBtn = document.createElement("button");
  moveUpBtn.className = "btn btn-icon";
  moveUpBtn.textContent = "▲";
  moveUpBtn.title = "Move up";
  moveUpBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    // No-op (via optional chaining) when already first in the list.
    card.previousElementSibling?.before(card);
  });

  const moveDownBtn = document.createElement("button");
  moveDownBtn.className = "btn btn-icon";
  moveDownBtn.textContent = "▼";
  moveDownBtn.title = "Move down";
  moveDownBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    // No-op (via optional chaining) when already last in the list.
    card.nextElementSibling?.after(card);
  });

  const removeBtn = document.createElement("button");
  removeBtn.className = "btn btn-icon btn-remove";
  removeBtn.textContent = "✕";
  removeBtn.title = "Remove";
  removeBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    card.remove();
    updateEmptyState();
  });

  const chevron = document.createElement("span");
  chevron.className = "chevron";
  chevron.textContent = "▶";

  headerActions.append(moveUpBtn, moveDownBtn, removeBtn, chevron);
  header.appendChild(headerActions);

  const body = document.createElement("div");
  body.className = "card-body";

  const controllers = new Map<string, FieldController>();
  typeDef.fields.forEach((row) => {
    const rowEl = document.createElement("div");
    rowEl.className = "field-row";
    if (Array.isArray(row)) {
      rowEl.classList.add("field-row-paired");
      row.forEach((field) => rowEl.appendChild(renderFieldElement(field, controllers)));
    } else {
      rowEl.appendChild(renderFieldElement(row, controllers));
    }
    body.appendChild(rowEl);
  });
  cardControllers.set(card, controllers);

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
 *  Unknown component types are skipped (logged to the activity log) rather
 *  than failing the whole load, since the spec is still evolving. */
export function loadComponentsFromState(components: Array<Record<string, unknown>>): void {
  clearAllCards();
  const list = document.getElementById("component-list")!;

  components.forEach((comp) => {
    const typeDef = COMPONENT_TYPES.find((t) => t.id === comp.type);
    if (!typeDef) {
      log(`Skipping unknown component type "${comp.type}" from pulled state`, "err");
      return;
    }
    const card = createCard(typeDef.id, false);
    list.appendChild(card);

    const controllers = cardControllers.get(card)!;
    flattenFields(typeDef.fields).forEach((field) => {
      const controller = controllers.get(field.key);
      const value = comp[field.key];
      if (controller?.setValue && value !== undefined) {
        controller.setValue(value as never);
      }
    });
  });

  updateEmptyState();
}
