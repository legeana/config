import dataclasses
import enum
import pprint
import subprocess
from typing import List, Optional


@enum.unique
class Algorithm(enum.Enum):
  UNKNOWN = -1 # not set properly
  RSA_RSA =  1 # RSA and RSA (default)
  DSA_ELG =  2 # DSA and Elgamal
  DSA     =  3 # DSA (sign only)
  RSA_S   =  4 # RSA (sign only)
  ELG     =  5 # Elgamal (encrypt only)
  RSA_E   =  6 # RSA (encrypt only)
  DSA_A   =  7 # DSA (set your own capabilities)
  RSA_A   =  8 # RSA (set your own capabilities)
  ECC_ECC =  9 # ECC and ECC
  ECC_S   = 10 # ECC (sign only)
  ECC_A   = 11 # ECC (set your own capabilities)
  ECC_E   = 12 # ECC (encrypt only)
  KEYGRIP = 13 # Existing key


class Tokens:

  _tokens: List[str]

  def __init__(self, line: str):
    self._tokens = line.split(':')

  def get(self, index: int) -> str:
    if 0 < index <= len(self._tokens):
      return self._tokens[index - 1]
    raise IndexError(f'No field with index {index}')

  def type(self) -> str:
    return self.get(1)


class KeyTokens:

  _tokens: Tokens

  def __init__(self, tokens: Tokens):
    self._tokens = tokens

  def type(self) -> str:
    return self._tokens.type()

  def validity(self) -> str:
    return self._tokens.get(2)

  def length(self) -> str:
    return self._tokens.get(3)

  def algo(self) -> Algorithm:
    a = self._tokens.get(4)
    if a:
      try:
        return Algorithm(int(a))
      except ValueError:
        pass
    return Algorithm.UNKNOWN

  def key_id(self) -> str:
    return self._tokens.get(5)

  def creation_date(self) -> str:
    return self._tokens.get(6)

  def expiration_date(self) -> str:
    return self._tokens.get(7)

  # 8 Certificate S/N

  def owner_trust(self) -> str:
    return self._tokens.get(9)

  def user_id(self) -> str:
    return self._tokens.get(10)

  def signature_class(self) -> str:
    return self._tokens.get(11)

  def capabilities(self) -> str:
    return self._tokens.get(12)


@dataclasses.dataclass
class BaseKey:

  fingerprint: str = ''
  validity: str = ''
  length: str = ''
  algo: Algorithm = Algorithm.UNKNOWN
  key_id: str = ''
  creation_date: str = ''
  expiration_date: str = ''
  owner_trust: str = ''
  capabilities: str = ''


@dataclasses.dataclass
class Key(BaseKey):

  subkeys: List['SubKey'] = dataclasses.field(default_factory=list)
  uids: List['Uid'] = dataclasses.field(default_factory=list)


class SubKey(BaseKey):

  def bad(self):
    for v in 'idren':
      if v in self.validity:
        return True
    return False


@dataclasses.dataclass
class Uid:

  name: str
  validity: str


class Parser:

  _keys: List[Key]
  _key: Optional[Key] = None
  _subkey: Optional[SubKey] = None

  def __init__(self):
    self._keys = []

  def parse(self, line) -> None:
    tokens = Tokens(line)
    if not tokens: return
    if tokens.type() in {'pub', 'sec'}:
      self._parse_key(KeyTokens(tokens))
    elif tokens.type() in {'sub', 'ssb'}:
      self._parse_subkey(KeyTokens(tokens))
    elif tokens.type() in {'uid'}:
      self._parse_uid(KeyTokens(tokens))
    elif tokens.type() in {'fpr'}:
      self._parse_fpr(KeyTokens(tokens))

  def keys(self) -> List[Key]:
    return self._keys

  def _fill_key(self, key, tokens: KeyTokens) -> None:
    key.validity = tokens.validity()
    key.length = tokens.length()
    key.key_id = tokens.key_id()
    key.creation_date = tokens.creation_date()
    key.expiration_date = tokens.expiration_date()
    key.owner_trust = tokens.owner_trust()
    key.capabilities = tokens.capabilities()

  def _parse_key(self, tokens: KeyTokens) -> None:
    self._key = Key()
    self._subkey = None
    self._keys.append(self._key)
    self._fill_key(self._key, tokens)

  def _parse_fpr(self, tokens: KeyTokens) -> None:
    if self._subkey:
      self._subkey.fingerprint = tokens.user_id()
    elif self._key:
      self._key.fingerprint = tokens.user_id()

  def _parse_subkey(self, tokens: KeyTokens) -> None:
    if not self._key: return
    self._subkey = SubKey()
    self._key.subkeys.append(self._subkey)
    self._fill_key(self._subkey, tokens)

  def _parse_uid(self, tokens: KeyTokens) -> None:
    self._subkey = None
    uid = Uid(name=tokens.user_id(), validity=tokens.validity())
    assert self._key is not None
    self._key.uids.append(uid)


class GPG:
  binary = 'gpg'
  gnupghome = None

  def __init__(self, binary=None, gnupghome=None):
    if binary:
      self.binary = binary
    if gnupghome:
      self.gnupghome = gnupghome

  def _argv(self, *args, **kwargs) -> List[str]:
    argv = [self.binary]
    argv.extend(['--batch', '--fixed-list-mode'])
    if self.gnupghome:
      argv.append('--homedir=' + self.gnupghome)
    argv.extend(args)
    for key, value in kwargs.items():
      argv += '--%s=%s' % (key.replace('_', '-'), value)
    return argv

  def _check_output(self, *args, **kwargs) -> bytes:
    return subprocess.check_output(self._argv(*args, **kwargs))

  def _check_utf8(self, *args, **kwargs) -> str:
    return self._check_output(*args, **kwargs).decode('utf8')

  def import_keys(self, data: bytes) -> None:
    with subprocess.Popen(self._argv('--import'),
                          stdin=subprocess.PIPE,
                          stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT) as proc:
      proc.communicate(input=data)

  def list_key(self, fp) -> Optional[Key]:
    keys = self._list_keys(fp)
    for key in keys:
      if key.fingerprint == fp:
        return key
    return None

  def list_keys(self) -> List[Key]:
    return self._list_keys()

  def _list_keys(self, *args) -> List[Key]:
    out = self._check_utf8('--list-keys', '--with-colons', *args)
    p = Parser()
    for line in out.split('\n'):
      p.parse(line)
    return p.keys()

  def export_ssh_key(self, fp) -> str:
    return self._check_utf8('--export-ssh-key', fp + '!').strip()
