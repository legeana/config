import base64
import hashlib
import pathlib

from pikaconfig import xdg

def _path_hash(path: pathlib.Path) -> str:
  data = str(path).encode('utf-8')
  h = hashlib.sha256(data)
  return base64.urlsafe_b64encode(h.digest()).decode('utf-8')


def make_state(path: pathlib.Path) -> pathlib.Path:
  """Return a path inside XDG_STATE_HOME uniquely identified by path."""
  pika_state = xdg.STATE_HOME / 'pikaconfig'
  output_state = pika_state / 'output'
  output_state.mkdir(parents=True, exist_ok=True)
  return output_state / _path_hash(path)