// ─────────────────────────────────────────────
//  TYPES (JSDoc)
// ─────────────────────────────────────────────

/**
 * @typedef {"text"|"number"|"color"|"select"|"textarea"|"file"} FieldType
 */

/**
 * @typedef {Object} FieldDef
 * @property {string}    key      - Input name attribute; should match the API field name
 * @property {string}    label    - Human-readable label shown above the input
 * @property {FieldType} type     - Controls which input element is rendered and how values are coerced
 * @property {string|number} [default] - Initial value; omit for file inputs
 * @property {string[]} [options] - Required when type is "select"
 */

/**
 * @typedef {Object} ComponentTypeDef
 * @property {string}     id     - Machine name used in state JSON
 * @property {string}     label  - Human-readable name shown in the UI
 * @property {FieldDef[]} fields
 */

/**
 * @typedef {Object} ComponentData
 * @property {string} type - Matches ComponentTypeDef.id
 * @property {Object.<string, string|number|File|null>} [*] - Field values keyed by FieldDef.key
 */

/**
 * @typedef {Object} DisplayInfo
 * @property {number} width
 * @property {number} height
 * @property {string} version
 */

/**
 * @typedef {Object} DisplayState
 * @property {Object[]} components
 */

// ─────────────────────────────────────────────
//  COMPONENT TYPE REGISTRY
// ─────────────────────────────────────────────

/** @type {ComponentTypeDef[]} */
const COMPONENT_TYPES = [
  {
    id: "text",
    label: "Text",
    fields: [
      { key: "content", label: "Content", type: "text",   default: "Hello" },
      { key: "x",       label: "X",       type: "number", default: 0 },
      { key: "y",       label: "Y",       type: "number", default: 0 },
      { key: "color",   label: "Color",   type: "color",  default: "#ffffff" },
    ]
  },
  {
    id: "image",
    label: "Image",
    fields: [
      { key: "file", label: "Image file", type: "file" },
      { key: "x",    label: "X",          type: "number", default: 0 },
      { key: "y",    label: "Y",          type: "number", default: 0 },
    ]
  },
  {
    id: "rect",
    label: "Rectangle",
    fields: [
      { key: "x",     label: "X",      type: "number", default: 0 },
      { key: "y",     label: "Y",      type: "number", default: 0 },
      { key: "w",     label: "Width",  type: "number", default: 10 },
      { key: "h",     label: "Height", type: "number", default: 10 },
      { key: "color", label: "Color",  type: "color",  default: "#ff0000" },
    ]
  },
  // TODO: add polygon, line, etc. as the spec evolves
];

// ─────────────────────────────────────────────
//  COERCE
// ─────────────────────────────────────────────

/**
 * Convert a raw FormData string value to the appropriate JS type.
 * @param {string} value
 * @param {FieldType} fieldType
 * @returns {string|number}
 */
function coerce(value, fieldType) {
  switch (fieldType) {
    case "number": return Number(value);
    default:       return value;  // text, color, select, textarea stay as strings
  }
  // Note: "file" fields are not in FormData values; handled separately in readCard()
}

// ─────────────────────────────────────────────
//  READ CARD
// ─────────────────────────────────────────────

/**
 * Collect and coerce all field values from a component card element.
 * @param {HTMLElement} cardEl
 * @returns {ComponentData}
 */
function readCard(cardEl) {
  const type = cardEl.dataset.type;
  const typeDef = COMPONENT_TYPES.find(t => t.id === type);
  const form = cardEl.querySelector("form");
  const raw = Object.fromEntries(new FormData(form));

  /** @type {ComponentData} */
  const data = { type };
  typeDef.fields.forEach(field => {
    if (field.type === "file") {
      const input = /** @type {HTMLInputElement} */ (form.querySelector(`[name="${field.key}"]`));
      data[field.key] = input.files[0] ?? null;
    } else {
      data[field.key] = coerce(raw[field.key], field.type);
    }
  });
  return data;
}

// ─────────────────────────────────────────────
//  API
// ─────────────────────────────────────────────

const api = (() => {
  const BASE = "/api";  // same origin — no IP needed

  /**
   * @param {string} method
   * @param {string} path
   * @param {FormData|Object} [body]
   * @param {boolean} [isFormData]
   * @returns {Promise<any|null>}  null on error or 204
   */
  async function call(method, path, body, isFormData) {
    const opts = { method };
    if (body) {
      if (isFormData) {
        opts.body = body;
      } else {
        opts.headers = { "Content-Type": "application/json" };
        opts.body = JSON.stringify(body);
      }
    }

    log(`${method} ${path}`, "info");
    const res = await fetch(BASE + path, opts).catch(err => {
      log(`Network error: ${err.message}`, "err");
      return null;
    });

    if (!res) return null;
    if (res.status === 204) { log("204 No Content", "ok"); return null; }

    const text = await res.text();
    if (!res.ok) { log(`${res.status} ${text}`, "err"); return null; }

    log(`${res.status} OK`, "ok");
    return text ? JSON.parse(text) : null;
  }

  return {
    async probe() {
      /** @type {DisplayInfo|null} */
      const info = await call("GET", "/info");
      if (info) {
        document.getElementById("display-info").innerHTML =
          `${info.width} × ${info.height} px<br>firmware ${info.version}`;
      }
    },

    async getState() {
      /** @type {DisplayState|null} */
      const state = await call("GET", "/state");
      if (state) {
        log(`Got ${state.components?.length ?? 0} components`, "info");
        // TODO: populate component list from pulled state
        console.log("Pulled state:", state);
      }
    },

    async pushState() {
      const cards = /** @type {HTMLElement[]} */ ([...document.querySelectorAll(".component-card")]);
      const components = cards.map(readCard);

      const stateObj = {
        components: components.map(c => {
          // Replace File objects with filenames for JSON serialisation
          const out = { ...c };
          Object.keys(out).forEach(k => {
            if (out[k] instanceof File) out[k] = out[k].name;
          });
          return out;
        })
      };

      const form = new FormData();
      form.append("state", JSON.stringify(stateObj));

      // Attach any uploaded files as separate multipart fields
      components.forEach(c => {
        Object.values(c).forEach(v => {
          if (v instanceof File) form.append("uploads", v, v.name);
        });
      });

      await call("POST", "/state", form, true);
    },

    async displayOn()  { await call("POST", "/display/on"); },
    async displayOff() { await call("POST", "/display/off"); },
  };
})();

// ─────────────────────────────────────────────
//  LOG
// ─────────────────────────────────────────────

/**
 * @param {string} msg
 * @param {"ok"|"err"|"info"|""} [kind]
 */
function log(msg, kind = "") {
  const el = document.getElementById("log");
  const now = new Date().toLocaleTimeString("en-GB", { hour12: false });
  const row = document.createElement("div");
  row.className = "log-entry";
  row.innerHTML = `<span class="log-time">${now}</span><span class="log-${kind}">${msg}</span>`;
  el.appendChild(row);
  el.scrollTop = el.scrollHeight;
}

// ─────────────────────────────────────────────
//  COMPONENT UI
// ─────────────────────────────────────────────

/**
 * Build a component card DOM element for the given type.
 * @param {string} typeId
 * @param {boolean} [startOpen=true]
 * @returns {HTMLElement}
 */
function createCard(typeId, startOpen = true) {
  const typeDef = COMPONENT_TYPES.find(t => t.id === typeId);
  const count = document.querySelectorAll(`.component-card[data-type="${typeId}"]`).length + 1;

  const card = document.createElement("div");
  card.className = "component-card" + (startOpen ? " open" : "");
  card.dataset.type = typeId;

  // Header — clicking anywhere on it toggles open/closed
  const header = document.createElement("div");
  header.className = "card-header";
  header.innerHTML = `
    <span class="component-type-badge">${typeId}</span>
    <span class="component-name">${typeDef.label} ${count}</span>
    <span class="chevron">▶</span>`;
  header.addEventListener("click", () => card.classList.toggle("open"));

  // Body — hidden until card is open
  const body = document.createElement("div");
  body.className = "card-body";

  const form = document.createElement("form");
  form.addEventListener("submit", e => e.preventDefault());

  typeDef.fields.forEach(field => {
    const wrap = document.createElement("div");
    wrap.className = "field";

    const label = document.createElement("label");
    label.textContent = field.label;

    let input;
    if (field.type === "select") {
      input = document.createElement("select");
      input.name = field.key;
      field.options.forEach(opt => {
        const o = document.createElement("option");
        o.value = o.textContent = opt;
        input.appendChild(o);
      });
      if (field.default !== undefined) input.value = field.default;
    } else if (field.type === "textarea") {
      input = document.createElement("textarea");
      input.name = field.key;
      input.value = field.default ?? "";
    } else {
      input = document.createElement("input");
      input.type = field.type;
      input.name = field.key;
      if (field.type !== "file" && field.default !== undefined) {
        input.value = field.default;
      }
    }

    wrap.appendChild(label);
    wrap.appendChild(input);
    form.appendChild(wrap);
  });

  const removeBtn = document.createElement("button");
  removeBtn.className = "btn btn-remove";
  removeBtn.textContent = "Remove";
  removeBtn.addEventListener("click", () => {
    card.remove();
    updateEmptyState();
  });

  body.appendChild(form);
  body.appendChild(removeBtn);
  card.appendChild(header);
  card.appendChild(body);
  return card;
}

/**
 * Add a new component card of the given type to the list.
 * @param {string} typeId
 */
function addComponent(typeId) {
  document.getElementById("component-list").appendChild(createCard(typeId));
  updateEmptyState();
}

/** Show or hide the empty state message based on whether any cards exist. */
function updateEmptyState() {
  const hasCards = document.querySelector(".component-card") !== null;
  document.getElementById("empty-state").style.display = hasCards ? "none" : "flex";
}

// ─────────────────────────────────────────────
//  INIT
// ─────────────────────────────────────────────
(function init() {
  const picker = document.getElementById("add-component");
  COMPONENT_TYPES.forEach(t => {
    const btn = document.createElement("button");
    btn.className = "type-option";
    btn.textContent = `+ ${t.label}`;
    btn.addEventListener("click", () => addComponent(t.id));
    picker.appendChild(btn);
  });

  updateEmptyState();
  log("Ready", "info");
  api.probe();
})();
