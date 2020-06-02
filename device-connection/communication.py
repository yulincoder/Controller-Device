#-*- coding: utf-8 -*-
#
# A cross-platform Mssage queue based redis
#
from gevent import monkey
monkey.patch_all()
import redis
import Config as cfg


class MessageQueue:
    def __init__(self, db=0, priority='p5'):
        self._queue = redis.StrictRedis(host=cfg.REDIS_ADDR,
                                        port=cfg.REDIS_PORT,
                                        db=0)
        self._priority = priority

    def get_priority(self) -> str:
        return self._priority

    def put(self, value):
        if not isinstance(value, str):
            raise RuntimeError("Must be a str type to put the queue of redis")
        self._queue.lpush(self._priority, value)

    def get(self, block=True, encoding='utf-8'):
        if block:
            v = self._queue.brpop(self._priority)
        else:
            v = self._queue.rpop(self._priority)

        if v is None:
            return None
        else:
            _, v = v
        if encoding:
            v = str(v, encoding=encoding)
        return v

    def clear(self, priority='p5'):
        self._queue.delete(priority)


# In[]
if __name__ == '__main__':
    mq = MessageQueue()
    mq.clear()
    mq.put('12')
    mq.put('14')
    mq.put('16')
    v = mq.get(block=True)
    print(v)
