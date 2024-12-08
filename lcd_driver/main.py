from RPLCD.i2c import CharLCD 
import socket
import os
import threading
import time
from dataclasses import dataclass
from typing import List, Optional
import json
from functools import wraps
socket_path = __file__.replace("main.py", "lcd.sock")

@dataclass
class SocketLayout:
    cmd: str
    args: Optional[any] = None

"""
cmd: 
buffer - text, directly, addative 
move - x, y
backlight - state
cursor_mode - mode
shift_display - amount
home
clear 
write
"""


class Controller():
    def __init__(self, address: int):
        self.__lcd = CharLCD(i2c_expander='PCF8574', port=1, address=address, cols=16, rows=2)
        self._lock = threading.Lock()
        self.__buffer = ""

    def write_lock(func):
        @wraps(func)
        def wrapper(self, *args, **kwargs):
            with self._lock:
                return func(self, *args, **kwargs)
        return wrapper

    
    @property
    def buffer(self):
        return self.__buffer

    @buffer.setter
    @write_lock
    def buffer(self, value):
        self.__buffer = value
    
    @write_lock
    def write_buffer(self):
        self.__lcd.write_string(self.__buffer)

    @write_lock
    def clear(self):
        self.__lcd.clear()

    @write_lock
    def toggle_backlight(self, new_state):
        self.__lcd.backlight_enabled = new_state
    
    @write_lock
    def cursor_mode(self, mode):
        match mode:
            case "hide" | "line" | "blink":
                self.__lcd.cursor_mode = mode
            case _:
                print("Skipping, not a valid argument")

    @write_lock
    def home(self):
        self.__lcd.home()

    @write_lock
    def shift_display(self, amount):
        self.__lcd.shift_display(amount)

    @write_lock
    def move(self, x, y):
        try:
            self.__lcd.cursor_pos = (y, x)
        except ValueError as e:
            print(f"Skipping, Invalide possition: {e}")


def receive_data(sock):
    buffer = ''
    while True:
        try:
            part = sock.recv(64).decode('utf-8')
            buffer += part
        except BlockingIOError:
            part = ""
        finally:
            if "" == part and buffer != "":
                try:
                    messages = list(filter(lambda e: e != "", buffer.split("\n")))
                    return messages
                except:
                    print("An error occured while reciving a stream.")
                    return []

def connection_handler(conn, addr):
    global shared_controller
    #conn.settimeout(5)
    conn.setblocking(False)
    try:
        last_msg = time.time()
        while True:
            stream = receive_data(conn)
            for data in stream:
                req = ""
                try:
                    req = SocketLayout(**json.loads(data))
                    print(req)
                except json.JSONDecodeError:
                    print("Sciping one malformed request")
                    continue
                last_msg = time.time()
                match req.cmd.lower():
                    case "write": 
                        if req.args.get("additive", False):
                            shared_controller.buffer = f"{shared_controller.buffer}{req.args.get('text', '')}"
                        else: 
                            shared_controller.buffer = req.args.get("text", "")
                        if req.args.get("directly", True):
                            shared_controller.write_buffer()
                        continue
                    case "move":
                        shared_controller.move(req.args.get("x", 0), req.args.get("y", 0))
                        continue
                    case "cursor_mode":
                        shared_controller.cursor_mode(req.args.get("mode", "hide"))
                        continue
                    case "backlight":
                        shared_controller.toggle_backlight(req.args.get("state", True))
                        continue
                    case "shift_display":
                        shared_controller.shift_display(req.args.get("amount", 4))
                        continue
                    case "home":
                        shared_controller.home()
                        continue
                    case "clear":
                        shared_controller.clear()
            else:
                if time.time() - last_msg >= 500: # timeout in s
                    raise socket.timeout
    except socket.timeout as t:
        print(f"Closing connection due to timeout..")
    finally:
        print(f"Closed {(conn, addr)}")
        conn.close()

if __name__ == "__main__":
    try:
        os.unlink(socket_path)
    except OSError:
        pass
    
    if os.environ.get("DEBUG_DEMO", False) == "true":
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.bind(socket_path)
        s.listen()
        print("Running in debug demo, we will just print the data we recive")
        while True:
            conn_obj = s.accept()
            print(f"Accepted a new connection {conn_obj}")
 


    shared_controller = Controller(0x27)

    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.bind(socket_path)
    s.listen()
    print("Setup compleate, accepting connections")
    while True:
        conn_obj = s.accept()
        print(f"Accepted a new connection {conn_obj}")
        threading.Thread(target=connection_handler, args=(*conn_obj,)).start()
        

    Controller(0x27)
# https://rplcd.readthedocs.io/en/stable/usage.html
# echo '{"command": "buffer", "args": {"text": "Test message", "additive": true, "directly": true}}' | socat - UNIX-CONNECT:/home/someone/rust_watchdog/pylcdriver/lcd.sock

# echo '{"command": "clear"}' | socat - UNIX-CONNECT:/home/someone/rust_watchdog/pylcdriver/lcd.sock

# echo '{"command": "move", "args": {"x": 0, "y": 4}}' | socat - UNIX-CONNECT:/home/someone/rust_watchdog/pylcdriver/lcd.sock