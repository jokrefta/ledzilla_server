import type {
  FieldDef,
  TextFieldDef,
  NumberFieldDef,
  SelectFieldDef,
  ColorFieldDef,
  Color,
  ColorKeyframe,
  MotionConfig,
} from "./types.js";

/**
 * A rendered field: the DOM element to insert, plus typed get/set access
 * to its current value. `getValue` returns undefined when the field is an
 * optional field that's currently disabled (in which case the key should
 * be omitted from the serialized component entirely).
 */
export interface FieldController<T = unknown> {
  element: HTMLElement;
  getValue: () => T | undefined;
  setValue?: (value: T) => void;
}

// ─────────────────────────────────────────────
//  Small DOM helpers
// ─────────────────────────────────────────────

function labelWrap(text: string, input: HTMLElement): HTMLElement {
  const row = document.createElement("div");
  row.className = "sub-field";
  const label = document.createElement("span");
  label.className = "sub-field-label";
  label.textContent = text;
  row.append(label, input);
  return row;
}

/** Wraps a controller with an "Enable X" checkbox. When unchecked,
 *  getValue() returns undefined so the field is dropped from output. */
function wrapOptional<T>(label: string, inner: FieldController<T>, startEnabled = false): FieldController<T> {
  const wrap = document.createElement("div");
  wrap.className = "optional-field";

  const toggleRow = document.createElement("label");
  toggleRow.className = "optional-toggle";
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.checked = startEnabled;
  toggleRow.append(checkbox, ` Enable ${label}`);

  const body = document.createElement("div");
  body.className = "optional-body";
  body.style.display = startEnabled ? "" : "none";
  body.appendChild(inner.element);

  checkbox.addEventListener("change", () => {
    body.style.display = checkbox.checked ? "" : "none";
  });

  wrap.append(toggleRow, body);

  return {
    element: wrap,
    getValue: () => (checkbox.checked ? inner.getValue() : undefined),
    setValue: (value: T) => {
      const present = value !== undefined && value !== null;
      checkbox.checked = present;
      body.style.display = present ? "" : "none";
      if (present && inner.setValue) inner.setValue(value);
    },
  };
}

// ─────────────────────────────────────────────
//  Simple field types
// ─────────────────────────────────────────────

function renderTextField(field: TextFieldDef): FieldController<string> {
  const input = document.createElement("input");
  input.type = "text";
  input.value = field.default ?? "";
  return {
    element: input,
    getValue: () => input.value,
    setValue: (v) => { input.value = v; },
  };
}

function renderNumberField(field: NumberFieldDef): FieldController<number> {
  const input = document.createElement("input");
  input.type = "number";
  if (field.step !== undefined) input.step = String(field.step);
  input.value = String(field.default ?? 0);
  return {
    element: input,
    getValue: () => Number(input.value),
    setValue: (v) => { input.value = String(v); },
  };
}

function renderSelectField(field: SelectFieldDef): FieldController<string> {
  const select = document.createElement("select");
  const isDynamic = typeof field.options === "function";

  function refresh(preserve?: string) {
    const opts = typeof field.options === "function" ? field.options() : field.options;
    select.innerHTML = "";
    opts.forEach((opt) => {
      const o = document.createElement("option");
      o.value = o.textContent = opt;
      select.appendChild(o);
    });
    const want = preserve ?? field.default;
    if (want && opts.includes(want)) select.value = want;
  }
  refresh();

  // Dynamic option lists (fonts, uploaded files) can change after this
  // field was rendered — re-read them right before the user opens the
  // dropdown so it doesn't go stale, without needing a pub/sub system.
  if (isDynamic) {
    select.addEventListener("focus", () => refresh(select.value));
  }

  return {
    element: select,
    getValue: () => select.value,
    setValue: (v) => { refresh(v); select.value = v; },
  };
}

// ─────────────────────────────────────────────
//  Color field (static or animated)
// ─────────────────────────────────────────────

function renderColorField(field: ColorFieldDef): FieldController<Color> {
  const container = document.createElement("div");
  container.className = "color-field";

  const modeSelect = document.createElement("select");
  (["static", "animated"] as const).forEach((m) => {
    const o = document.createElement("option");
    o.value = o.textContent = m;
    modeSelect.appendChild(o);
  });

  const staticInput = document.createElement("input");
  staticInput.type = "color";
  staticInput.value = field.default ?? "#ffffff";

  const animatedWrap = document.createElement("div");
  animatedWrap.className = "color-animated";
  animatedWrap.style.display = "none";

  const durationInput = document.createElement("input");
  durationInput.type = "number";
  durationInput.min = "1";
  durationInput.value = "30";

  const keyframesList = document.createElement("div");
  keyframesList.className = "keyframe-list";

  function addKeyframeRow(percent: number, color: string) {
    const row = document.createElement("div");
    row.className = "keyframe-row";

    const pctInput = document.createElement("input");
    pctInput.type = "number";
    pctInput.min = "0";
    pctInput.max = "100";
    pctInput.value = String(percent);

    const colorInput = document.createElement("input");
    colorInput.type = "color";
    colorInput.value = color;

    const removeBtn = document.createElement("button");
    removeBtn.type = "button";
    removeBtn.className = "btn btn-remove";
    removeBtn.textContent = "×";
    removeBtn.addEventListener("click", () => row.remove());

    row.append(pctInput, colorInput, removeBtn);
    keyframesList.appendChild(row);
  }
  // Spec requires keyframes at 0 and 100; seed both so a valid payload
  // is possible without the user having to know that rule up front.
  addKeyframeRow(0, field.default ?? "#ffffff");
  addKeyframeRow(100, field.default ?? "#ffffff");

  const addKeyframeBtn = document.createElement("button");
  addKeyframeBtn.type = "button";
  addKeyframeBtn.className = "btn btn-secondary";
  addKeyframeBtn.textContent = "+ Keyframe";
  addKeyframeBtn.addEventListener("click", () => addKeyframeRow(50, "#ffffff"));

  animatedWrap.append(labelWrap("Duration (frames)", durationInput), keyframesList, addKeyframeBtn);

  modeSelect.addEventListener("change", () => {
    staticInput.style.display = modeSelect.value === "static" ? "" : "none";
    animatedWrap.style.display = modeSelect.value === "animated" ? "" : "none";
  });

  container.append(modeSelect, staticInput, animatedWrap);

  function getValue(): Color {
    if (modeSelect.value === "static") {
      return { type: "static", color: staticInput.value };
    }
    const keyframes: ColorKeyframe[] = [...keyframesList.querySelectorAll<HTMLDivElement>(".keyframe-row")].map(
      (row) => {
        const inputs = row.querySelectorAll("input");
        const pct = Number((inputs[0] as HTMLInputElement).value);
        const color = (inputs[1] as HTMLInputElement).value;
        return [pct, color] as ColorKeyframe;
      },
    );
    return { type: "animated", duration: Number(durationInput.value), keyframes };
  }

  function setValue(value: Color) {
    modeSelect.value = value.type;
    modeSelect.dispatchEvent(new Event("change"));
    if (value.type === "static") {
      staticInput.value = value.color;
    } else {
      durationInput.value = String(value.duration);
      keyframesList.innerHTML = "";
      value.keyframes.forEach(([pct, color]) => addKeyframeRow(pct, color));
    }
  }

  return { element: container, getValue, setValue };
}

// ─────────────────────────────────────────────
//  Motion field
// ─────────────────────────────────────────────

function renderMotionField(): FieldController<MotionConfig> {
  const wrap = document.createElement("div");
  wrap.className = "motion-field";

  const dir = document.createElement("input");
  dir.type = "number";
  dir.min = "0";
  dir.max = "360";
  dir.value = "180";

  const dist = document.createElement("input");
  dist.type = "number";
  dist.step = "0.1";
  dist.value = "0.5";

  const period = document.createElement("input");
  period.type = "number";
  period.value = "64";

  wrap.append(
    labelWrap("Direction (deg)", dir),
    labelWrap("Distance/tick", dist),
    labelWrap("Periodicity", period),
  );

  return {
    element: wrap,
    getValue: () => ({
      direction_degrees: Number(dir.value),
      distance_per_tick: Number(dist.value),
      periodicity: Number(period.value),
    }),
    setValue: (v) => {
      dir.value = String(v.direction_degrees);
      dist.value = String(v.distance_per_tick);
      period.value = String(v.periodicity);
    },
  };
}

// ─────────────────────────────────────────────
//  Dispatcher
// ─────────────────────────────────────────────

export function renderField(field: FieldDef): FieldController<any> {
  let controller: FieldController<any>;
  switch (field.type) {
    case "text":
      controller = renderTextField(field);
      break;
    case "number":
      controller = renderNumberField(field);
      break;
    case "select":
      controller = renderSelectField(field);
      break;
    case "color":
      controller = renderColorField(field);
      break;
    case "motion":
      controller = renderMotionField();
      break;
  }
  return field.optional ? wrapOptional(field.label, controller) : controller;
}
