#!/usr/bin/python3
# -*- coding: utf-8 -*-

from subprocess import *
import shlex
import re
import sys
import argparse
import os

device = "PCM"
bufsize = 4096

increment = re.compile("^\+$")
decrement = re.compile("^-$")
changing = re.compile("^[-+][0-9]{1,3}$")

get_command = "amixer get {0} | grep 'Front Left: Playback' | sed -r 's|^[^[]+\[([[:digit:]]+)%\].*$|\\1|'"
set_command = "amixer set {0} {1}%"


def get_volume():
    with Popen(get_command.format(device), shell=True, bufsize=bufsize, stdout=PIPE).stdout as pipe:
        return int(pipe.read())


def set_volume(x):
    vol = 0
    if increment.match(x):
        #vol = int(get_volume()*1.25+1)
        vol = int(get_volume()+10)
    elif decrement.match(x):
        #vol = int(get_volume()*0.8-1)
        vol = int(get_volume()-10)
    elif changing.match(x):
        vol = int(get_volume()+x)
    else:
        vol = int(x)
    if vol<0:
        vol = 0
    if vol>100:
        vol = 100
    return call(set_command.format(device, vol), shell=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(prog="volume")
    parser.add_argument('-s', '--set', action="store", dest="volume", help="new volume (with '+' or '-' will be increased or decreased)")
    parser.add_argument('-v', '--version', action='version', version='%(prog)s 0.0.1', help="version information")
    args = parser.parse_args()
    if args.volume:
        set_volume(args.volume)
    else:
        print(get_volume())
