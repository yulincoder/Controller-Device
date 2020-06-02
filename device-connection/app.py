from flask import Flask, request
import threading

import device_support
from devicepool import devicesocketpool as dsp
import Config as cfg
import json
import logger

app = Flask(__name__)


@app.route('/')
def hello_world():
    return 'MX-Backend'


@app.route('/query/alive_num', methods=['GET'])
def get_alive_device_num():
    """
    HTTP API: Query alive num
    """
    resp = {
        "type": "api.query",
        "deviceinfo": dsp.get_alive_device_num(),
    }
    print('app:', dsp.get_alive_device_num())
    return json.dumps(resp)


@app.route('/query/devices_num', methods=['GET'])
def get_device_num():
    """
    HTTP API: Query alive num
    """
    resp = {
        "type": "api.query",
        "deviceinfo": dsp.get_device_num(),
    }
    return json.dumps(resp)


@app.route('/query/is_alive', methods=['GET'])
def get_sn_is_alive():
    """
    HTTP API: Query a sn whether alive
    """
    sn = request.args.get('sn')
    logger.info("{}: query sn({}) whether".format(__file__, str(sn)))
    is_alive = dsp.is_alive(sn)
    resp = {
        'type': "api.query",
        'is_alive': is_alive,
        'sn': sn,
    }
    return json.dumps(resp)


def restful_run() -> threading.Thread:
    t = threading.Thread(target=app.run, args=(cfg.WEB_ADDR, cfg.WEB_PORT))
    t.start()
    return t


def device_maintain() -> threading.Thread:
    t = threading.Thread(target=device_support.device_maintain)
    t.start()
    return t


if __name__ == '__main__':
    t1 = restful_run()
    t2 = device_maintain()
    t1.join()
    t2.join()
