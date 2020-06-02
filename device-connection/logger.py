from gevent import monkey, socket
monkey.patch_all()
import logging

LOG_FILENAME = "device_log.log"

logger = logging.getLogger()

logger.setLevel('INFO')
BASIC_FORMAT = "%(asctime)s [%(levelname)s] - %(message)s"
DATE_FORMAT = '%Y-%m-%d %H:%M:%S %p'
formatter = logging.Formatter(BASIC_FORMAT, DATE_FORMAT)

chlr = logging.StreamHandler() # 输出到控制台的handler
chlr.setFormatter(formatter)
chlr.setLevel('INFO')


fhlr = logging.FileHandler(LOG_FILENAME) # 输出到文件的handler
fhlr.setFormatter(formatter)
fhlr.setLevel('INFO')

logger.addHandler(chlr)
logger.addHandler(fhlr)

def info(msg):
    logging.info('\033[1;32m'+msg+'\033[0m')

def debug(msg):
    logging.debug('\033[1;36m'+msg+'\033[0m')

def warning(msg):
    logging.warning('\033[1;34m'+msg+'\033[0m')

def error(msg):
    logging.error('\033[1;31m'+msg+'\033[0m')

def critical(msg):
    logging.critical('\033[1;91m'+msg+'\033[0m')