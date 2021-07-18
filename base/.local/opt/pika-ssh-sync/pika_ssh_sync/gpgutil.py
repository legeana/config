import dataclasses
import enum
import pprint
import subprocess
from typing import List, Optional


@enum.unique
class Algorithm(enum.Enum):
  rsa_rsa =  1 # RSA and RSA (default)
  dsa_elg =  2 # DSA and Elgamal
  dsa     =  3 # DSA (sign only)
  rsa_s   =  4 # RSA (sign only)
  elg     =  5 # Elgamal (encrypt only)
  rsa_e   =  6 # RSA (encrypt only)
  dsa_a   =  7 # DSA (set your own capabilities)
  rsa_a   =  8 # RSA (set your own capabilities)
  ecc_ecc =  9 # ECC and ECC
  ecc_s   = 10 # ECC (sign only)
  ecc_a   = 11 # ECC (set your own capabilities)
  ecc_e   = 12 # ECC (encrypt only)
  keygrip = 13 # Existing key


class Tokens:

  def __init__(self, line):
    self._tokens = line.split(':')

  def _get(self, index) -> Optional[str]:
    if 0 < index <= len(self._tokens):
      return self._tokens[index - 1]
    return None

  def field(self) -> Optional[str]:
    return self._get(1)

  def validity(self) -> Optional[str]:
    return self._get(2)

  def length(self) -> Optional[str]:
    return self._get(3)

  def algo(self) -> Optional[Algorithm]:
    a = self._get(4)
    if a:
      return Algorithm(int(a))
    return None

  def key_id(self) -> Optional[str]:
    return self._get(5)

  def creation_date(self) -> Optional[str]:
    return self._get(6)

  def expiration_date(self) -> Optional[str]:
    return self._get(7)

  # 8 Certificate S/N

  def owner_trust(self) -> Optional[str]:
    return self._get(9)

  def user_id(self) -> Optional[str]:
    return self._get(10)

  def signature_class(self) -> Optional[str]:
    return self._get(11)

  def capabilities(self) -> Optional[str]:
    return self._get(12)


class DictRepr:

  def _drepr(self):
    return dict()

  def __repr__(self) -> str:
    return pprint.pformat(self._drepr())


class BaseKey(DictRepr):
  fingerprint = None
  validity = None
  length = None
  algo = None
  key_id = None
  creation_date = None
  expiration_date = None
  owner_trust = None
  capabilities = None

  def _drepr(self):
    d = super(BaseKey, self)._drepr()
    for attr in ['fingerprint',
                 'validity',
                 'length',
                 'algo',
                 'key_id',
                 'creation_date',
                 'expiration_date',
                 'owner_trust']:
      d[attr] = getattr(self, attr)
    return d


@dataclasses.dataclass
class Key(BaseKey):
  subkeys: List['SubKey'] = dataclasses.field(default_factory=list)
  uids: List['Uid'] = dataclasses.field(default_factory=list)

  def _drepr(self):
    d = super(Key, self)._drepr()
    d['subkeys'] = list(map(lambda x: x._drepr(), self.subkeys))
    d['uids'] = list(map(lambda x: x._drepr(), self.uids))
    return d


class SubKey(BaseKey):
  pass

  def bad(self):
    for v in 'idren':
      if v in self.validity:
        return True
    return False


@dataclasses.dataclass
class Uid(DictRepr):
  name: Optional[str]
  validity: Optional[str]

  def _drepr(self):
    d = super(Uid, self)._drepr()
    d['name'] = self.name
    d['validity'] = self.validity
    return d


class Parser:

  _keys: List[Key]
  _key: Optional[Key] = None
  _subkey: Optional[SubKey] = None

  def __init__(self):
    self._keys = []

  def parse(self, line) -> None:
    tokens = Tokens(line)
    if not tokens: return
    if tokens.field() in {'pub', 'sec'}:
      self._parse_key(tokens)
    elif tokens.field() in {'sub', 'ssb'}:
      self._parse_subkey(tokens)
    elif tokens.field() in {'uid'}:
      self._parse_uid(tokens)
    elif tokens.field() in {'fpr'}:
      self._parse_fpr(tokens)

  def keys(self) -> List[Key]:
    return self._keys

  def _fill_key(self, key, tokens: Tokens) -> None:
    key.validity = tokens.validity()
    key.length = tokens.length()
    key.key_id = tokens.key_id()
    key.creation_date = tokens.creation_date()
    key.expiration_date = tokens.expiration_date()
    key.owner_trust = tokens.owner_trust()
    key.capabilities = tokens.capabilities()

  def _parse_key(self, tokens: Tokens) -> None:
    self._key = Key()
    self._subkey = None
    self._keys.append(self._key)
    self._fill_key(self._key, tokens)

  def _parse_fpr(self, tokens) -> None:
    if self._subkey:
      self._subkey.fingerprint = tokens.user_id()
    elif self._key:
      self._key.fingerprint = tokens.user_id()

  def _parse_subkey(self, tokens: Tokens) -> None:
    if not self._key: return
    self._subkey = SubKey()
    self._key.subkeys.append(self._subkey)
    self._fill_key(self._subkey, tokens)

  def _parse_uid(self, tokens: Tokens) -> None:
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
