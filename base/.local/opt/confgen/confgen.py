#!/usr/bin/python3

import json
import os
import pathlib
import sys

from jinja2 import BaseLoader, Environment, PackageLoader

GEN_SFX = '.gen'


class ConfGenLoader(BaseLoader):

  def __init__(self, path):
    self._path = path

  def get_source(self, environment, template):
    path = self._path / template
    if not path.is_file():
      raise TemplateNotFound(template)
    mtime = path.stat().st_mtime
    with open(path) as f:
      source = f.read()
    return source, path, lambda: mtime == pathlib.Path(path).stat().st_mtime


def main():
  cwd = pathlib.Path.cwd()
  templates = cwd / '.confgen'
  if not templates.is_dir():
    sys.exit(f'Create {templates!r} directory in order to use ConfGen')
  env = Environment(loader=ConfGenLoader(templates))
  for root, dirs, files in os.walk(templates):
    root = pathlib.Path(root)
    relroot = root.relative_to(templates)
    outroot = cwd / relroot
    outroot.mkdir(parents=True, exist_ok=True)
    for tmpl in files:
      tmpl = pathlib.Path(tmpl)
      if tmpl.suffix == GEN_SFX:
        templ = env.get_template(str(relroot / tmpl))
        tout = outroot / tmpl.stem
        with open(tout, mode='w') as out:
          print(f'Writing {tout}')
          out.write(templ.render())


if __name__ == '__main__':
  main()
