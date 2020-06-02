from gevent import monkey
monkey.patch_all()
import device
from typing import Union, Dict, Tuple
# from multiprocessing.managers import BaseManager

class DeviceSocketPool:
    def __init__(self):
        self._devices = {}
        
    def _lock_acquire(self):
        pass
    
    def _lock_release(self):
        pass
        
    def put_device(self, sn: str, d: device.Device):
        self._lock_acquire()
        self._devices[sn] = d
        self._lock_release()

    def peak_device(self, sn: str) -> Union[None, device.Device]:
        return self._devices[sn] if sn in self._devices else None
    
    def get_device_num(self) -> int:
        return len(self._devices.keys())
    
    def get_alive_device_num(self) -> int:
        return len(list(filter(lambda e: e.is_alive, self._devices.values())))
    
    def rm_device(self, sn: str):
        if sn in self._devices:
            del self._devices[sn]
    
    def set_alive(self, sn: str, alive: bool):
        self._devices[sn].is_alive = alive
    
    def is_in_pool(self, sn: str) -> bool:
        return sn in self._devices
    
    def is_alive(self, sn: str) -> bool:
        if self.is_in_pool(sn):
            return self._devices[sn].is_alive
        else:
            return False
    
    def msg_put_thread(self, addr: Tuple[str, int]):
        """
        Post lastest msg to redis.
        """
        pass
 

# It is seem that Gevent confilct to multiprocessing.managers.Manager
# def ManagerInstance():
#     """
#     Create a process share instance
#     """                                                                                             
#     m = BaseManager()                                                                                               
#     m.register('DeviceSocketPool', DeviceSocketPool)                                                                                        
#     m.start()                                                                                                
#     return m.DeviceSocketPool()                                                                       
#     """
#     m = BaseManager()
#     m.register('DeviceSocketPool', DeviceSocketPool)
#     m.start()
#     return m.DeviceSocketPool()
# devicesocketpool = ManagerInstance()
 
devicesocketpool = DeviceSocketPool()
 
if __name__ == '__main__':
    print(devicesocketpool.is_in_pool('sdf'))
