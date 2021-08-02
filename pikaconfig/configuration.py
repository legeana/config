import dataclasses
import logging
import pathlib
import shlex
import shutil
import sys
from typing import Collection, Dict, List

from . import entry
from . import post_install_hook
from . import system_entry
from . import util


class CombinedParser(entry.Parser):

  def __init__(self, *parsers):
    self._parsers: Dict[str, Parser] = dict()
    for parser in parsers:
      for command in parser.supported_commands:
        assert command not in self._parsers
        self._parsers[command] = parser

  @property
  def supported_commands(self) -> Collection[str]:
    return self._parsers.keys()

  def parse(self, command: str, args: List[str]) -> entry.Entry:
    parser = self._parsers.get(command)
    if parser is None:
      raise entry.ParserError(f'{command} is not supported by {type(self)}')
    return parser.parse(command, args)


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
class SetPrefixEntry(entry.Entry):

  prefix: pathlib.Path

  def install(self, record: entry.PathRecorder) -> None:
    del record  # unused
    # noop


class SetPrefixParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['prefix']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    # It is important to set prefix relative to the base value
    # inherited from parent prefix plus current directory.
    # 'prefix' command should always set prefix
    # relative to the current directory.
    self.prefix.current = self.prefix.base / path.expanduser()
    return SetPrefixEntry(prefix=path)


class Manifest(entry.Entry):

  def __init__(self, root: pathlib.Path, prefix: pathlib.Path):
    self._entries: List[entry.Entry] = []
    self._path = root / 'MANIFEST'
    self._prefix = entry.Prefix(prefix)
    self._parsers = CombinedParser(
        ManifestParser(root=root, prefix=self._prefix),
        SetPrefixParser(root=root, prefix=self._prefix),
        system_entry.SystemCommandParser(),
        system_entry.AnyPackageParser(),
        system_entry.PacmanPackageParser(),
        system_entry.AptPackageParser(),
        system_entry.BrewPackageParser(),
        system_entry.PipPackageParser(),
        SymlinkParser(root=root, prefix=self._prefix),
        CopyParser(root=root, prefix=self._prefix),
        post_install_hook.ExecPostHookParser(root=root, prefix=self._prefix),
    )
    with open(self._path) as f:
      for lineno, line in enumerate(f, 1):
        try:
          self._add_line(line)
        except entry.ParserError as e:
          sys.exit(f'{self}:{lineno}: {e}')

  def __str__(self) -> str:
    return util.format_path(self._path)

  def _add_line(self, line) -> None:
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
    parser = self._parsers.parse(command, args)
    self._entries.append(parser)

  def system_setup(self) -> None:
    for entry in self._entries:
      entry.system_setup()

  def install(self, record: entry.PathRecorder) -> None:
    for entry in self._entries:
      entry.install(record)

  def post_install(self) -> None:
    for entry in self._entries:
      entry.post_install()


class ManifestParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['subdir']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    return Manifest(self.root / path, self.prefix.current / path)
