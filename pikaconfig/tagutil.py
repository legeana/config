import dataclasses
import platform
import sys
from typing import Iterable

from . import system


TagSet = set[str]


@dataclasses.dataclass
class Matcher:

  prerequisites: set[str] = dataclasses.field(default_factory=set)
  conflicts: set[str] = dataclasses.field(default_factory=set)

  def match(self, profiles: TagSet) -> bool:
    return self._match_prerequisites(profiles) and self._match_conflicts(profiles)

  def _match_prerequisites(self, tags: TagSet) -> bool:
    for required in self.prerequisites:
      if required not in tags:
        return False
    return True

  def _match_conflicts(self, tags: TagSet) -> bool:
    for tag in tags:
      if tag in self.conflicts:
        return False
    return True


def _system_profiles() -> Iterable[str]:
  yield f'hostname={platform.node()}'
  yield f'os={platform.system()}'
  yield f'os-release.id={system.os_id()}'


def system_tags() -> TagSet:
  return set(_system_profiles())