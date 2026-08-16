API version: 0.6.4
date: 2026-08-16

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
  "api_version": "0.6.3",
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
- `resize_filter` (optional): interpolation filter for resizing.
  - "nearest_neighbor"
  - "bilinear" (default)
  - "bicubic"

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
  "direction_degrees": 180,
  "distance_per_tick": 0.5,
  "periodicity": 256
}
```

Allows a component to scroll in one direction, repeating every `periodicity` pixels. `direction_degrees` is any integer from 0
and 360. `distance_per_tick` is in pixels.

For example, an angle of 180 causes the object to scroll toward the left. If `periodicity` equals display width, then the object 
will start to wrap around exactly when it reaches the left edge of the screen. 
If `periodicity` is greater than display width, there will be a delay between reaching the left edge and starting to appear on the right.
If `periodicity` is less than display width, multiple copies of the component will be visible on the screen at one time.


#### Color types

##### Static
```json
"color" : {
  "type": "static",
  "color": "#ABC9EA"
}
```

`color` is a string in CSS color format. (Named colors are not currently supported).

##### Animated

```json
"color" : {
    "type": "animated",
    "duration": 30,
    "keyframes": [
        [0, "#FF0000"],
        [20, "rgb(0, 120, 0)"],
        [90, "#00F"],
        [100, "#FF0000"]
    ]
}
```

Color smoothly transitions in a looping animation.

Note the various valid CSS formats that are accepted.

Keyframes are in percent - a value outside [0, 100] will be rejected.

The above example 
- starts at red but smoothly transitions to dark green
- Hits dark green when 20% of the way through the animation
- Reaches blue when 90% of the way through
- Quickly shifts back to red during the last 10%, so the loop looks clean.

Both 0 and 100 are **required** to be present.

The meaning of `duration` is TBD, but probably will be number of total frames for the animation loop. Note that all steps in
the gradient will be computed and stored in memory, so don't set this to some insane value in the millions.

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
  "type": "rectangle",
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
