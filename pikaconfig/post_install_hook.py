import dataclasses
import pathlib
from typing import Collection, List

from . import entry
from . import util


class PostInstallHook(entry.Entry):

  def install(self, record: entry.PathRecorder) -> None:
    del record  # unused

  def post_install(self) -> None:
    raise NotImplementedError


@dataclasses.dataclass
class ExecPostHook(PostInstallHook):

  cwd: pathlib.Path
  args: List[str]

  def post_install(self) -> None:
    util.verbose_check_call(*self.args, cwd=self.cwd)


@dataclasses.dataclass
class ExecPostHookParser(entry.Parser):

  root: pathlib.Path
  prefix: entry.Prefix

  @property
  def supported_commands(self) -> Collection[str]:
    return ['post_install_exec']

  def parse(self, command: str, args: List[str]) -> entry.Entry:
    self.check_supported(command)
    return ExecPostHook(cwd=self.prefix.current, args=args)
