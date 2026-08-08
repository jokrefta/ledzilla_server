API version: 0.5.0
date: 2026-08-08

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

## PUT /files/\<name\>
Upload a file. Server processes and stores it synchronously.
Request is `multipart/form-data`. Response may be delayed for large files.
If a file with the given name already exists, it is replaced.

**Fields:**
- `file` (required): the file to upload
- `animated` (required): true or false
- `width` (optional): target width in pixels
- `height` (optional): target height in pixels
  - If only one dimension is given, aspect ratio is maintained.
  - If both are given, image is stretched to fit exactly.
  - If neither is given, the file is stored at its original dimensions.

**Response:**
- `201 Created` (for new image) or `204 No Content` (for updating an existing image) on success
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

Any coordinates are relative to top-left of display. (They are allowed to be outside the range of the display, e.g. if you want part of a shape to be offscreen.)

### Common structures
There are some structures that used in multiple component definitions. These are defined here to avoid
repeating them for each component type.

#### Motion config

```json
"motion_config" : {
  // TBD
}
```

#### Color types

```json
"color" : {
  "type": "hex",
  "hex_code": "#ABC9EA"
}
```

Future addition: animated color type

---

### Text
```json
{
  "type": "text",
  "x": 1,
  "y": 2,
  "content": "Hello World",
  "font": "mono_default_7x13",
  "color": ...,
  "alignment": "Left",
  "motion_config": { ... }
}
```

For a text component, the positioning of the text relative to the provided "x" and "y" coordinates is given by the alignment property.

Alignments may be "Left", "Center", "Right".

A list of valid fonts may be retrieved from the INFO endpoint.

`motion_config` is optional.

---

### Image / GIF
```json
{
  "type": "image",
  "x": 1,
  "y": 2,
  "source": "logo.png",

  "frame_slowdown": 5,
  "motion_config": { ... }

}
```

`frame_slowdown` is optional. It only applies for animated images and is ignored otherwise. If omitted, it defaults to a reasonable value.

A value of 5 means that each GIF frame will last 5 LED refresh frames.
Assumes the GIF has a constant frame delay for all its frames. 
The resulting animation speed will of course depend on the refresh 
rate of the LED display, so the `frame_slowdown` values may need to be determined experimentally. A starting value in the range of 4-10 is suggested.

`motion_config` is optional.

---

### Rectangle
```json
{
  "type": "rect",
  "x": 1,
  "y": 2,
  "width": 10,
  "height": 10,
  "border_color": { ... },
  "border_width": 1,

  "fill_color": { ... } ,
  "motion_config": { ... }
}
```

`fill_color` and `border_color` are color specifications as defined in the  **Common structures** section above.

`x` and `y` are the top-left corner of the rectangle.

`fill_color` and `motion_config` are optional. If no fill color is given, the interior will be transparent (only the border will be drawn).


---

### Line
```json
{
  "type": "line",
  "x1":2,
  "y1": 9,
  "x2": 3,
  "y2": 4,
  "stroke_width": 1,
  "color": { ... },
  "motion_config": { ... }
}
```

`motion_config` is optional.



---
