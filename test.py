import socket
import json
import time

import gevent

from gevent import monkey, socket, time
monkey.patch_all()

def request(sn):
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.settimeout(15)
    server = ("127.0.0.1", 9000)
    s.connect(server)
    
    msg = {
        "v": "0.1",
        "sn": sn,
    }

    time.sleep(1)
    msg = json.dumps(msg)
    s.sendall(bytes(msg, encoding='utf-8'))

    print(s.recv(1000))
    s.sendall(bytes(msg, encoding='utf-8'))

    gevent.sleep(2)
    msg = {
        "type": "push",
        "msg": "okokokoko",
    }
    msg = json.dumps(msg)
    s.sendall(bytes(msg, encoding='utf-8'))

    s.close()

gs = []
for e in range(200):
    g = gevent.spawn(request, f"sn/controller/{e}")
    gs.append(g)

for g in gs:
    g.join()
