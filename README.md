HTTP server + API for controlling the raspberry-pi powered HUB75 LED matrix in the office.

API spec - see [led_api_spec.md](led_api_spec.md)

tools/test-client has some examples of using python to interact with it.

#### TODO

- Additional components in spec 
    - plot - display data on a bar/line graph (like stock prices or whatever)
    - maybe other shapes (circles, polygon?)
- Allow changing LED matrix hardware params, to take effect next tiem display is turned on
- API to retrieve refresh rate measurements?

