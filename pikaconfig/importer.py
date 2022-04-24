import enum
import os
import pathlib
from typing import Iterable, TextIO


@enum.unique
class Keywords(enum.Enum):
  IMPORT = '#import'
  IMPORT_TREE = '#import_tree'

  @classmethod
  def all(cls) -> Iterable[str]:
    # reverse order by length is important so that we match the longest keyword
    return sorted(cls, key=lambda v: len(v.value), reverse=True)


def _render(*, prefix: pathlib.Path, src: pathlib.Path, out: TextIO) -> None:
  with open(src, 'rt') as tmpl:
    for line in tmpl:
      keyword = None
      arg = None
      for kw in Keywords.all():
        if line.startswith(kw.value):
          keyword = kw
          arg = line[len(kw.value):].strip()
          break
      if keyword is None:
        out.write(line)
      elif keyword is Keywords.IMPORT:
        include_file = prefix / pathlib.Path(arg)
        _render(prefix=include_file.parent, src=include_file, out=out)
      elif keyword is Keywords.IMPORT_TREE:
        include_dir = prefix / pathlib.Path(arg)
        for _root, _, files in os.walk(include_dir):
          root = pathlib.Path(_root)
          for f in files:
            _render(prefix=root, src=root / f, out=out)
      else:
        raise Exception(f'Logic error: unknown keyword {keyword!r}')


def render(*, src: pathlib.Path, dst: pathlib.Path) -> None:
    with open(dst, 'wt') as out:
      _render(prefix=dst.parent, src=src, out=out)
