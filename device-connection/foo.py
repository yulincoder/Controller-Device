

import device_support
from devicepool import devicesocketpool as dsp

print(dsp.get_device_num())
'''
test = []


def call_ping(sn, device):
    device.ping()
    test.append(device.sn)
    return sn, "OK"


gs = []
for e in devices.items():
    g = gevent.spawn(call_ping, *e)
    gs.append(g)
for e in gs:
    e.join()

print(test)
print(len(test))
'''