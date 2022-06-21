import dataclasses
import pathlib
import shlex
import sys
from typing import Collection

from . import entry
from . import file_entry
from . import post_install_hook
from . import system_entry
from . import tagutil
from . import util
from . import xdg

_MANIFEST_FILENAME: str = 'MANIFEST'


def is_overlay(path: pathlib.Path) -> bool:
  return (path / _MANIFEST_FILENAME).is_file()


class CombinedParser(entry.Parser):

  def __init__(self, *parsers):
    self._parsers: dict[str, Parser] = dict()
    for parser in parsers:
      for command in parser.supported_commands:
        assert command not in self._parsers
        self._parsers[command] = parser

  @property
  def supported_commands(self) -> Collection[str]:
    return self._parsers.keys()

  def parse(self, command: str, args: list[str]) -> entry.Entry:
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


class TagsEntry(entry.Entry):

  def install(self, record: entry.PathRecorder) -> None:
    del record  # unused
    # noop


class TagsParser(entry.Parser):

  def __init__(self, matcher: tagutil.Matcher):
    self.matcher = matcher

  def parse(self, command: str, args: list[str]) -> entry.Entry:
    self.check_supported(command)
    return self.parse_tags(tagutil.TagSet(args))

  def parse_tags(self, tags: tagutil.TagSet) -> entry.Entry:
    raise NotImplementedError


class PrerequisiteTagsParser(TagsParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['requires']

  def parse_tags(self, tags: tagutil.TagSet) -> entry.Entry:
    self.matcher.prerequisites.update(tags)
    return TagsEntry()


class ConflictTagsParser(TagsParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['conflicts']

  def parse_tags(self, tags: tagutil.TagSet) -> entry.Entry:
    self.matcher.conflicts.update(tags)
    return TagsEntry()


class Manifest(entry.Entry):

  def __init__(self, root: pathlib.Path, prefix: pathlib.Path):
    self._entries: list[entry.Entry] = []
    self._path = root / _MANIFEST_FILENAME
    self._prefix = entry.Prefix(prefix)
    self._tag_matcher = tagutil.Matcher()
    self._parsers = CombinedParser(
        SubdirParser(root=root, prefix=self._prefix),
        SubdirsParser(root=root, prefix=self._prefix),
        SetPrefixParser(root=root, prefix=self._prefix),
        SetXdgCachePrefixParser(root=root, prefix=self._prefix),
        SetXdgConfigPrefixParser(root=root, prefix=self._prefix),
        SetXdgDataPrefixParser(root=root, prefix=self._prefix),
        SetXdgStatePrefixParser(root=root, prefix=self._prefix),
        PrerequisiteTagsParser(matcher=self._tag_matcher),
        ConflictTagsParser(matcher=self._tag_matcher),
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
    self._add_command(parts[0], parts[1:])

  def _add_command(self, command: str, args: list[str]) -> None:
    parser = self._parsers.parse(command, args)
    self._entries.append(parser)

  def tags_match(self, tags: tagutil.TagSet) -> bool:
    return self._tag_matcher.match(tags)

  def recursive_system_setup(self, tags: tagutil.TagSet) -> None:
    if not self.tags_match(tags):
      return
    for entry in self._entries:
      entry.recursive_system_setup(tags)

  def recursive_install(self, tags: tagutil.TagSet,
                        record: entry.PathRecorder) -> None:
    if not self.tags_match(tags):
      return
    for entry in self._entries:
      entry.recursive_install(tags, record)

  def recursive_post_install(self, tags: tagutil.TagSet) -> None:
    if not self.tags_match(tags):
      return
    for entry in self._entries:
      entry.recursive_post_install(tags)


@dataclasses.dataclass
class CombinedManifest(entry.Entry):

  manifests: list[Manifest]

  def recursive_system_setup(self, tags: tagutil.TagSet) -> None:
    if not self.tags_match(tags):
      return
    for entry in self.manifests:
      entry.recursive_system_setup(tags)

  def recursive_install(self, tags: tagutil.TagSet,
                        record: entry.PathRecorder) -> None:
    if not self.tags_match(tags):
      return
    for entry in self.manifests:
      entry.recursive_install(tags, record)

  def recursive_post_install(self, tags: tagutil.TagSet) -> None:
    if not self.tags_match(tags):
      return
    for entry in self.manifests:
      entry.recursive_post_install(tags)


class SubdirParser(file_entry.SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['subdir']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    return Manifest(self.root / path, self.prefix.current / path)


@dataclasses.dataclass
class SubdirsParser(entry.Parser):

  root: pathlib.Path
  prefix: entry.Prefix

  @property
  def supported_commands(self) -> Collection[str]:
    return ['subdirs']

  def parse(self, command: str, args: list[str]) -> entry.Entry:
    del args  # unused
    self.check_supported(command)
    manifests: list[Manifest] = []
    for subdir in sorted(self.root.iterdir()):
      if not subdir.is_dir():
        continue
      manifests.append(Manifest(self.root / subdir, self.prefix.current / subdir))
    return CombinedManifest(manifests)
