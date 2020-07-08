import dataclasses
import logging
import pathlib
import shlex
import shutil
import subprocess
import sys
from typing import Callable, Collection, Dict, List

from . import system

PathRecorder = Callable[[pathlib.Path], None]


def _verbose_check_call(*args) -> None:
  logging.info(f'$ {" ".join(shlex.quote(arg) for arg in args)}')
  subprocess.check_call(args)


class ParserError(Exception):
  pass


class Entry:

  def system_setup(self) -> None:
    pass

  def install(self, record: PathRecorder) -> None:
    raise NotImplementedError

  def post_install(self) -> None:
    """Post install hook."""
    pass


class SystemSetupEntry(Entry):

  def system_setup(self) -> None:
    raise NotImplementedError

  def install(self, record: PathRecorder) -> None:
    pass


class PostInstallHook(Entry):

  def install(self, record: PathRecorder) -> None:
    del record  # unused

  def post_install(self) -> None:
    raise NotImplementedError


class Parser:

  @property
  def supported_commands(self) -> Collection[str]:
    raise NotImplementedError

  def check_supported(self, command: str) -> None:
    if command not in self.supported_commands:
      raise ParserError(f'{command} is not supported by {type(self)}')

  def parse(self, command: str, args: List[str]) -> Entry:
    raise NotImplementedError


@dataclasses.dataclass
class SinglePathParser(Parser):

  root: pathlib.Path
  prefix: pathlib.Path

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    if len(args) != 1:
      raise ParserError(f'{command} supports only 1 argument, got {len(args)}')
    return self.parse_single_path(command, pathlib.Path(args[0]))

  def parse_single_path(self, command: str, path: pathlib.Path) -> Entry:
    raise NotImplementedError


@dataclasses.dataclass
class FileEntry(Entry):

  src: pathlib.Path
  dst: pathlib.Path


@dataclasses.dataclass
class SystemCommandEntry(SystemSetupEntry):

  args: List[str]

  def system_setup(self) -> None:
    # TODO implement a confirmation
    _verbose_check_call('sudo', *self.args)


class SystemCommandParser(Parser):

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    return SystemCommandEntry(args=args)

  @property
  def supported_commands(self) -> Collection[str]:
    return ['sudo']


class AnyPackageEntry(SystemSetupEntry):

  def __init__(self, entries):
    self._entries = entries

  def system_setup(self) -> None:
    for entry in self._entries:
      entry.system_setup()


class AnyPackageParser(Parser):

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    sysid = system.OsRelease.from_etc().id
    entries = [
        cls(args) for cls in [PacmanPackageEntry, AptPackageEntry]
        if sysid in cls.DISTROS
    ]
    return AnyPackageEntry(entries)

  @property
  def supported_commands(self) -> Collection[str]:
    return ['install_system_package']


@dataclasses.dataclass
class PacmanPackageEntry(SystemSetupEntry):

  DISTROS = ['arch']
  names: List[str]

  def system_setup(self) -> None:
    if system.OsRelease.from_etc().id not in self.DISTROS:
      return
    _verbose_check_call('sudo', 'pacman', '-S', '--', *self.names)


class PacmanPackageParser(Parser):

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    return PacmanPackageEntry(args)

  @property
  def supported_commands(self) -> Collection[str]:
    return ['install_pacman_package']


@dataclasses.dataclass
class AptPackageEntry(SystemSetupEntry):

  DISTROS = ['debian', 'ubuntu']
  names: List[str]

  def system_setup(self) -> None:
    if system.OsRelease.from_etc().id not in self.DISTROS:
      return
    _verbose_check_call('sudo', 'apt', 'install', '--', *self.names)


class AptPackageParser(Parser):

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    return AptPackageEntry(args)

  @property
  def supported_commands(self) -> Collection[str]:
    return ['install_apt_package']


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


class SymlinkParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['symlink']

  def parse_single_path(self, command: str, path: pathlib.Path) -> Entry:
    del command  # unused
    return SymlinkEntry(self.root / path, self.prefix / path)


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


class CopyParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['copy']

  def parse_single_path(self, command: str, path: pathlib.Path) -> Entry:
    del command  # unused
    return CopyEntry(self.root / path, self.prefix / path)


@dataclasses.dataclass
class ExecPostHook(PostInstallHook):

  args: List[str]

  def post_install(self) -> None:
    _verbose_check_call(*self.args)


class ExecPostHookParser(Parser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['post_install_exec']

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    return ExecPostHook(args)


class Manifest(Entry):

  def __init__(self, root: pathlib.Path, prefix: pathlib.Path = pathlib.Path()):
    self._entries: List[Entry] = []
    self._path = root / 'MANIFEST'
    self._register_parsers(
        ManifestParser(root=root, prefix=prefix),
        SystemCommandParser(),
        AnyPackageParser(),
        PacmanPackageParser(),
        AptPackageParser(),
        SymlinkParser(root=root, prefix=prefix),
        CopyParser(root=root, prefix=prefix),
        ExecPostHookParser(),
    )
    with open(self._path) as f:
      for lineno, line in enumerate(f, 1):
        try:
          self._add_line(line)
        except ParserError as e:
          sys.exit(f'{self}:{lineno}: {e}')

  def __str__(self) -> str:
    return str(self._path)

  def _register_parsers(self, *parsers: Parser) -> None:
    self._parsers: Dict[str, Parser] = dict()
    for parser in parsers:
      for command in parser.supported_commands:
        assert command not in self._parsers
        self._parsers[command] = parser

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
    parser = self._parsers.get(command)
    if parser is None:
      raise ParserError(f'{command} is not supported by {type(self)}')
    self._entries.append(parser.parse(command, args))

  def system_setup(self) -> None:
    for entry in self._entries:
      entry.system_setup()

  def install(self, record: PathRecorder) -> None:
    for entry in self._entries:
      entry.install(record)

  def post_install(self) -> None:
    for entry in self._entries:
      entry.post_install()


class ManifestParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['subdir']

  def parse_single_path(self, command: str, path: pathlib.Path) -> Entry:
    del command  # unused
    return Manifest(self.root / path, self.prefix / path)
