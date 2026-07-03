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

**Request:** multipart/form-data
- `state` (required): JSON containing component list
- `uploads` (optional): one or more uploaded files, referenced by filename in the state JSON

**Response:**
- `204 No Content` on success
- `400 Bad Request` with an error message on failure. State may
  have been cleared - use GET /state to confirm current state.

---

## POST /display/on
Turns the display on, resuming rendering of the current state.

**Response:** 204 No Content

---

## POST /display/off
Turns the display off. Stops the refresh loop. State is preserved.

**Response:** 204 No Content

---

## Component Schema

All components share these base fields:

```json
{
  "type": "<component type>",
  "x": 0,
  "y": 0,
  "scroll": { ... },        // TBD
  "bounce": ...             // TBD
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
  "color": ...              // TBD
}
```

---

### Image
```json
{
  "type": "image",
  "source": "file:<filename>",
  "scale": 1.0
}
```

---

### Video / GIF
```json
{
  "type": "video",
  "source": "file:<filename>",
  "scale": 1.0,
  "speed": 1.0              // multiplier on native frame rate
}
```

---

### Rectangle
```json
{
  "type": "rect",
  "width": 10,
  "height": 10,
  "border_color": ...,      // TBD
  "border_width": 1,
  "fill_color": ...         // TBD
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
  "color": ...              // TBD
}
```

