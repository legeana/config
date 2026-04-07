namespace := "lontra"
allowed-signers := "~/.gitconfig.keys/allowed-signers"
identity := shell("git config get user.email")
key := trim_start_match(shell("git signing-key"), "key::")
cross-target := "cross-target"

default: build-all

ssh-sign file: && (ssh-verify file)
    #!/usr/bin/env -S bash -xeuo pipefail
    KEY={{quote(key)}}
    FILE={{quote(file)}}
    SIG={{quote(file + ".sig")}}
    if [[ -f $SIG ]]; then
        rm -f "$SIG"
    fi
    ssh-keygen -Y sign -n {{namespace}} -f <(echo "$KEY") "$FILE"

ssh-verify file:
    ssh-keygen -Y verify -n {{namespace}} -I {{identity}} -f {{allowed-signers}} -s {{quote(file + ".sig")}} <{{quote(file)}}

install-cross:
    cargo install cross --git https://github.com/cross-rs/cross

# Better error message for cross-rs.
require-docker:
    docker ps >/dev/null

build target: install-cross require-docker && (ssh-sign cross-target / target / "release" / "lontra")
    cross test --release --target={{target}} --target-dir={{cross-target}}
    cross build --release --target={{target}} --target-dir={{cross-target}}

build-all:
    # See <https://github.com/cross-rs/cross-toolchains?tab=readme-ov-file#apple-targets>.
    just build aarch64-apple-darwin
    just build x86_64-unknown-linux-gnu
