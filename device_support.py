import socket
import logger
import json
import traceback
import subprocess
from concurrent.futures import ThreadPoolExecutor
from typing import Union, Tuple
import threading
import device
import queue
import gevent
from gevent import monkey, socket, time
monkey.patch_all()

devices = {}
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

socket_queue = queue.Queue()



def checkdevice(data) -> Union[bool, str]:
    """
    Check a new connection whether a device
    :param data: the first massage from a new connection
    :return: False if it a invalid else it's sn
    """
    try:
        logger.info('{}: receive data :{}'.format(__file__, data))
        data = json.loads(data)
        if 'sn' in data:
            return data['sn']
    except:
        return False

def handshake(sock: socket, addr: Tuple[str, int]) -> Union[device.Device, None]:
    """
    Handshake
    :param sock:
    :param addr:
    :return: True if handshake succeed else False
    """
    sock.setblocking(False)
    sock_f = sock.makefile(mode='r', encoding='utf-8')
    d = None
    # Handshake
    for poll in range(3):
        gevent.sleep(3)
        try:
            data = sock_f.readline()
            print(data)
            sn = checkdevice(data)
            if sn is False:
                logger.warning("{}: poll {}, error sn with ip({})".format(
                    __file__, poll, str(addr)))
                continue
            d = device.GenericSwitcherDevice(sn=sn, ip_mac=addr, sock=sock, is_alive=True)
            logger.info(
                '{}: poll {}, alive device {}, connection{} was succeed, save it.'.format(
                    __file__, poll, len(devices), str(addr)))
            break
        except:
            err_msg = traceback.format_exc()
            logger.error("{}: ip({}) abort with error: {}".format(
                __file__, str(addr), err_msg))
            continue
    if d is None:
        sock.close()
        logger.warning('{}: connection{} failed, close it.'.format(
            __file__, str(addr)))
    return d


def long_connection(d: device.Device) -> None:
    """
    Maintain a long connection with a device.
    :param sock:
    :param addr:
    :return:
    :exception:
    """
    heartbeat_cnt = 0
    while True:
        data = d.readline()
        if data:
            logger.info('{}: receive a message from {}: {}'.format(__file__, str(d.ip_mac), data))
            # push to redis
            heartbeat_cnt = 0
        else:
            heartbeat_cnt += 1

        if heartbeat_cnt == 20:
            logger.info('{}:heart beat with {}'.format(__file__, str(d.ip_mac)))
            if d.heartbeat() is False:
                logger.warning('{}:heart beat fail with {}, delete it.'.format(__file__, str(d.ip_mac)))
                del devices[d.sn] # Fail to heartbeat_cnt, delete it from devices alive.
                break
            logger.info('{}:heart beat succeed with {}.'.format(__file__, str(d.ip_mac)))
            heartbeat_cnt = 0
        gevent.sleep(0.5)

def maintain(sock: socket, addr: Tuple[str, int]) -> None:
    """
    Maintain a long connection between device.
    :param sock:
    :param addr:
    :return:
    """
    print('Accept new connection from %s:%s...' % addr)
    d = handshake(sock, addr)
    if d is None:
        logger.warning('{}: handshake with {} failed, close it.'.format(
            __file__, str(addr)))
        return
    devices[d.sn] = d
    long_connection(devices[d.sn])
    '''
    while True:
        pass
    '''

def accept_connection_handler():
    logger.info("{}: tcp process".format(__file__))
    while True:
        sock, addr = socket_queue.get()
        print('---------------- ', len(devices.keys()))
        logger.info("{}: get a connection({})".format(__file__, str(addr)))
        gevent.spawn(maintain, sock, addr)

def device_maintain():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    PACKET_SIZE = 4 * 1024

    # 监听端口:
    s.bind(('0.0.0.0', 9000))

    s.listen(10000)
    print('Waiting for connection...')

    t = threading.Thread(target=accept_connection_handler)
    t.start()

    while True:
        logger.info("{}: start accept".format(__file__))
        sock, addr = s.accept()

        logger.info("{}: put a connection()".format(__file__, str(addr)))
        socket_queue.put((sock, addr))
