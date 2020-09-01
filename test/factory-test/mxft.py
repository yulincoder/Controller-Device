#-*- coding: utf-8 -*-

import os
import serial

input_csv = input("输入sn文件# ")
fp, fn = os.path.split(input_csv.strip())
fn, ext = os.path.splitext(fn)
output_csv = fn + '-finished' + ext


def get_first_sn(fp: str) -> str:
	with open(fp, 'r') as f:
		lines = f.readlines()
		try:
			ret = lines[0]
		except IndexError:
			return ''
	return ret.strip()

def del_first_sn(fp: str):
	with open(fp, 'r') as f:
		lines = f.readlines()
		try:
			ret = lines.pop(0)
		except IndexError:
			return ''
	with open(fp, 'w') as f:
		f.write(''.join(lines))



def dump_sn(fp: str, sn) -> str:
	with open(fp, 'a') as f:
		f.write(sn+'\n')

def wirte_device(sn: str):
	return 
	with serial.Serial(port='com4', baudrate=115200) as ser:
		ser.write(sn)


def process_a_sn() -> bool:
	sn = get_first_sn(input_csv)
	if sn == '':
		print('没有sn号了')
		return False
	write_sn = False

	while True:
		do = input(f'确定烧写({sn})？默认烧写(y/N): ')
		if do in ['', 'y', 'Y']:
			write_sn = True
			break
		elif do in ['n', 'N']:
			write_sn = False
			break
		print(sn)

	if write_sn:
		print(f'开始烧写sn号({sn}) ...')
		wirte_device(sn)
		dump_sn(output_csv, sn)
		del_first_sn(input_csv)
		print(f'烧写完毕')
	else:
		while True:
			next_step = input('  程序退出: 1 \n烧写下一台: 2 #')
			if next_step == '1':
				return False
			elif next_step == '2':
				return True

def check_device(port: str, baudrate: float) -> bool:
	with serial.Serial(port=port, baudrate=baudrate) as ser:
		ser.timeout = 1
		ser.write(bytes('READY?\n'))
		ret = ser.readline()
		if 'NO READY' not in ret:
			return False
		return True


while True:
	next_step = input('烧录下一台?默认下一台(y/N): ')
	if next_step in ['', 'y', 'Y']:
		stats = process_a_sn()
		if stats is False:
			print(f'退出 ...')
			break			
	elif next_step in ['n', 'N']:
		print(f'退出 ...')
		break
	print('\n')



