// ─────────────────────────────────────────────
//  Field / component registry types
// ─────────────────────────────────────────────

interface BaseFieldDef {
  key: string;
  label: string;
  /**
   * If true, the field is rendered with an "enable" checkbox and is
   * omitted from the serialized component entirely when unchecked.
   * Use this for anything the API spec marks optional (fill_color,
   * motion_config, frame_slowdown, ...).
   */
  optional?: boolean;
}

export interface TextFieldDef extends BaseFieldDef {
  type: "text";
  default?: string;
}

export interface NumberFieldDef extends BaseFieldDef {
  type: "number";
  default?: number;
  step?: number;
}

export interface SelectFieldDef extends BaseFieldDef {
  type: "select";
  /** Either a fixed list, or a function returning the current list
   *  (used for fonts, which are only known after GET /info resolves). */
  options: string[] | (() => string[]);
  default?: string;
}

/** A static-or-animated color, per the API's color schema. */
export interface ColorFieldDef extends BaseFieldDef {
  type: "color";
  default?: string;
}

/** An optional scroll/motion configuration. */
export interface MotionFieldDef extends BaseFieldDef {
  type: "motion";
}

export type FieldDef =
  | TextFieldDef
  | NumberFieldDef
  | SelectFieldDef
  | ColorFieldDef
  | MotionFieldDef;

export interface ComponentTypeDef {
  /** Machine name — matches the "type" field in state JSON. */
  id: string;
  /** Human-readable name shown in the UI. */
  label: string;
  fields: FieldDef[];
}

// ─────────────────────────────────────────────
//  API JSON shapes
// ─────────────────────────────────────────────

export interface DisplayInfo {
  width: number;
  height: number;
  api_version: string;
  available_fonts: string[];
}

export interface DisplayState {
  components: Array<Record<string, unknown> & { type: string }>;
}

export interface MotionConfig {
  direction_degrees: number;
  distance_per_tick: number;
  periodicity: number;
}

export type ColorKeyframe = [percent: number, color: string];

export interface StaticColor {
  type: "static";
  color: string;
}

export interface AnimatedColor {
  type: "animated";
  duration: number;
  keyframes: ColorKeyframe[];
}

export type Color = StaticColor | AnimatedColor;
