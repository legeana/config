#!/usr/bin/python3
# -*- coding: utf-8 -*-

from subprocess import *
import shlex
import re
import sys
import argparse
import os

device = "00:02.0"
bufsize = 4096

#increment = re.compile("^\+[0-9a-fA-F]{2}$")
#decrement = re.compile("^-[0-9a-fA-F]{2}$")
increment = re.compile("^\+$")
decrement = re.compile("^-$")
changing = re.compile("^[-+][0-9a-fA-F]{2}$")
#value = re.compile("^[0-9a-fA-F]{2}$")

get_command = "lspci -xxx -s {0} | grep 'f0:' | cut -s -f6 -d' ' "
set_command = "setpci -s 00:02.0 F4.B={0}"

def h2d(x):
    return int(x, 16)

def d2h(d):
    return '%x' % (d)

def get_brightness():
    pipe = None
    if os.geteuid()!=0:
        pipe = Popen("sudo {0}".format(sys.argv[0]), shell=True, bufsize=bufsize, stdout=PIPE).stdout
    else:
        pipe = Popen(get_command.format(device), shell=True, bufsize=bufsize, stdout=PIPE).stdout
    v = pipe.read()
    pipe.close()
    return h2d(v.strip())

def set_brightness(x):
    if os.geteuid()!=0:
        return call("sudo {0} -s {1}".format(sys.argv[0], x), shell=True)
    br = 0
    if increment.match(x):
        br = int(get_brightness()*1.25+1)
    elif decrement.match(x):
        br = int(get_brightness()*0.8-1)
    elif changing.match(x):
        br = get_brightness() + h2d(x)
    else:
        br = h2d(x)
    if br<0x00:
        br = 0x00
    if br>0xff:
        br = 0xff
    br = d2h(br)
    return call(set_command.format(br), shell=True)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(prog="brightness")
    parser.add_argument('-s', '--set', action="store", dest="brightness", help="new brightness (with '+' or '-' will be increased or decreased)")
    #parser.add_argument('-a', '--auto', dest="auto", action='store_true', help="automatic (for example in acpi runlevel switch)")
    #parser.add_argument('-r', '--root', action="store_true", dest="as_root", help="run as root using sudo")
    parser.add_argument('-v', '--version', action='version', version='%(prog)s 0.0.1', help="version information")
    args = parser.parse_args()
    if args.brightness:
        set_brightness(args.brightness)
    else:
        print(d2h(get_brightness()))

