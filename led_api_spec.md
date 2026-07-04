date: 2026-07-03

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
  "version": "1.0.0"
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

All components share these base fields:

```json
{
  "type": "<component type>",
  "x": 0,
  "y": 0,
  "scroll": { ... },
  "bounce": ...
}
```

---

### Text
```json
{
  "type": "text",
  "content": "Hello World",
  "font": {
    "face": "Arial",
    "size": 12
  },
  "color": ...
}
```

---

### Image
```json
{
  "type": "image",
  "source": "logo.png"
}
```

---

### Video / GIF
```json
{
  "type": "video",
  "source": "rickastley.gif"
}
```

---

### Rectangle
```json
{
  "type": "rect",
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
  "x1": 0,
  "y1": 0,
  "x2": 10,
  "y2": 10,
  "width": 1,
  "color": ...
}
```

---

*Fields marked `...` are TBD: color, scroll, and bounce are not yet fully specified.*
