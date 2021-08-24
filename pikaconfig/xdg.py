import pathlib
import os


def _xdg_path(name: str, default: pathlib.Path) -> pathlib.Path:
  env = os.environ.get(name)
  if env:
    return pathlib.Path(env)
  return default

_HOME = pathlib.Path.home()

CACHE_HOME = _xdg_path('XDG_CACHE_HOME', _HOME / '.cache')
CONFIG_HOME = _xdg_path('XDG_CONFIG_HOME', _HOME / '.config')
DATA_HOME = _xdg_path('XDG_DATA_HOME', _HOME / '.local' / 'share')
STATE_HOME = _xdg_path('XDG_STATE_HOME', _HOME / '.local' / 'state')
