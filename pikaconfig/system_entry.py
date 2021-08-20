import dataclasses
from typing import Collection, List, Type

from . import entry
from . import system
from . import util


class SystemSetupEntry(entry.Entry):

  DISTROS: List[str] = []

  def __init__(self, args: List[str]):
    raise NotImplementedError

  def system_setup(self) -> None:
    raise NotImplementedError

  def install(self, record: entry.PathRecorder) -> None:
    pass


@dataclasses.dataclass
class SystemCommandEntry(SystemSetupEntry):

  args: List[str]

  def system_setup(self) -> None:
    # TODO implement a confirmation
    util.verbose_check_user_call('sudo', *self.args)


class SystemCommandParser(entry.Parser):

  def parse(self, command: str, args: List[str]) -> entry.Entry:
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


class AnyPackageParser(entry.Parser):

  @property
  def _package_managers(self) -> List[Type[SystemSetupEntry]]:
    return [
        PacmanPackageEntry,
        AptPackageEntry,
        BrewPackageEntry,
    ]

  def parse(self, command: str, args: List[str]) -> entry.Entry:
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
    util.verbose_check_call('sudo', 'pacman', '-S', '--', *self.names)


class PacmanPackageParser(entry.Parser):

  def parse(self, command: str, args: List[str]) -> entry.Entry:
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
    util.verbose_check_call('sudo', 'apt', 'install', '--', *self.names)


class AptPackageParser(entry.Parser):

  def parse(self, command: str, args: List[str]) -> entry.Entry:
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
    util.verbose_check_call('brew', 'install', '--', *self.names)


class BrewPackageParser(entry.Parser):

  def parse(self, command: str, args: List[str]) -> entry.Entry:
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
    util.verbose_check_call('pip', 'install', '--user', '--', *self.names)


class PipPackageParser(entry.Parser):

  def parse(self, command: str, args: List[str]) -> entry.Entry:
    self.check_supported(command)
    return PipPackageEntry(args)

  @property
  def supported_commands(self) -> Collection[str]:
    return ['install_pip_user_package']
