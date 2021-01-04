import dataclasses
import shlex
import sys

_SYSTEM_PATH = '/etc/os-release'
_SYSTEM_OS_RELEASE = None


@dataclasses.dataclass
class OsRelease:

  pretty_name: str = ''
  name: str = ''
  id: str = ''
  version_codename: str = ''
  version_id: str = ''
  home_url: str = ''
  support_url: str = ''
  bug_report_url: str = ''

  def __init__(self, fd):
    for line in fd:
      self._parse_line(line)

  def _parse_line(self, line):
    line = line.strip()
    eq = line.find('=')
    if eq == -1:
      return
    key = line[:eq].strip().lower()
    value = shlex.split(line[eq + 1:])
    if not value:
      return
    if hasattr(self, key):
      setattr(self, key, value[0])

  @classmethod
  def from_file(cls, src) -> 'OsRelease':
    with open(src) as f:
      return OsRelease(f)

  @classmethod
  def from_etc(cls) -> 'OsRelease':
    global _SYSTEM_OS_RELEASE
    if _SYSTEM_OS_RELEASE is None:
      _SYSTEM_OS_RELEASE = cls.from_file(_SYSTEM_PATH)
    return _SYSTEM_OS_RELEASE


def os_id() -> str:
  if sys.platform.startswith('darwin'):
    return 'darwin'
  return OsRelease.from_etc().id
