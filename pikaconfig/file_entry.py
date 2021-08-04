import dataclasses
import logging
import pathlib
import shutil
from typing import Collection, List

from . import entry
from . import local_state
from . import util


@dataclasses.dataclass
class SinglePathParser(entry.Parser):

  root: pathlib.Path
  prefix: entry.Prefix

  def parse(self, command: str, args: List[str]) -> entry.Entry:
    self.check_supported(command)
    if len(args) != 1:
      raise entry.ParserError(
          f'{command} supports only 1 argument, got {len(args)}')
    return self.parse_single_path(command, pathlib.Path(args[0]))

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    raise NotImplementedError


@dataclasses.dataclass
class FileEntry(entry.Entry):

  src: pathlib.Path
  dst: pathlib.Path


class SymlinkEntry(FileEntry):

  def install(self, record: entry.PathRecorder) -> None:
    if self.dst.exists():
      if not self.dst.is_symlink():
        logging.error(f'Symlink: unable to overwrite {util.format_path(self.dst)}')
        return
      self.dst.unlink()
    self.dst.parent.mkdir(parents=True, exist_ok=True)
    self.dst.symlink_to(self.src)
    record(self.dst)
    logging.info(f'{util.format_path(self.src)} -> {util.format_path(self.dst)}')


class SymlinkParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['symlink']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    return SymlinkEntry(self.root / path, self.prefix.current / path)


class CopyEntry(FileEntry):

  def install(self, record: entry.PathRecorder) -> None:
    del record  # unused
    if self.dst.exists():
      logging.info(f'Copy: skipping already existing {util.format_path(self.dst)}')
      return
    self.dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(self.src, self.dst)
    # do not record this file in the deletion database to prevent data loss
    logging.info(f'{util.format_path(self.src)} -> {util.format_path(self.dst)}')


class CopyParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['copy']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    return CopyEntry(self.root / path, self.prefix.current / path)


@dataclasses.dataclass
class OutputFileEntry(entry.Entry):
  """OutputFileEntry registers an auto-generated file.

  It is not safe to automatically delete a configuration file.
  There is nothing worse than losing a configuration accidentally,
  because of this pikaconfig only deletes symlinks.
  However this does not solve the problem of automatically generated files
  that can be left behind if configuration is uninstalled.
  This entry allows a user to register a file as a generated output.
  pikaconfig will still not remove the date, instead it will pre-create
  a symlink to a pikaconfig-owned location. That file will not be removed
  either, but at least it's not left in the middle of the directory tree.
  """

  dst: pathlib.Path

  def install(self, record: entry.PathRecorder) -> None:
    src = local_state.make_state(self.dst)
    SymlinkEntry(src=src, dst=self.dst).install(record)


class OutputFileParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['output_file']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    return OutputFileEntry(self.prefix.current / path)
