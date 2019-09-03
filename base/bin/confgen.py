#!/usr/bin/python3

import sys
import os
import json
from os.path import join, exists, getmtime

from jinja2 import BaseLoader, Environment, PackageLoader


class ConfGenLoader(BaseLoader):

    def __init__(self, path):
        self._path = path

    def get_source(self, environment, template):
        path = join(self._path, template)
        if not exists(path):
            raise TemplateNotFound(template)
        mtime = getmtime(path)
        with open(path) as f:
            source = f.read()
        return source, path, lambda: mtime == getmtime(path)


if __name__=='__main__':
    cwd = os.getcwd()
    templates = join(cwd, '.confgen')
    generate = join(cwd, '.confgen', 'generate.json')
    if not exists(templates):
        print('Create "{}" directory in order to use ConfGen'.format(templates))
        sys.exit(2)
    if not exists(generate):
        print('Create "{}" file in order to use ConfGen'.format(generate))
        sys.exit(3)
    env = Environment(loader=ConfGenLoader(templates))
    with open(generate) as gen:
        generate = json.load(gen)
    for i in generate:
        templ = env.get_template(i)
        with open(i, "w") as f:
            print(templ.render(), file=f)
