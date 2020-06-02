from gevent import monkey
monkey.patch_all()
from typing import Union, Tuple
import socket
import json
import logger
import time
import MsgField
import gevent
import traceback


class Device:
    def __init__(self,
                 sn: str,
                 ip_mac: Union[Tuple[str, int], None] = None,
                 sock: Union[socket.socket, None] = None,
                 is_alive: bool = True):
        self.sn = sn
        self.ip_mac = ip_mac
        self.socket = sock
        self.socket.setblocking(False)
        self.is_alive = is_alive


class GenericSwitcherDevice(Device):
    def __init__(self,
                 sn: str,
                 ip_mac: Union[str, None] = None,
                 sock: Union[socket.socket, None] = None,
                 is_alive: bool = True):
        super(GenericSwitcherDevice, self).__init__(sn, ip_mac, sock, is_alive)
        self._heartbeat_msg_dict = {"sn": None}

    def _build_ping_msg(self) -> str:
        self._heartbeat_msg_dict['sn'] = self.sn
        return json.dumps(self._heartbeat_msg_dict)

    def ping(self) -> bool:
        pingmsg = self._build_ping_msg()
        logger.info(pingmsg)
        if self.socket is not None:
            self.socket.sendall(pingmsg)
        time.sleep(1)
        logger.info(pingmsg + '  pong')

    def heartbeat(self) -> bool:
        if self.socket is None:
            self.is_alive = False
            return False
        sock_f = self.socket.makefile(mode='r', encoding='utf-8')
        msg = MsgField.HEARTBEAT
        msg['sn'] = self.sn
        msg = bytes(json.dumps(msg) + '\n', encoding='utf-8')
        self.socket.sendall(msg)

        for poll in range(3):
            gevent.sleep(1)
            try:
                data = sock_f.readline()
                if data:
                    logger.info(
                        '{}: device({}) heartbeat was succeed, {}'.format(
                            __file__, self.sn, data))
                    self.is_alive = True
                    return True
            except:
                err_msg = traceback.format_exc()
                logger.info('{}: device({}) heartbeat was fail, {}'.format(
                    __file__, self.sn, err_msg))
                continue

        logger.warning('{}: device({}) heartbeat was fail'.format(
            __file__, self.sn))
        self.is_alive = False
        return False

    def readline(self) -> Union[str, None]:
        sock_f = self.socket.makefile(mode='r', encoding='utf-8')
        data = sock_f.readline()
        return None if len(data) == 0 else data


if __name__ == '__main__':
    switcher = GenericSwitcherDevice(sn='abc/12344', ip_mac="192.168.1.1")
    switcher.ping()
