#!/usr/bin/env python3
import requests
import traceback
from time import sleep
from typing import List, Callable

SERVER_ROOT = "http://127.0.0.1:8080/"
API_ROOT = SERVER_ROOT + "api/"


def assert_get_info():
    print("Sending GET /info")
    resp = requests.get(API_ROOT + "info")
    print("  Response [{}]: {}".format(resp.status_code, resp.text))
    assert 200 == resp.status_code

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

#####################################################

test_cases : List[Callable]= []

# decorator to define a test case and add to list at same time
def testcase(func):
    test_cases.append(func)
    return func

@testcase
def test_get_info():
    assert_get_info()

@testcase
def test_display_on():
    assert_post_display_on()

@testcase
def test_display_on_off():
    assert_post_display_on()
    sleep(2)
    assert_post_display_off()



print("Starting...")
for test in test_cases:
    try:
        print("")
        print("==================================")
        print("Running test", test.__name__)
        test()
    except requests.exceptions.ConnectionError as e:
        print("Connection failed:", str(e))
    except AssertionError as e:
        traceback.print_exception(e)
        print("Failed assertion. Continuing...")
