import logging
import pathlib
import shlex
import subprocess
from typing import Optional


def unexpanduser(path: pathlib.Path) -> pathlib.Path:
  try:
    home = pathlib.Path.home()
  except Exception:
    return path
  if home in path.parents:
    return pathlib.Path('~') / path.relative_to(home)
  return path


def format_path(path: pathlib.Path) -> str:
  return str(unexpanduser(path))


def verbose_check_call(*args, cwd: Optional[pathlib.Path] = None) -> None:
  pwd = '' if cwd is None else f'[{format_path(cwd)}] '
  command = f'$ {" ".join(shlex.quote(arg) for arg in args)}'
  logging.info(pwd + command)
  subprocess.check_call(args, cwd=cwd)
