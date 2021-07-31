#!/usr/bin/python3

# http://color.smyck.org/

colors = [
    ('black', '000000'),
    ('red', 'C75646'),
    ('green', '8EB33B'),
    ('yellow', 'D0B03C'),
    ('blue', '72B3CC'),
    ('magenta', 'C8A0D1'),
    ('cyan', '218693'),
    ('gray', 'B0B0B0'),
    ('dark_gray', '5D5D5D'),
    ('light_red', 'E09690'),
    ('light_green', 'CDEE69'),
    ('light_yellow', 'FFE377'),
    ('light_blue', '9CD9F0'),
    ('light_magenta', 'FBB1F9'),
    ('light_cyan', '77DFD8'),
    ('white', 'F7F7F7'),
]


if __name__ == '__main__':
  for name, code in colors:
    red = code[0:2]
    green = code[2:4]
    blue = code[4:6]
    print(name, int(red, 16), int(green, 16), int(blue, 16))
