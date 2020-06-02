
import socket
import json
import time

import gevent

from gevent import monkey, socket, time
monkey.patch_all()
cnt = 0


def request(sn, close=True):
    global cnt
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    #s.settimeout(15)
    server = ("127.0.0.1", 9000)
    s.connect(server)
    s.setblocking(True)
    msg = {
        "v": "0.1",
        "sn": sn,
    }

    time.sleep(1)
    msg = json.dumps(msg) + '\n'
    s.sendall(bytes(msg, encoding='utf-8'))

    while not close:
        cnt += 1
        print(s.recv(1000))
        s.sendall(bytes(msg, encoding='utf-8'))
        msg = {
            "type": "push",
            "msg": "okokokoko",
        }
        msg = json.dumps(msg) + '\n'
        s.sendall(bytes(msg, encoding='utf-8'))

        time.sleep(1)
        event = {
            "event": "frome the device {}".format(cnt),
            }
        event = json.dumps(event)+'\n'
        print(event)
        s.sendall(
            bytes(event, encoding='utf-8'))
    s.close()


gs = []
for e in range(500):
    g = gevent.spawn(request, f"sn/controller/{e*4000}",
                     False if e < 2500 else True)
    gs.append(g)

for g in gs:
    g.join()
