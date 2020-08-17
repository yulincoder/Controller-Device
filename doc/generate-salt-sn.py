import hashlib
import string


def md5(s: str, encode="utf-8") -> str:
    md = hashlib.md5()
    md.update(s.encode(encode))
    return md.hexdigest()


def salted_sn(raw_sn: str, salt='anbwscx') -> str:
    salted = md5(raw_sn + salt)
    return (raw_sn + salted[-3:]).upper()


def generate_date_mark(year, month, day):
    year_mark = {2020+year: mark for year,
                 mark in enumerate(string.ascii_uppercase)}
    month_mark = {i: hex(i)[2:].upper() for i in range(1, 13)}
    day_mark = {day+1: mark for day, mark in enumerate(
        ''.join([str(i) for i in range(1, 10)]) + string.ascii_uppercase)}
    return year_mark[year] + month_mark[month] + day_mark[day]


def generate_s(product_id, year, month, day, serial_start, serial_end, title, f):
    if not title:
        f.write(title+'\n')
    for e in range(serial_start, serial_end+1):
        raw_sn = product_id + generate_date_mark(year, month, day)
        print(f'raw_sn {raw_sn}')
        sn1_salted = salted_sn(raw_sn + f'{hex(e)[2:]:>04}')
        print(sn1_salted, len(sn1_salted))
        f.write(sn1_salted+'\n')


if __name__ == '__main__':
    print('''
        ---------------------------------------
        sn生成器
        [注意] 日期没有大小年，大小月校验
        ---------------------------------------
              ''')

    year, month, day = int(input("        输入年# ")), int(
        input("        输入月# ")), int(input("        输入日# "))
    assert 2020 <= year <= 2045, '年错误'
    assert 1 <= month <= 12, '月错误'
    assert 1 <= day <= 31, '日错误(没有大小月，平润校验)'

    serial_start = int(input("        输入流水号起始编号# "))
    serial_end = int(input("        输入流水号截止编号# "))

    print('''
        ---------------------------------------
        单控开关: B0
        双控开关: C1
        三控开关: D2
        单控带温湿度: E3
        双控带温湿度: F4
        三控带温湿度: G5
        ---------------------------------------''')

    product_id = input("        输入产品ID# ")
    assert product_id in ['B0', 'C1', 'D2', 'E3', 'F4', 'G5'], '错误的ID'

    with open('salted_sn.csv', 'w') as f:
        generate_s(product_id, year, month, day,
                   serial_start, serial_end, 'Controller', f)
    print('''
        -------------------------------------
        输出到salted_sn.csv
        ''')
