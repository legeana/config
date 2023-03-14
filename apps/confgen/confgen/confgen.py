#!/usr/bin/python3

import os
import pathlib
import sys

import jinja2

GEN_SFX = '.gen'
STD_PREFIX = 'std/'
STD_TEMPLATES = pathlib.Path(__file__).parent.absolute() / 'templates'


class ConfGenLoader(jinja2.BaseLoader):

  def __init__(self, path: pathlib.Path):
    self._path = path
    if STD_TEMPLATES.is_dir():
      self._std = STD_TEMPLATES
    else:
      self._std = None

  @staticmethod
  def _resolve(template, *search):
    for root in search:
      path = root / template
      if path.is_file():
        return path
    raise jinja2.TemplateNotFound(template)

  def get_source(self, environment, template):
    path = self._resolve(template, self._path, self._std)
    mtime = path.stat().st_mtime
    with open(path) as f:
      source = f.read()
    return source, path, lambda: mtime == pathlib.Path(path).stat().st_mtime


def helpers():
  return dict(
      exists=os.path.exists,
  )


def main():
  cwd = pathlib.Path.cwd()
  templates = cwd / '.confgen'
  if not templates.is_dir():
    sys.exit(f'Create {str(templates)!r} directory in order to use ConfGen')
  env = jinja2.Environment(loader=ConfGenLoader(templates))
  env.globals.update(**helpers())
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
