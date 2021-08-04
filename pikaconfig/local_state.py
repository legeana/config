import base64
import hashlib
import pathlib


def _path_hash(path: pathlib.Path) -> str:
  data = str(path).encode('utf-8')
  h = hashlib.sha256(data)
  return base64.urlsafe_b64encode(h.digest()).decode('utf-8')


def make_state(path: pathlib.Path) -> pathlib.Path:
  """Return a path inside XDG_STATE_HOME uniquely identified by path."""
  # TODO: use XDG_STATE_HOME
  pika_state = pathlib.Path.home() / '.local' / 'state' / 'pikaconfig'
  output_state = pika_state / 'output'
  output_state.mkdir(parents=True, exist_ok=True)
  return output_state / _path_hash(path)
