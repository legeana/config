import pathlib


def unexpanduser(path: pathlib.Path) -> pathlib.Path:
  try:
    home = pathlib.Path.home()
  except Exception:
    return path
  if home in path.parents:
    return pathlib.Path('~') / path.relative_to(home)
  return path


def format_path(path: pathlib.Path) -> str:
  return str(unexpanduser(path))
