import copy
import os
import pathlib
import sys
import tempfile
from typing import Callable, Iterable, IO, Optional

FilterFunction = Callable[[str], bool]


def _authorized_keys_path() -> pathlib.Path:
  return pathlib.Path.home() / '.ssh' / 'authorized_keys'


class AuthorizedKeys:

  _lines: list[str]

  def __init__(self):
    self._lines = []

  @classmethod
  def load(cls) -> 'AuthorizedKeys':
    keys = cls()
    with open(_authorized_keys_path(), 'r') as inp:
      keys.extend(inp.readlines())
    return keys

  def save(self) -> None:
    path = _authorized_keys_path()
    with tempfile.NamedTemporaryFile(
        mode='w', prefix=str(path), delete=False) as tmpfile:
      # TODO make sure permissions are correct
      try:
        self.print(tmpfile)
        tmpfile.close()
        os.rename(tmpfile.name, path)  # must be the last action
      except:
        os.unlink(tmpfile.name)
        raise

  def print(self, file: Optional[IO[str]] = None) -> None:
    if file is None:
      file = sys.stdout
    for line in self._lines:
      file.write(line)
      if not line.endswith('\n'):
        file.write('\n')

  def append(self, line: str) -> None:
    line = line.strip('\n')
    if line:
      self._lines.append(line)

  def extend(self, lines: Iterable[str]) -> None:
    for line in lines:
      self.append(line)

  def filter(self, function: FilterFunction) -> None:
    self._lines = [line for line in self._lines if function(line)]

  def replace(self, function: FilterFunction, lines: Iterable[str]) -> None:
    """Replace entries that do not match function with lines.

    If this method raises the object is unchanged.
    """
    # run the code below as a transaction
    try:
      new = copy.deepcopy(self)
      # remove old entries first as function will match lines
      new.filter(function)
      new.extend(lines)
      # transaction is complete, replace self with new
      self._lines = new._lines
    except:
      raise


def is_not_token(token: str) -> FilterFunction:
  def matcher(key: str) -> bool:
    return token not in key
  return matcher


def is_tokens_except(tokens: Iterable[str], exception: str) -> FilterFunction:
  token_list = list(tokens)
  def matcher(key: str) -> bool:
    if any(token in key for token in token_list):
      return True
    if exception in key:
      return False
    return True
  return matcher
