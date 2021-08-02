import dataclasses
import logging
import pathlib
import shlex
import shutil
import subprocess
import sys
from typing import Callable, Collection, Dict, List, Optional, Type

from . import system
from . import util

PathRecorder = Callable[[pathlib.Path], None]


def _verbose_check_call(*args, cwd: Optional[pathlib.Path] = None) -> None:
  pwd = '' if cwd is None else f'[{util.format_path(cwd)}] '
  command = f'$ {" ".join(shlex.quote(arg) for arg in args)}'
  logging.info(pwd + command)
  subprocess.check_call(args, cwd=cwd)


class Prefix:

  def __init__(self, prefix: pathlib.Path):
    self._prefix = prefix

  def set(self, prefix: pathlib.Path) -> None:
    self._prefix = prefix

  def get(self) -> pathlib.Path:
    return self._prefix


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

  DISTROS: List[str] = []

  def __init__(self, args: List[str]):
    raise NotImplementedError

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


class CombinedParser(Parser):

  def __init__(self, *parsers):
    self._parsers: Dict[str, Parser] = dict()
    for parser in parsers:
      for command in parser.supported_commands:
        assert command not in self._parsers
        self._parsers[command] = parser

  @property
  def supported_commands(self) -> Collection[str]:
    return self._parsers.keys()

  def parse(self, command: str, args: List[str]) -> Entry:
    parser = self._parsers.get(command)
    if parser is None:
      raise ParserError(f'{command} is not supported by {type(self)}')
    return parser.parse(command, args)


@dataclasses.dataclass
class SinglePathParser(Parser):

  root: pathlib.Path
  prefix: Prefix

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

  @property
  def _package_managers(self) -> List[Type[SystemSetupEntry]]:
    return [
        PacmanPackageEntry,
        AptPackageEntry,
        BrewPackageEntry,
    ]

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    sysid = system.os_id()
    entries = [
        cls(args) for cls in self._package_managers
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
    if system.os_id() not in self.DISTROS:
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
    if system.os_id() not in self.DISTROS:
      return
    _verbose_check_call('sudo', 'apt', 'install', '--', *self.names)


class AptPackageParser(Parser):

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    return AptPackageEntry(args)

  @property
  def supported_commands(self) -> Collection[str]:
    return ['install_apt_package']


@dataclasses.dataclass
class BrewPackageEntry(SystemSetupEntry):

  DISTROS = ['darwin']
  names: List[str]

  def system_setup(self) -> None:
    if system.os_id() not in self.DISTROS:
      return
    _verbose_check_call('brew', 'install', '--', *self.names)


class BrewPackageParser(Parser):

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    return BrewPackageEntry(args)

  @property
  def supported_commands(self) -> Collection[str]:
    return ['install_brew_package']


@dataclasses.dataclass
class PipPackageEntry(SystemSetupEntry):

  names: List[str]

  def system_setup(self) -> None:
    # system check is not necessary because pip is cross platform
    _verbose_check_call('pip', 'install', '--user', '--', *self.names)


class PipPackageParser(Parser):

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    return PipPackageEntry(args)

  @property
  def supported_commands(self) -> Collection[str]:
    return ['install_pip_user_package']


class SymlinkEntry(FileEntry):

  def install(self, record: PathRecorder) -> None:
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

  def parse_single_path(self, command: str, path: pathlib.Path) -> Entry:
    del command  # unused
    return SymlinkEntry(self.root / path, self.prefix.get() / path)


class CopyEntry(FileEntry):

  def install(self, record: PathRecorder) -> None:
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

  def parse_single_path(self, command: str, path: pathlib.Path) -> Entry:
    del command  # unused
    return CopyEntry(self.root / path, self.prefix.get() / path)


@dataclasses.dataclass
class ExecPostHook(PostInstallHook):

  cwd: pathlib.Path
  args: List[str]

  def post_install(self) -> None:
    _verbose_check_call(*self.args, cwd=self.cwd)


@dataclasses.dataclass
class ExecPostHookParser(Parser):

  root: pathlib.Path
  prefix: Prefix

  @property
  def supported_commands(self) -> Collection[str]:
    return ['post_install_exec']

  def parse(self, command: str, args: List[str]) -> Entry:
    self.check_supported(command)
    return ExecPostHook(cwd=self.prefix.get(), args=args)


@dataclasses.dataclass
class SetPrefixEntry(Entry):

  prefix: pathlib.Path

  def install(self, record: PathRecorder) -> None:
    del record  # unused
    # noop


class SetPrefixParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['prefix']

  def parse_single_path(self, command: str, path: pathlib.Path) -> Entry:
    del command  # unused
    self.prefix.set(self.prefix.get() / path.expanduser())
    return SetPrefixEntry(prefix=path)


class Manifest(Entry):

  def __init__(self, root: pathlib.Path, prefix: pathlib.Path):
    self._entries: List[Entry] = []
    self._path = root / 'MANIFEST'
    self._prefix = Prefix(prefix)
    self._parsers = CombinedParser(
        ManifestParser(root=root, prefix=self._prefix),
        SetPrefixParser(root=root, prefix=self._prefix),
        SystemCommandParser(),
        AnyPackageParser(),
        PacmanPackageParser(),
        AptPackageParser(),
        BrewPackageParser(),
        PipPackageParser(),
        SymlinkParser(root=root, prefix=self._prefix),
        CopyParser(root=root, prefix=self._prefix),
        ExecPostHookParser(root=root, prefix=self._prefix),
    )
    with open(self._path) as f:
      for lineno, line in enumerate(f, 1):
        try:
          self._add_line(line)
        except ParserError as e:
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
    return Manifest(self.root / path, self.prefix.get() / path)
