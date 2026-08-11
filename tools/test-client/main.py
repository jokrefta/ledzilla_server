#!/usr/bin/env python3
"""
Some tests for the ledzilla server. Not 100% automated - some manual inspection is
required to ensure proper output (especially for the display)
"""


import requests
import traceback
from time import sleep
from typing import List, Callable
from pathlib import Path
import termcolor

SERVER_ROOT = "http://127.0.0.1:8080/"
API_ROOT = SERVER_ROOT + "api/"

CURRENT_DIR = Path(__file__).parent.resolve()
ROOT_DIR = CURRENT_DIR.parent.parent.resolve()

def get_empty_state_json():
    return {"components": []}

def assert_get_info():
    print("Sending GET /info")
    resp = requests.get(API_ROOT + "info")
    print("  Response [{}]: {}".format(resp.status_code, resp.text))
    assert 200 == resp.status_code

# requires manual inspection to ensure the components match what is expected
def assert_get_state():
    print("Sending GET /state")
    resp = requests.get(API_ROOT + "state")
    print("  Response [{}]: {}".format(resp.status_code, resp.text))
    assert 200 == resp.status_code
    assert '"components"' in resp.text

def assert_post_display_on():
    print("Sending POST /display/on")
    resp = requests.post(API_ROOT + "display/on")
    print("  Response [{}]: {}".format(resp.status_code, resp.text))
    assert 204 == resp.status_code

def assert_post_display_off():
    print("Sending POST /display/off")
    resp = requests.post(API_ROOT + "display/off")
    print("  Response [{}]: {}".format(resp.status_code, resp.text))
    assert 204 == resp.status_code

def assert_flash_display(seconds):
    assert_post_display_on()
    sleep(seconds)
    assert_post_display_off()


def assert_post_state(state_json, is_valid=True):
    print("Sending POST /state")
    resp = requests.post(API_ROOT + "state", json=state_json)
    print("  Response [{}]: {}".format(resp.status_code, resp.text))
    if is_valid:
        assert 204 == resp.status_code
    else:
        assert 400 == resp.status_code

def assert_reset_state():
    # reset as much as possible for new test case
    assert_post_display_off()
    assert_post_state(get_empty_state_json())
    for filename in assert_list_files():
        assert_delete_file(filename)

def assert_list_files() -> List:
    print("Sending GET /files")
    resp = requests.get(API_ROOT + "files")
    print("  Response [{}]: {}".format(resp.status_code, resp.text))
    assert 200 == resp.status_code
    return resp.json()["files"]

def assert_put_file(target_file_name, file_data, is_animated, is_new_file, width=None, height=None):
    print("Sending PUT /files/" + target_file_name)
    resp = requests.put(
        API_ROOT + "files/" + target_file_name,
        data={
            "width": width,
            "height": height,
            "animated": "true" if is_animated else "false"
        },
        files={"file": file_data}
    )
    print("  Response [{}]: {}".format(resp.status_code, resp.text))
    if is_new_file:
        assert 201 == resp.status_code
    else:
        assert 204 == resp.status_code

def assert_delete_file(target_file_name):
    print("Sending DELETE /files/" + target_file_name)
    resp = requests.delete(API_ROOT + "files/" + target_file_name)
    print("  Response [{}]: {}".format(resp.status_code, resp.text))
    assert 204 == resp.status_code

#####################################################

test_cases : List[Callable]= []

# decorator to define a test case and add to list at same time
def testcase(func):
    test_cases.append(func)
    return func

@testcase
def test_get_info():
    assert_get_info()
'''
@testcase
def test_display_on_off():
    assert_reset_state()
    assert_flash_display(2)

@testcase
def test_get_state():
    assert_reset_state()
    assert_get_state()

@testcase
def test_post_state_empty():
    assert_post_state(get_empty_state_json())

@testcase
def test_post_state_line_and_get_state():
    assert_reset_state()
    json_state = {"components": [
        {
            "type": "line",
            "x1":  10,
            "y1": 10,
            "x2": 11,
            "y2": 5,
            "stroke_width": 1,
            "color": {"type": "static", "color": "#0022FF"}
        }
    ]}
    assert_post_state(json_state)
    assert_get_state()

@testcase
def test_post_state_line_and_clear_state_and_get_state():
    assert_reset_state()
    json_state = {"components": [
        {
            "type": "line",
            "x1":  10,
            "y1": 10,
            "x2": 11,
            "y2": 5,
            "stroke_width": 1,
            "color": {"type": "static", "color": "#0022FF"}
        }
    ]}
    assert_post_state(json_state)
    assert_post_state(get_empty_state_json())
    assert_get_state()

@testcase
def test_post_state_line_and_render():
    assert_reset_state()
    json_state = {"components": [
        {
            "type": "line",
            "x1":  10,
            "y1": 10,
            "x2": 11,
            "y2": 5,
            "stroke_width": 1,
            "color": {"type": "static", "color": "#0022FF"}
        }
    ]}
    assert_post_state(json_state)
    assert_flash_display(2)

@testcase
def test_post_state_line_animated_color_and_render():
    assert_reset_state()
    json_state = {"components": [
        {
            "type": "line",
            "x1":  10,
            "y1": 30,
            "x2": 200,
            "y2": 35,
            "stroke_width": 20,
            "color": {
                "type": "animated",
                "duration": 60,
                "keyframes": [
                    [0, "#FF0000"],
                    [20, "rgb(0, 120, 0)"],
                    [90, "#00F"],
                    [100, "#FF0000"]
                ]
            }
        }
    ]}
    assert_post_state(json_state)
    assert_flash_display(4)


@testcase
def test_post_state_multiple_lines():
    assert_reset_state()
    json_state = {"components": [
        {
            "type": "line",
            "x1":  30,
            "y1": 10,
            "x2": 110,
            "y2": 10,
            "stroke_width": 5,
            "color": {"type": "static", "color": "#0022FF"}
        },
        {
            "type": "line",
            "x1":  20,
            "y1": 15,
            "x2": 100,
            "y2": 15,
            "stroke_width": 5,
            "color": {"type": "static", "color": "#FF0022"}
        },
        {
            "type": "line",
            "x1":  10,
            "y1": 20,
            "x2": 90,
            "y2": 20,
            "stroke_width": 5,
            "color": {"type": "static", "color": "#22FF00"}
        }
    ]}
    assert_post_state(json_state)
    assert_flash_display(2)

@testcase
def test_post_state_multiple_lines_with_text():
    assert_reset_state()
    json_state = {"components": [
        {
            "type": "line",
            "x1":  30,
            "y1": 10,
            "x2": 110,
            "y2": 10,
            "stroke_width": 5,
            "color": {"type": "static", "color": "#0022FF"}
        },
        {
            "type": "line",
            "x1":  20,
            "y1": 15,
            "x2": 100,
            "y2": 15,
            "stroke_width": 5,
            "color": {"type": "static", "color": "#FF0022"}
        },
        {
            "type": "line",
            "x1":  10,
            "y1": 20,
            "x2": 90,
            "y2": 20,
            "stroke_width": 5,
            "color": {"type": "static", "color": "#22FF00"}
        },
        {
            "type": "text",
            "x":  100,
            "y": 40,
            "content": "Hello world",
            "font": "mono_default_7x13_italic",
            "color": {"type": "static", "color": "#44AA44"},
            "alignment": "Center"
        }
    ]}
    assert_post_state(json_state)
    assert_flash_display(2)

def test_upload_and_delete_file():
    with open(ROOT_DIR / "assets" / "test" / "vertical_gradient.png", "rb") as f:
        assert_put_file("smol_upload.png", ("dont_care_filename", f, "image/png"), False, True)
    assert_delete_file("smol_upload.png")

def test_upload_files_and_get_files():
    assert_reset_state()
    with open(ROOT_DIR / "assets" / "test" / "vertical_gradient.png", "rb") as f:
        assert_put_file("smol_upload.png", ("dont_care_filename", f, "image/png"), False, True)
        f.seek(0)
        assert_put_file("smol_upload2.png", ("dont_care_filename", f, "image/png"), False, True)
    assert 2 == len(assert_list_files())

@testcase
def test_upload_files_and_modify_files():
    assert_reset_state()
    # Just check that the response code is correct for modification. Will need a test later on to
    # verify visually that the image updates upon reconfiguring the component state
    with open(ROOT_DIR / "assets" / "test" / "vertical_gradient.png", "rb") as f:
        assert_put_file("smol_upload.png", ("dont_care_filename", f, "image/png"), False, True)
        f.seek(0)
        assert_put_file("smol_upload.png", ("dont_care_filename", f, "image/png"), False, False)
'''

@testcase
def test_draw_image():
    assert_reset_state()

    with open(ROOT_DIR / "assets" / "test" / "vertical_gradient.png", "rb") as f:
        assert_put_file("smol_upload.png", ("dont_care_filename", f, "image/png"), False, True)

    json_state = {"components": [
        {
            "type": "image",
            "x":  30,
            "y": 10,
            "source": "smol_upload.png"
        }
    ]}
    assert_post_state(json_state)
    assert_flash_display(2)

@testcase
def test_draw_image_nonexistent_file():
    assert_reset_state()
    json_state = {"components": [
        {
            "type": "image",
            "x":  30,
            "y": 10,
            "source": "nonexistent.png"
        }
    ]}
    assert_post_state(json_state, False)
    assert_flash_display(1)

@testcase
def test_draw_animated():
    assert_reset_state()

    with open(ROOT_DIR / "assets" / "test" / "gradient.gif", "rb") as f:
        assert_put_file("animated.gif", ("dont_care_filename", f, "image/gif"), True, True)

    json_state = {"components": [
        {
            "type": "image",
            "x":  30,
            "y": 10,
            "source": "animated.gif",
            "frame_slowdown": 8
        }
    ]}
    assert_post_state(json_state)
    assert_flash_display(2.5)

@testcase
def test_draw_image_resize_maintain_aspect_ratio():
    assert_reset_state()

    with open(ROOT_DIR / "assets" / "test" / "vertical_gradient.png", "rb") as f:
        assert_put_file("smol_upload.png", ("dont_care_filename", f, "image/png"), False, True, None, 7)

    json_state = {"components": [
        {
            "type": "image",
            "x":  30,
            "y": 10,
            "source": "smol_upload.png"
        }
    ]}
    assert_post_state(json_state)
    assert_flash_display(2)

@testcase
def test_draw_animated_resize_squish():
    assert_reset_state()

    with open(ROOT_DIR / "assets" / "test" / "gradient.gif", "rb") as f:
        assert_put_file("animated.gif", ("dont_care_filename", f, "image/gif"), True, True, 7, 10)

    json_state = {"components": [
        {
            "type": "image",
            "x":  30,
            "y": 10,
            "source": "animated.gif",
            "frame_slowdown": 8
        }
    ]}
    assert_post_state(json_state)
    assert_flash_display(2.5)

@testcase
def test_post_state_rect_and_render():
    assert_reset_state()
    json_state = {"components": [
        {

          "type": "rectangle",
          "x": 30,
          "y": 20,
          "width": 20,
          "height": 10,
          "border_color": {
              "type": "animated",
              "duration": 60,
              "keyframes": [
                  [0, "#FF0000"],
                  [20, "0000FF"],
                  [90, "#00FFFF"],
                  [100, "#FF0000"]
              ]
          },
          "border_width": 3,
          "fill_color": {
              "type": "static",
              "color": "ffa09e"
          }
        }
    ]}
    assert_post_state(json_state)
    assert_flash_display(2)


print("Starting...")
failures = 0
for test in test_cases:
    try:
        print("")
        print("==================================")
        print("Running test", test.__name__)
        test()
    except requests.exceptions.ConnectionError as e:
        termcolor.cprint("Connection failed: " + str(e), "red")
        break
    except AssertionError as e:
        traceback.print_exception(e)
        termcolor.cprint("Failed assertion. Continuing...", "red")
        failures += 1

print("Total failures", failures)
