import dataclasses
import logging
import os
import pathlib
import shutil
from typing import Collection, Iterable

from . import entry
from . import importer
from . import local_state
from . import util


def _make_symlink(src: pathlib.Path, dst: pathlib.Path,
                  record: entry.PathRecorder) -> None:
  if dst.exists():
    if not dst.is_symlink():
      logging.error(f'Symlink: unable to overwrite {util.format_path(dst)} '
                    f'by {util.format_path(src)}: destination is not a symlink')
      return
    dst.unlink()
  try:
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.symlink_to(src)
  except Exception as e:
    logging.exception(f'Unable to create a symlink to {util.format_path(dst)} '
                      f'from {util.format_path(src)}')
    return
  record(dst)
  logging.info(f'{util.format_path(src)} -> {util.format_path(dst)}')


@dataclasses.dataclass
class SinglePathParser(entry.Parser):

  root: pathlib.Path
  prefix: entry.Prefix

  def parse(self, command: str, args: list[str]) -> entry.Entry:
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
    _make_symlink(src=self.src, dst=self.dst, record=record)


class SymlinkParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['symlink']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    return SymlinkEntry(self.root / path, self.prefix.current / path)


@dataclasses.dataclass
class SymlinkTreeEntry(FileEntry):

  def install(self, record: entry.PathRecorder) -> None:
    for root, _, files in os.walk(self.src):
      relroot = pathlib.Path(root).relative_to(self.src)
      srcroot = self.src / relroot
      dstroot = self.dst / relroot
      for f in files:
        _make_symlink(src=srcroot / f, dst=dstroot / f, record=record)


class SymlinkTreeParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['symlink_tree']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    return SymlinkTreeEntry(self.root / path, self.prefix.current / path)


class MkDirEntry(FileEntry):

  def install(self, record: entry.PathRecorder) -> None:
    if self.dst.exists() and not self.dst.is_dir():
      logging.error(f'mkdir: unable to overwrite {util.format_path(self.dst)}')
      return
    self.dst.mkdir(parents=True, exist_ok=True)
    record(self.dst)


class MkDirParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['mkdir']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    return MkDirEntry(self.root / path, self.prefix.current / path)


class CopyEntry(FileEntry):

  def install(self, record: entry.PathRecorder) -> None:
    state = local_state.make_state(self.dst)
    _make_symlink(src=state, dst=self.dst, record=record)
    if state.exists():
      logging.info(f'Copy: skipping already existing state '
                   f'for {util.format_path(self.dst)}')
      return
    shutil.copy2(self.src, state)
    logging.info(f'{util.format_path(self.src)} -> {util.format_path(state)}')


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
    _make_symlink(src=src, dst=self.dst, record=record)


class OutputFileParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['output_file']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    return OutputFileEntry(self.prefix.current / path)


def _glob(base: pathlib.Path, glob: str) -> Iterable[pathlib.Path]:
  # This workaround is required because pathlib.Path.glob
  # does not support absolute globs.
  # NotImplementedError: Non-relative patterns are unsupported
  glob_path = pathlib.Path(glob)
  if glob_path.is_absolute():
    anchor = pathlib.Path(glob_path.anchor)
    relative_glob = str(glob_path.relative_to(anchor))
    return anchor.glob(relative_glob)
  else:
    return base.glob(glob)


@dataclasses.dataclass
class CatGlobEntry(OutputFileEntry):

  prefix: pathlib.Path
  globs: list[str]

  def install(self, record: entry.PathRecorder) -> None:
    super().install(record)

  def post_install(self) -> None:
    if not self.dst.is_symlink():
      logging.error(f'{util.format_path(self.dst)} is not a symlink')
      return
    inputs: list[pathlib.Path] = []
    with self.dst.open('w') as out:
      for glob in self.globs:
        for src in sorted(_glob(self.prefix, glob)):
          inputs.append(src)
          with open(src) as inp:
            shutil.copyfileobj(inp, out)
    inputs_formatted = ', '.join(util.format_path(p) for p in inputs)
    logging.info(f'{util.format_path(self.dst)} <- [{inputs_formatted}]')


@dataclasses.dataclass
class CatGlobParser(entry.Parser):

  root: pathlib.Path
  prefix: entry.Prefix

  @property
  def supported_commands(self) -> Collection[str]:
    return ['cat_glob_into']

  def parse(self, command: str, args: list[str]) -> entry.Entry:
    self.check_supported(command)
    if len(args) < 1:
      raise entry.ParserError(
          f'{command} requires at least 1 argument, got {len(args)}')
    return CatGlobEntry(dst=self.prefix.current / args[0],
                        prefix=self.prefix.current,
                        globs=args[1:])


class ImporterEntry(FileEntry):

  def install(self, record: entry.PathRecorder) -> None:
    state = local_state.make_state(self.dst)
    _make_symlink(src=state, dst=self.dst, record=record)

  def post_install(self) -> None:
    if not self.dst.is_symlink():
      logging.error(f'{util.format_path(self.dst)} is not a symlink')
      return
    importer.render(src=self.src, dst=self.dst)
    logging.info(f'{util.format_path(self.dst)} <- {util.format_path(self.src)}')


class ImporterParser(SinglePathParser):

  @property
  def supported_commands(self) -> Collection[str]:
    return ['import_from']

  def parse_single_path(self, command: str, path: pathlib.Path) -> entry.Entry:
    del command  # unused
    return ImporterEntry(self.root / path, self.prefix.current / path)
