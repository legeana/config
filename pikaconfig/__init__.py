import argparse
import asyncio
import os
import pathlib
import shlex
import subprocess
import sys

from . import configuration
from . import database

SELF = pathlib.Path(sys.argv[0]).absolute()
ROOT = SELF.parent
OVERLAYS = ROOT / 'overlay.d'
INSTALL = ROOT / '.install'
BASE = ROOT / 'base'


def log(*args, **kwargs):
  print(*args, **kwargs, file=sys.stderr)


async def run(*args, **kwargs) -> subprocess.CompletedProcess:
  process = await asyncio.create_subprocess_exec(*args, **kwargs)
  stdout, stderr = await process.communicate()
  retcode = await process.wait()
  return subprocess.CompletedProcess(args, retcode, stdout, stderr)


async def check_output(*args, **kwargs) -> bytes:
  kwargs['stdout'] = subprocess.PIPE
  kwargs['stderr'] = subprocess.STDOUT
  ret = await run(*args, **kwargs)
  ret.check_returncode()
  return ret.stdout


async def update(plugin: pathlib.Path) -> bool:
  gitdir = plugin / '.git'
  if not gitdir.is_dir():
    return False
  ref = gitdir / 'refs' / 'heads' / 'master'
  with open(ref) as f:
    old_ref = f.read()
  args = ['git', '-C', str(plugin), 'pull', '--ff-only']
  try:
    output = await check_output(*args)
    log(f'Updated {str(plugin)}: {output.decode().strip()}')
  except subprocess.CalledProcessError as e:
    # TODO use shlex.join(), python-3.8+
    command = ' '.join(shlex.quote(arg) for arg in args)
    sys.exit(f'Failed to update {str(plugin)}, manual intervention is required!\n'
             f'$ {command}\n'
             f'{e.stdout.decode().strip()}')
  with open(ref) as f:
    new_ref = f.read()
  return old_ref != new_ref


async def update_all() -> bool:
  log('Updating...')
  updates = [update(ROOT)] + [
      update(overlay) for overlay in OVERLAYS.iterdir()
      if overlay.is_dir()
  ]
  update_results = await asyncio.gather(*updates, return_exceptions=False)
  log()
  assert len(updates) == len(update_results)
  for res in update_results:
    if isinstance(res, BaseException):
      raise res
  return update_results[0]


class Installer:

  def __init__(self):
    self._old_db = database.InstalledDatabase.load_from(INSTALL)
    self._db = database.SyncInstalledDatabase(INSTALL)
    self._manifests = None  # lazy loading

  def uninstall(self):
    for link in reversed(self._old_db):
      if not link.is_symlink():
        log(f'Unable to remove {str(link)}')
        continue
      try:
        link.unlink()
        log(f'Removed symlink {str(link)}')
      except FileNotFoundError:
        # TODO missing_ok=True, python-3.8+
        pass
      for parent in link.parents:
        try:
          parent.rmdir()
          log(f'Removed empty directory {str(parent)}')
        except OSError:
          break
    try:
      INSTALL.unlink()
      # TODO missing_ok=True, python-3.8+
    except FileNotFoundError:
      pass

  def _load_manifests(self):
    if self._manifests is not None:
      return self._manifests
    self._manifests = []
    for path in [BASE] + sorted(OVERLAYS.iterdir()):
      if not path.is_dir():
        continue
      try:
        self._manifests.append(configuration.Manifest(path, pathlib.Path.home()))
      except FileNotFoundError as e:
        log(f'Unable to load MANIFEST in {str(path)}: {e}')
        continue
    return self._manifests

  def install(self):
    for manifest in self._load_manifests():
      log(f'\nInstalling from {str(manifest.root)}')
      manifest.install(self._db.add)

  def post_install(self):
    for manifest in self._load_manifests():
      log(f'\nRunning post install from {str(manifest.root)}')
      manifest.post_install()


async def asyncio_main():
  parser = argparse.ArgumentParser(description='Synchronized configuration setup')
  parser.add_argument('--no-update', '-d', action='store_false', dest='update')
  parser.add_argument('--uninstall', '-u', action='store_true', dest='uninstall_only')
  args = parser.parse_args()
  if args.update and not args.uninstall_only:
    if await update_all():
      log(f'Updated {str(SELF)}, restarting')
      os.execv(SELF, sys.argv + ['--no-update'])
  installer = Installer()
  installer.uninstall()
  if args.uninstall_only:
    sys.exit()
  installer.install()
  installer.post_install()


def main():
  asyncio.run(asyncio_main())
