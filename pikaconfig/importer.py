import pathlib
from typing import TextIO

KEYWORD = '#import'


def _render(*, prefix: pathlib.Path, src: pathlib.Path,
            out: TextIO, keyword: str) -> None:
  with open(src, 'rt') as tmpl:
    for line in tmpl:
      if not line.startswith(keyword):
        out.write(line)
        continue
      include = line[len(keyword):].strip()
      include_file = prefix / pathlib.Path(include)
      _render(prefix=include_file.parent, src=include_file,
              out=out, keyword=keyword)


def render(*, src: pathlib.Path, dst: pathlib.Path,
           keyword: str = KEYWORD) -> None:
    with open(dst, 'wt') as out:
      _render(prefix=dst.parent, src=src, out=out, keyword=keyword)
