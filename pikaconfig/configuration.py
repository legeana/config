import dataclasses
import pathlib
import shlex
import sys
from typing import Collection, Dict, List

from . import entry
from . import file_entry
from . import post_install_hook
from . import system_entry
from . import util
from . import xdg

_MANIFEST_FILENAME: str = 'MANIFEST'


def is_overlay(path: pathlib.Path) -> bool:
  return (path / _MANIFEST_FILENAME).is_file()


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
class SetPrefixEntry(entry.Entry):

  prefix: pathlib.Path

  def install(self, record: entry.PathRecorder) -> None:
    del record  # unused
    # noop


class SetPrefixParser(file_entry.SinglePathParser):

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


class SetXdgCachePrefixParser(file_entry.SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['xdg_cache_prefix']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    self.prefix.current = xdg.CACHE_HOME / path
    return SetPrefixEntry(prefix=path)


class SetXdgConfigPrefixParser(file_entry.SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['xdg_config_prefix']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    self.prefix.current = xdg.CONFIG_HOME / path
    return SetPrefixEntry(prefix=path)


class SetXdgDataPrefixParser(file_entry.SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['xdg_data_prefix']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    self.prefix.current = xdg.DATA_HOME / path
    return SetPrefixEntry(prefix=path)


class SetXdgStatePrefixParser(file_entry.SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['xdg_state_prefix']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    self.prefix.current = xdg.STATE_HOME / path
    return SetPrefixEntry(prefix=path)


class Manifest(entry.Entry):

  def __init__(self, root: pathlib.Path, prefix: pathlib.Path):
    self._entries: List[entry.Entry] = []
    self._path = root / _MANIFEST_FILENAME
    self._prefix = entry.Prefix(prefix)
    self._parsers = CombinedParser(
        ManifestParser(root=root, prefix=self._prefix),
        SetPrefixParser(root=root, prefix=self._prefix),
        SetXdgCachePrefixParser(root=root, prefix=self._prefix),
        SetXdgConfigPrefixParser(root=root, prefix=self._prefix),
        SetXdgDataPrefixParser(root=root, prefix=self._prefix),
        SetXdgStatePrefixParser(root=root, prefix=self._prefix),
        system_entry.SystemCommandParser(),
        system_entry.AnyPackageParser(),
        system_entry.PacmanPackageParser(),
        system_entry.AptPackageParser(),
        system_entry.BrewPackageParser(),
        system_entry.PipPackageParser(),
        file_entry.SymlinkParser(root=root, prefix=self._prefix),
        file_entry.SymlinkTreeParser(root=root, prefix=self._prefix),
        file_entry.MkDirParser(root=root, prefix=self._prefix),
        file_entry.CopyParser(root=root, prefix=self._prefix),
        file_entry.OutputFileParser(root=root, prefix=self._prefix),
        file_entry.CatGlobParser(root=root, prefix=self._prefix),
        file_entry.ImporterParser(root=root, prefix=self._prefix),
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


class ManifestParser(file_entry.SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['subdir']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    return Manifest(self.root / path, self.prefix.current / path)
