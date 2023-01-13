#!/usr/bin/env python3

import socket
import struct
import json
import subprocess
import sys
import time
import itertools

class SocketClosedException(Exception): pass
MARKS = 'QWERTYUIOP'

COMMANDS = [
    'run_command',
    'get_workspaces',
    'subscribe',
    'get_outputs',
    'get_tree',
    'get_marks',
    'get_bar_config',
    'get_version',
    'get_binding_modes',
    'send_tick',
]

EVENTS = [
    'workspace',
    'output',
    'mode',
    'window',
    'barconfig_update',
    'binding',
    'shutdown',
    'tick',
]

def recv(sock, length):
    buf = b''
    while length:
        data = sock.recv(length)
        if not data:
            break
        buf += data
        length -= len(data)
    return buf

def send_msg(sock, command, payload=''):
    payload = payload.encode('utf-8')
    msg = struct.pack('II', len(payload), COMMANDS.index(command))
    sock.sendall(b'i3-ipc' + msg + payload)

    while True:
        type, response = read_msg(sock)
        if type == command:
            if command == 'run_command':
                response = response[0]
            if isinstance(response, dict) and not response.get('success', True):
                raise Exception(response.get('error'))
            return response

def read_msg(sock):
    reply = recv(sock, 14)
    if not reply:
        raise SocketClosedException
    length, type = struct.unpack('ii', reply[6:])
    if type & 0x80000000:
        type = EVENTS[type & 0x7fffffff]
    else:
        type = COMMANDS[type]
    return type, json.loads(recv(sock, length))

def refresh_all_marks(sock, marks):
    # sort left to right, top to bottom
    workspaces = sorted(send_msg(sock, 'get_workspaces'), key=lambda w: (w['rect']['y'], w['rect']['x']))
    visible_ws = [w['name'] for w in workspaces if w['visible']]
    tree = send_msg(sock, 'get_tree')

    windows = itertools.chain.from_iterable(get_windows(tree, workspace) for workspace in visible_ws)
    for mark, id in zip(marks, windows):
        try:
            send_msg(sock, 'run_command', '[con_id="{}"] mark --replace {}'.format(id, mark))
        except Exception as e:
            if str(e) == "Given criteria don't match a window":
                pass
            else:
                raise

def get_windows(node, workspace):
    if node['window']:
        yield node['id']
    elif node['type'] == 'dockarea':
        return
    elif node['type'] == 'workspace' and node['name'] != workspace:
        return
    else:
        for child in node['nodes'] + node['floating_nodes']:
            yield from get_windows(child, workspace)

if __name__ == '__main__':
    no_socket_counter = 0
    marks = sys.argv[1] if len(sys.argv) > 1 else MARKS
    while True:
        try:
            socketpath = subprocess.check_output(['i3', '--get-socketpath']).rstrip(b'\n')
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as sock:
                sock.connect(socketpath)
                no_socket_counter = 0

                send_msg(sock, 'subscribe', json.dumps(['workspace', 'output', 'window']))
                refresh_all_marks(sock, marks)
                while True:
                    type, event = read_msg(sock)
                    if type in {'workspace', 'output'} or (type == 'window' and event['change'] in {'new', 'close', 'move', 'floating'}):
                        refresh_all_marks(sock, marks)

        except SocketClosedException:
            pass
        except FileNotFoundError:
            no_socket_counter += 1
            if no_socket_counter > 10:
                raise
            time.sleep(0.1)
        except KeyboardInterrupt:
            break
