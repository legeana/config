import dataclasses
import pathlib
from typing import Callable, Collection, Optional, Type

from . import tagutil

PathRecorder = Callable[[pathlib.Path], None]


class Prefix:

  _base: pathlib.Path
  _current: pathlib.Path

  def __init__(self, base: pathlib.Path):
    self._base = base
    self._current = base

  @property
  def base(self) -> pathlib.Path:
    """Base value inherited from the parent.

    Represents current MANIFEST before any overrides took place."""
    return self._base

  @property
  def current(self) -> pathlib.Path:
    """Value equal to the base or provided by the user."""
    return self._current

  @current.setter
  def current(self, path: pathlib.Path) -> None:
    self._current = path


class Entry:

  def tags_match(self, tags: tagutil.TagSet) -> bool:
    return True

  def recursive_system_setup(self, tags: tagutil.TagSet) -> None:
    if not self.tags_match(tags):
      return
    self.system_setup()

  def recursive_install(self, tags: tagutil.TagSet,
                        record: PathRecorder) -> None:
    if not self.tags_match(tags):
      return
    self.install(record)

  def recursive_post_install(self, tags: tagutil.TagSet) -> None:
    """Post install hook."""
    if not self.tags_match(tags):
      return
    self.post_install()

  def system_setup(self) -> None:
    pass

  def install(self, record: PathRecorder) -> None:
    raise NotImplementedError

  def post_install(self) -> None:
    """Post install hook."""
    pass


class ParserError(Exception):
  pass


class Parser:

  @property
  def supported_commands(self) -> Collection[str]:
    raise NotImplementedError

  def check_supported(self, command: str) -> None:
    if command not in self.supported_commands:
      raise ParserError(f'{command} is not supported by {type(self)}')

  def parse(self, command: str, args: list[str]) -> Entry:
    raise NotImplementedError
