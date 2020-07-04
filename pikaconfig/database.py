import pathlib


class InstalledDatabase:

  def __init__(self):
    self._entries = []

  @classmethod
  def load_from(cls, path):
    inst = cls()
    try:
      with open(path) as f:
        for line in f:
          inst._add_line(line.strip())
    except FileNotFoundError:
      pass
    return inst

  def _add_line(self, line):
    self.add(line)

  def write(self, fp):
    for item in sorted(self._entries):
      self.write_line(fp, item)

  def write_file(self, path):
    with open(path, 'w') as f:
      self.write(f)

  @classmethod
  def write_item(cls, fp, item) -> None:
    fp.write(str(item))
    fp.write('\n')

  def add(self, item: pathlib.Path) -> None:
    self._entries.append(str(item))

  def clear(self) -> None:
    self._entries.clear()

  def __contains__(self, item):
    return str(item) in self._entries

  def __iter__(self):
    return map(pathlib.Path, self._entries)

  def __reversed__(self):
    return map(pathlib.Path, reversed(self._entries))

  def __bool__(self):
    return bool(self._entries)


class SyncInstalledDatabase(InstalledDatabase):

  def __init__(self, path: pathlib.Path):
    super().__init__()
    self._path = path

  def add(self, item: pathlib.Path) -> None:
    if item in self:
      return
    mode = 'a' if self else 'w'
    super().add(item)
    with open(self._path, mode) as f:
      self.write_item(f, item)
