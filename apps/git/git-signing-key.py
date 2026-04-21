#!/usr/bin/python3

import dataclasses
import os
import subprocess
import sys


@dataclasses.dataclass(frozen=True)
class SshKey:
    kind: str
    pubkey: str

    def __str__(self) -> str:
        return f"{self.kind} {self.pubkey}"


def main():
    available = available_keys()
    allowed = allowed_keys()
    for a in allowed:
        if a in available:
            print(f"key::{a}")
            return
    print(
        f"failed to find allowed signing key, allowed = {allowed}, "
        f"available = {available}",
        file=sys.stderr,
    )
    sys.exit(1)


def line_fields(line: str, a: int, b: int) -> tuple[str, str]:
    split = line.strip().split()
    a_s = None
    b_s = None
    for i, part in enumerate(split):
        if i == a:
            a_s = part
        if i == b:
            b_s = part
        if a_s is not None and b_s is not None:
            return (a_s, b_s)
    raise IndexError(f"{line!r} does not contain fields {a} and {b}")


def split_ssh_add_line(line: str) -> SshKey:
    try:
        kind, pubkey = line_fields(line, 0, 1)
    except IndexError as e:
        raise ValueError(f"failed to split ssh-add output: {line}") from e
    return SshKey(kind, pubkey)


def split_allowed_signers_line(line: str) -> SshKey:
    try:
        kind, pubkey = line_fields(line, 1, 2)
    except IndexError as e:
        raise ValueError(f"failed to split allowed-signers line: {line}") from e
    return SshKey(kind, pubkey)


def available_keys() -> set[SshKey]:
    lines = subprocess.check_output(["ssh-add", "-L"], text=True).split("\n")
    return {split_ssh_add_line(line) for line in lines if line}


def allowed_keys() -> set[SshKey]:
    output = subprocess.check_output(
        ["git", "config", "get", "gpg.ssh.allowedSignersFile"],
        text=True,
    )
    with open(os.path.expanduser(output.strip())) as f:
        return {split_allowed_signers_line(line) for line in f if line}


if __name__ == "__main__":
    main()
