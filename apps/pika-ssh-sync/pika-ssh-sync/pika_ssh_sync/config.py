import configparser
import dataclasses
import pathlib

_LOCATION = pathlib.Path.home() / '.config' / 'pika-ssh-sync.ini'


@dataclasses.dataclass
class Entry:
  name: str


@dataclasses.dataclass
class KeysAPIEntry(Entry):
  url: str


@dataclasses.dataclass
class GPGKeyEntry(Entry):
  fingerprint: str


@dataclasses.dataclass
class GPGUltimateEntry(Entry):
  pass


@dataclasses.dataclass
class KeybaseEntry(Entry):
  pass


@dataclasses.dataclass
class Config:

  entries: list[Entry]

  @classmethod
  def load(cls) -> 'Config':
    cfg = configparser.ConfigParser()
    cfg.read([_LOCATION])
    entries: list[Entry] = []
    for section in cfg.sections():
      entry_type = cfg.get(section, 'type')
      if entry_type == 'web_keys_api':
        entries.append(KeysAPIEntry(
            name=section,
            url=cfg.get(section, 'url'),
        ))
      elif entry_type == 'gpg_key':
        entries.append(GPGKeyEntry(
            name=section,
            fingerprint=cfg.get(section, 'fingerprint'),
        ))
      elif entry_type == 'gpg_ultimate_trust':
        entries.append(GPGUltimateEntry(name=section))
      elif entry_type == 'keybase_pgp':
        entries.append(KeybaseEntry(name=section))
      else:
        raise ValueError(f'Unknown key source {entry_type} in [{section}]')
    return cls(entries=entries)
