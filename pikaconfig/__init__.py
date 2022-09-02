import argparse
import asyncio
import logging
import os
import pathlib
import shlex
import stat
import subprocess
import sys
from typing import Iterable, Optional

from . import configuration
from . import database
from . import logconfig
from . import tagutil
from . import util

SELF = pathlib.Path(sys.argv[0]).absolute()
ROOT = SELF.parent
OVERLAYS = ROOT / 'overlay.d'
INSTALL = ROOT / '.install'
TAGS = ROOT / '.tags'
APPS = ROOT / 'apps'


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
    logging.info(f'Updated {util.format_path(plugin)}: {output.decode().strip()}')
  except subprocess.CalledProcessError as e:
    # TODO use shlex.join(), python-3.8+
    command = ' '.join(shlex.quote(arg) for arg in args)
    sys.exit(f'Failed to update {util.format_path(plugin)}, manual intervention is required!\n'
             f'$ {command}\n'
             f'{e.stdout.decode().strip()}')
  with open(ref) as f:
    new_ref = f.read()
  return old_ref != new_ref


async def update_all() -> bool:
  logging.info('Updating...')
  updates = [update(ROOT)] + [
      update(overlay) for overlay in OVERLAYS.iterdir()
      if overlay.is_dir()
  ]
  update_results = await asyncio.gather(*updates, return_exceptions=False)
  logging.info('')
  assert len(updates) == len(update_results)
  for res in update_results:
    if isinstance(res, BaseException):
      raise res
  return update_results[0]


class Installer:

  def __init__(self):
    self._old_db = database.InstalledDatabase.load_from(INSTALL)
    self._db = database.SyncInstalledDatabase(INSTALL)
    # lazy loading
    self._manifests: Optional[list[configuration.Manifest]] = None
    self._tags = tagutil.system_tags().union(tagutil.local_tags(TAGS))

  @classmethod
  def _rm_link(cls, path: pathlib.Path) -> None:
    try:
      path.unlink()
      logging.info(f'Removed symlink {util.format_path(path)}')
    except FileNotFoundError:
      # TODO missing_ok=True, python-3.8+
      pass
    cls._rm_dirs(path.parents)

  @classmethod
  def _rm_dir(cls, path: pathlib.Path) -> None:
    try:
      path.rmdir()
      logging.info(f'Removed empty directory {util.format_path(path)}')
    except OSError as e:
      logging.error(f'Unable to remove directory {util.format_path(path)}: {e}')
    cls._rm_dirs(path.parents)

  @classmethod
  def _rm_dirs(cls, paths: Iterable[pathlib.Path]) -> None:
    for path in paths:
      try:
        path.rmdir()
        logging.info(f'Removed empty directory {util.format_path(path)}')
      except OSError:
        break

  @classmethod
  def _rm_install(cls) -> None:
    try:
      INSTALL.unlink()
    except FileNotFoundError:
      pass

  def uninstall(self) -> None:
    for path in reversed(self._old_db):
      try:
        pstat = path.lstat()
      except FileNotFoundError:
        logging.info(f'{util.format_path(path)} does not exist, nothing to remove')
      if stat.S_ISLNK(pstat.st_mode):
        self._rm_link(path)
      elif stat.S_ISDIR(pstat.st_mode):
        self._rm_dir(path)
      else:
        logging.error(f'Unable to remove {util.format_path(path)}')
    self._rm_install()

  def _paths(self) -> Iterable[pathlib.Path]:
    yield from sorted(APPS.iterdir())
    for overlay in sorted(OVERLAYS.iterdir()):
      if not overlay.is_dir():
        continue
      if configuration.is_overlay(overlay):
        yield overlay
      else:
        for sub in sorted(overlay.iterdir()):
          if configuration.is_overlay(sub):
            yield sub

  def force_load(self) -> None:
    """Force MANIFEST loading to catch errors."""
    self._load_manifests()

  def _load_manifests(self) -> Iterable[configuration.Manifest]:
    if self._manifests is not None:
      return self._manifests
    self._manifests = []
    for path in self._paths():
      if not path.is_dir():
        continue
      try:
        self._manifests.append(configuration.Manifest(path, pathlib.Path.home()))
      except FileNotFoundError as e:
        logging.error(f'Unable to load MANIFEST in {util.format_path(path)}: {e}')
        continue
    return self._manifests

  def system_setup(self) -> None:
    for manifest in self._load_manifests():
      logging.info(f'\nInstalling packages from {manifest}')
      manifest.recursive_system_setup(self._tags)

  def install(self) -> None:
    for manifest in self._load_manifests():
      logging.info(f'\nInstalling from {manifest}')
      manifest.recursive_install(self._tags, self._db.add)

  def post_install(self) -> None:
    for manifest in self._load_manifests():
      logging.info(f'\nRunning post install from {manifest}')
      manifest.recursive_post_install(self._tags)

  def supported_tags(self) -> tagutil.TagSet:
    tags = tagutil.TagSet()
    for manifest in self._load_manifests():
      tags.update(manifest.supported_tags())
    return tags


async def handle_update(args: argparse.Namespace) -> None:
  if args.update:
    if await update_all():
      logging.info(f'Updated {util.format_path(SELF)}, restarting')
      os.execv(SELF, [sys.argv[0], '--no-update'] + sys.argv[1:])


async def handle_install(args: argparse.Namespace) -> None:
  await handle_update(args)
  installer = Installer()
  installer.force_load()  # catch errors early
  installer.uninstall()
  installer.install()
  installer.post_install()


async def handle_tag(args: argparse.Namespace) -> None:
  await handle_update(args)
  print('System tags:', tagutil.system_tags())
  print('Local tags:', tagutil.local_tags(TAGS))
  installer = Installer()
  print('Supported tags:', installer.supported_tags())


async def handle_tag_add(args: argparse.Namespace) -> None:
  tags = tagutil.local_tags(TAGS)
  tags.update(args.tags)
  tagutil.save_local_tags(TAGS, tags)


async def handle_tag_rm(args: argparse.Namespace) -> None:
  tags = tagutil.local_tags(TAGS)
  tags.difference_update(args.tags)
  tagutil.save_local_tags(TAGS, tags)


async def handle_system_setup(args: argparse.Namespace) -> None:
  await handle_update(args)
  installer = Installer()
  installer.system_setup()


async def handle_uninstall(args: argparse.Namespace) -> None:
  # no update is necessary, just purge everything
  installer = Installer(args.tags)
  installer.uninstall()


async def asyncio_main():
  parser = argparse.ArgumentParser(description='Synchronized configuration setup')
  parser.set_defaults(func=None)
  parser.add_argument(
      '--verbose', '-v', action='store_true', dest='verbose',
      help='Print all actions taken')
  parser.add_argument('--no-update', '-d', action='store_false', dest='update')
  parser.add_argument(
      '--tags', '-t', nargs='+', default=tagutil.system_tags(),
      help='List of tags to set for the current machine')
  subparsers = parser.add_subparsers()

  uninstall_parser = subparsers.add_parser(
      'uninstall', help='Uninstall configuration')
  uninstall_parser.set_defaults(func=handle_uninstall)

  install_parser = subparsers.add_parser(
      'install', help='Install configuration')
  install_parser.set_defaults(func=handle_install)

  tag_parser = subparsers.add_parser('tag', help='Configure local tags')
  tag_parser.set_defaults(func=handle_tag)
  tag_subparsers = tag_parser.add_subparsers()

  tag_parser_add = tag_subparsers.add_parser('add', help='Add tags')
  tag_parser_add.set_defaults(func=handle_tag_add)
  tag_parser_add.add_argument('tags', nargs='+', default=[])

  tag_parser_rm = tag_subparsers.add_parser('rm', help='Remove tags')
  tag_parser_rm.set_defaults(func=handle_tag_rm)
  tag_parser_rm.add_argument('tags', nargs='+', default=[])

  system_parser = subparsers.add_parser(
      'system',
      help='Execute system level commands such as package installation')
  system_parser.set_defaults(func=handle_system_setup)

  args = parser.parse_args()

  logconfig.init(verbose=args.verbose)
  if args.func is None:
    parser.print_help()
  else:
    await args.func(args)


def main():
  asyncio.run(asyncio_main())
