import abc
import dataclasses
import logging
import pathlib
import shlex
import shutil
import subprocess
import sys
from typing import Callable, Collection, Dict, List

PathRecorder = Callable[[pathlib.Path], None]


class Entry(abc.ABC):

  @abc.abstractmethod
  def install(self, record: PathRecorder) -> None:
    pass

  def post_install(self) -> None:
    """Post install hook."""
    pass


@dataclasses.dataclass
class FileEntry(Entry):

  src: pathlib.Path
  dst: pathlib.Path


class SymlinkEntry(FileEntry):

  def install(self, record: PathRecorder) -> None:
    if self.dst.exists():
      if not self.dst.is_symlink():
        logging.error(f'Symlink: unable to overwrite {str(self.dst)}')
        return
      self.dst.unlink()
    self.dst.parent.mkdir(parents=True, exist_ok=True)
    self.dst.symlink_to(self.src)
    record(self.dst)
    logging.info(f'{str(self.src)} -> {str(self.dst)}')


class CopyEntry(FileEntry):

  def install(self, record: PathRecorder) -> None:
    del record  # unused
    if self.dst.exists():
      logging.info(f'Copy: skipping already existing {str(self.dst)}')
      return
    self.dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(self.src, self.dst)
    # do not record this file in the deletion database to prevent data loss
    logging.info(f'{str(self.src)} -> {str(self.dst)}')


class PostInstallHook(Entry):

  def install(self, record: PathRecorder) -> None:
    del record  # unused

  @abc.abstractmethod
  def post_install(self) -> None:
    pass


@dataclasses.dataclass
class ExecPostHook(PostInstallHook):

  args: List[str]

  def post_install(self) -> None:
    logging.info(f'$ {" ".join(shlex.quote(arg) for arg in self.args)}')
    subprocess.check_call(self.args)


class ParserError(Exception):
  pass


@dataclasses.dataclass
class Parser(abc.ABC):

  root: pathlib.Path
  prefix: pathlib.Path

  @property
  @abc.abstractmethod
  def supported_commands(self) -> Collection[str]:
    return []

  def check_supported(self, command: str) -> None:
    if command not in self.supported_commands:
      raise ParserError(f'{command} is not supported by {type(self)}')

  @abc.abstractmethod
  def parse(self, command: str, args: List[str]) -> Entry:
    pass


class SinglePathParser(Parser):

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    if len(args) != 1:
      raise ParserError(f'{command} supports only 1 argument, got {len(args)}')
    return self.parse_single_path(command, pathlib.Path(args[0]))

  @abc.abstractmethod
  def parse_single_path(self, command: str, path: pathlib.Path) -> Entry:
    pass


class SymlinkParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['symlink']

  def parse_single_path(self, command: str, path: pathlib.Path) -> Entry:
    del command  # unused
    return SymlinkEntry(self.root / path, self.prefix / path)


class CopyParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['copy']

  def parse_single_path(self, command: str, path: pathlib.Path) -> Entry:
    del command  # unused
    return CopyEntry(self.root / path, self.prefix / path)


class ManifestParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['subdir']

  def parse_single_path(self, command: str, path: pathlib.Path) -> Entry:
    del command  # unused
    return Manifest(self.root / path, self.prefix / path)


class ExecPostHookParser(Parser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['post_install_exec']

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    return ExecPostHook(args)


class Manifest(Entry):

  def __init__(self, root: pathlib.Path, prefix: pathlib.Path = pathlib.Path()):
    self.root = root
    self._entries: List[Entry] = []
    manifest_path = root / 'MANIFEST'
    self._register_parsers(
        ManifestParser(root=root, prefix=prefix),
        SymlinkParser(root=root, prefix=prefix),
        CopyParser(root=root, prefix=prefix),
        ExecPostHookParser(root=root, prefix=prefix),
    )
    with open(manifest_path) as f:
      for lineno, line in enumerate(f, 1):
        try:
          self._add_line(line)
        except ParserError as e:
          sys.exit(f'{manifest_path}:{lineno}: {e}')

  def _register_parsers(self, *parsers: Parser) -> None:
    self._parsers: Dict[str, Parser] = dict()
    for parser in parsers:
      for command in parser.supported_commands:
        assert command not in self._parsers
        self._parsers[command] = parser

  def _add_line(self, line):
    if line.startswith('#'):
      return
    parts = shlex.split(line)
    if not parts:
      return
    if len(parts) == 0:
      return
    elif len(parts) == 1:
      self._add_command('symlink', parts)
    else:
      self._add_command(parts[0], parts[1:])

  def _add_command(self, command: str, args: List[str]) -> None:
    parser = self._parsers.get(command)
    if parser is None:
      raise ParserError(f'{command} is not supported by {type(self)}')
    self._entries.append(parser.parse(command, args))

  def install(self, record: PathRecorder) -> None:
    for entry in self._entries:
      entry.install(record)

  def post_install(self) -> None:
    for entry in self._entries:
      entry.post_install()
