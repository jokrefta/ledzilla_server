HTTP server + API for controlling the raspberry-pi powered HUB75 LED matrix in the office.

API spec - see [led_api_spec.md](led_api_spec.md)

tools/test-client has some examplesof using python to interact with it.

#### TODO

- Additional components in spec 
    - plot - display data on a bar/line graph (like stock prices or whatever)
    - maybe other shapes (circles, polygon?)
- Add support for special patterns in text (e.g, "{TIME}" to display current time - allows user to make a clock or 
  something without needing HTTP request every second)
- web GUI improvements
  - BUG- seems you have to upload a file twice (or reload) for it to be available in GUI

