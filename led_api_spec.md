API version: 0.2.0
date: 2026-07-29

# LEDzilla API

Base URL: `http://<pi-address>/api`

---

## GET /info
Returns display capabilities.

**Response:**
```json
{
  "width": 64,
  "height": 32,
  "api_version": "0.2.0",
  "available_fonts": ["mono_default_4x6", "mono_default_5x7", ...],
}
```

---

## GET /state
Returns the current display state, reflecting exactly what is on the display.

**Response:**
```json
{
  "components": [ ... ]
}
```

---

## POST /state
Replace the current display state. Implicitly turns the display on.
Request body is `application/json`.

**Response:**
- `204 No Content` on success
- `400 Bad Request` with an error message on failure. State may
  have been cleared — use `GET /state` to confirm current state.

---

## POST /display/on
Turns the display on, resuming rendering of the current state.

**Response:** `204 No Content`

---

## POST /display/off
Turns the display off. Stops the refresh loop. State is preserved.

**Response:** `204 No Content`

---

## POST /files
Upload a file. Server processes and stores it synchronously.
Request is `multipart/form-data`. Response may be delayed for large files.

**Fields:**
- `file` (required): the file to upload
- `name` (required): the name to store the file under
- `width` (optional): target width in pixels
- `height` (optional): target height in pixels
- If only one dimension is given, aspect ratio is maintained.
- If both are given, image is stretched to fit exactly.
- If neither is given, the file is stored at its original dimensions.

**Response:**
- `201 Created` with header `Location: /api/files/<name>`
  and body:
  ```json
  { "name": "<name>" }
  ```
- `400 Bad Request` if file is invalid or unsupported format

---

## GET /files
List uploaded files.

**Response:**
```json
{
  "files": ["rickastley.gif", "logo.png"]
}
```

---

## DELETE /files/\<name\>
Delete an uploaded file.

**Response:**
- `204 No Content` on success
- `404 Not Found` if file does not exist

---

## Component Schema

All components have a `"type"` field declaring the component type.

### Common properties
All components share a `"common_properties"` object with these fields:

```json
"common_properties": {
  "x": 0,
  "y": 0,
  "scroll": { ... },
}
```

"scroll" is optional and currently not supported.

For most component types, the "x" and "y" values represent the position of the top-left corner of the component.

---

### Text
```json
{
  "type": "text",
  "common_properties": {...},
  "content": "Hello World",
  "font": "mono_default_7x13",
  "color": ...,
  "alignment": "Left",
}
```

For a text component, the positioning of the text relative to the provided "x" and "y" coordinates is given by the alignment property.

Alignments may be "Left", "Center", "Right".

A list of valid fonts may be retrieved from the INFO endpoint.

---

### Image
```json
{
  "type": "image",
  "common_properties": {...},
  "source": "logo.png"
}
```

---

### Video / GIF
```json
{
  "type": "video",
  "common_properties": {...},
  "source": "rickastley.gif"
}
```

---

### Rectangle
```json
{
  "type": "rect",
  "common_properties": {...},
  "width": 10,
  "height": 10,
  "border_color": ...,
  "border_width": 1,
  "fill_color": ...
}
```

---

### Line
```json
{
  "type": "line",
  "common_properties": {...},
  "delta_x": 1,
  "delta_y": -5,
  "stroke_width": 1,
  "color": ...
}
```

For a line, the "common_properties" x and y values define one endpoint, and the "delta_x"/"delta_y" values define the 
offset of the other endpoint relative to the first.

---

*Fields marked `...` are TBD: color and scroll are not yet fully specified.*
