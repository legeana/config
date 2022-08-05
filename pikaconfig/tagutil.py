import dataclasses
import json
import pathlib
import platform
import sys
from typing import Iterable

from . import system


TagSet = set[str]


@dataclasses.dataclass
class Matcher:

  prerequisites: TagSet = dataclasses.field(default_factory=TagSet)
  conflicts: TagSet = dataclasses.field(default_factory=TagSet)

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

  def supported_tags(self) -> TagSet:
    return self.prerequisites.union(self.conflicts)


def _system_profiles() -> Iterable[str]:
  yield f'hostname={platform.node()}'
  yield f'os={platform.system()}'
  yield f'os-release.id={system.os_id()}'


def system_tags() -> TagSet:
  return set(_system_profiles())


def local_tags(path: pathlib.Path) -> TagSet:
  try:
    with open(path) as f:
      data = json.load(f)
  except FileNotFoundError:
    data = []
  if not isinstance(data, list):
    raise ValueError(f'{path}: invalid format, expected JSON list, '
                     f'found {type(data)}')
  for i, v in enumerate(data):
    if not isinstance(v, str):
      raise ValueError(f'{path}: invalid format, expected JSON list[str], '
                       f'found {type(v)} at index {i}')
  return TagSet(data)


def save_local_tags(path: pathlib.Path, tags: TagSet) -> None:
  with open(path, 'w') as f:
    json.dump(list(tags), f)
