import type { ComponentTypeDef } from "./types.js";
import { getAvailableFiles } from "./files.js";

// Populated from GET /info once it resolves (see api.ts). The "text" field's
// font <select> reads this array lazily via a closure, so it stays in sync
// without every card needing to be rebuilt.
let availableFonts: string[] = ["mono_default_5x7"];

export function setAvailableFonts(fonts: string[]): void {
  availableFonts = fonts;
}

export const COMPONENT_TYPES: ComponentTypeDef[] = [
  {
    id: "text",
    label: "Text",
    fields: [
      { key: "content", label: "Content", type: "text", default: "Hello" },
      { key: "x", label: "X", type: "number", default: 0 },
      { key: "y", label: "Y", type: "number", default: 0 },
      { key: "font", label: "Font", type: "select", options: () => availableFonts },
      {
        key: "alignment",
        label: "Alignment",
        type: "select",
        options: ["Left", "Center", "Right"],
        default: "Left",
      },
      { key: "color", label: "Color", type: "color", default: "#ffffff" },
      { key: "motion_config", label: "motion", type: "motion", optional: true },
    ],
  },
  {
    id: "image",
    label: "Image",
    fields: [
      { key: "source", label: "Image file", type: "select", options: () => getAvailableFiles() },
      { key: "x", label: "X", type: "number", default: 0 },
      { key: "y", label: "Y", type: "number", default: 0 },
      { key: "frame_slowdown", label: "frame slowdown", type: "number", default: 6, optional: true },
      { key: "motion_config", label: "motion", type: "motion", optional: true },
    ],
  },
  {
    id: "rectangle",
    label: "Rectangle",
    fields: [
      { key: "x", label: "X", type: "number", default: 0 },
      { key: "y", label: "Y", type: "number", default: 0 },
      { key: "width", label: "Width", type: "number", default: 10 },
      { key: "height", label: "Height", type: "number", default: 10 },
      { key: "border_color", label: "Border color", type: "color", default: "#ffffff" },
      { key: "border_width", label: "Border width", type: "number", default: 1 },
      { key: "fill_color", label: "fill color", type: "color", default: "#ff0000", optional: true },
      { key: "motion_config", label: "motion", type: "motion", optional: true },
    ],
  },
  {
    id: "line",
    label: "Line",
    fields: [
      { key: "x1", label: "X1", type: "number", default: 0 },
      { key: "y1", label: "Y1", type: "number", default: 0 },
      { key: "x2", label: "X2", type: "number", default: 10 },
      { key: "y2", label: "Y2", type: "number", default: 10 },
      { key: "stroke_width", label: "Stroke width", type: "number", default: 1 },
      { key: "color", label: "Color", type: "color", default: "#ffffff" },
      { key: "motion_config", label: "motion", type: "motion", optional: true },
    ],
  },
];
