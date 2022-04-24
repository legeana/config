import pathlib
from typing import TextIO

IMPORT = '#import'


def _render(*, prefix: pathlib.Path, src: pathlib.Path, out: TextIO) -> None:
  with open(src, 'rt') as tmpl:
    for line in tmpl:
      if not line.startswith(IMPORT):
        out.write(line)
        continue
      include = line[len(IMPORT):].strip()
      include_file = prefix / pathlib.Path(include)
      _render(prefix=include_file.parent, src=include_file, out=out)


def render(*, src: pathlib.Path, dst: pathlib.Path) -> None:
    with open(dst, 'wt') as out:
      _render(prefix=dst.parent, src=src, out=out)
