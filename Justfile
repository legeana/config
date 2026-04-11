namespace := "lontra"
binary := "lontra"
allowed-signers := "~/.gitconfig.keys/allowed-signers"
identity := shell("git config get user.email")
key := trim_start_match(shell("git signing-key"), "key::")
cross-root := absolute_path("cross")
cross-target := cross-root / "target"
cross-toolchains := cross-root / "toolchains"
macos-sdk-dir := cross-toolchains / "docker"

# MacOS SDK.
# https://github.com/cross-rs/cross-toolchains?tab=readme-ov-file#darwin-targets
macos-target := "aarch64-apple-darwin"
macos-image := macos-target + "-cross"
macos-image-tag := "local"
macos-sdk-url := "https://github.com/joseluisq/macosx-sdks/releases/download/13.3/MacOSX13.3.sdk.tar.xz"
macos-sdk-sha256 := "518e35eae6039b3f64e8025f4525c1c43786cc5cf39459d609852faf091e34be"
macos-sdk-filename := file_name(macos-sdk-url)
macos-sdk-file := macos-sdk-dir / macos-sdk-filename
macos-cross-config := cross-root / (macos-target + ".toml")

default: build-all

[no-cd]
ssh-sign file: && (ssh-verify file)
  #!/usr/bin/env -S bash -xeuo pipefail
  KEY={{quote(key)}}
  FILE={{quote(file)}}
  SIG={{quote(file + ".sig")}}
  if [[ -f $SIG ]]; then
    rm -f "$SIG"
  fi
  ssh-keygen -Y sign -n {{namespace}} -f <(echo "$KEY") "$FILE"

[no-cd]
curl url filename:
  curl --silent --show-error --fail \
    --follow {{quote(url)}} \
    --create-dirs --output {{quote(filename)}}

[no-cd]
verify-sha256 filename checksum:
  echo {{checksum}} {{filename}} | sha256sum -c -

[no-cd]
mkdir dirname:
  mkdir -p {{quote(dirname)}}

[no-cd]
download url filename sha256: && (verify-sha256 filename sha256)
  if [[ ! -f {{quote(filename)}} ]]; then \
    just curl {{quote(url)}} {{quote(filename)}}; \
  fi

[no-cd]
ssh-verify file:
  ssh-keygen -Y verify -n {{namespace}} -I {{identity}} -f {{allowed-signers}} -s {{quote(file + ".sig")}} <{{quote(file)}}

download-macos-sdk: (download macos-sdk-url macos-sdk-file macos-sdk-sha256)

cross-toolchains: (mkdir cross-root)
  #!/usr/bin/env -S bash -xeuo pipefail
  if [[ ! -d {{quote(cross-toolchains)}} ]]; then
    git clone https://github.com/cross-rs/cross.git {{quote(cross-toolchains)}}
  fi
  cd {{quote(cross-toolchains)}}
  git pull --ff-only
  git submodule update --init --remote
  # https://github.com/rust-lang/cargo/issues/7621
  sed -r 's|\["|"|; s|", "| |g; s|"\]|"|' -i .cargo/config.toml

macos-toolchain: cross-toolchains download-macos-sdk
  #!/usr/bin/env -S bash -xeuo pipefail
  pushd {{quote(cross-toolchains)}}
  cargo build-docker-image {{macos-image}} \
    --build-arg MACOS_SDK_FILE={{quote(macos-sdk-filename)}} \
    --tag {{macos-image-tag}}
  popd

macos-cross-config:
  #!/usr/bin/env -S bash -xeuo pipefail
  FILE={{quote(macos-cross-config)}}
  echo -n >"$FILE"
  echo "[target.{{macos-target}}]" >>"$FILE"
  echo "image = \"ghcr.io/cross-rs/{{macos-image}}:{{macos-image-tag}}\"" >> "$FILE"

install-cross:
  cargo install cross --git https://github.com/cross-rs/cross

generic-build tool target: \
    test \
    && \
    (ssh-sign cross-target / target / "release" / binary)
  {{tool}} build --release --target={{target}} --target-dir={{cross-target}}

native-build target: && (generic-build "cargo" target)

cross-build target: install-cross && (generic-build "cross" target)

test:
  cargo test --all-targets

[linux]
build-x86_64-unknown-linux-gnu: \
    (native-build "x86_64-unknown-linux-gnu")

[macos]
build-x86_64-unknown-linux-gnu: \
    (cross-build "x86_64-unknown-linux-gnu")

[linux]
build-macos: \
    macos-toolchain \
    macos-cross-config \
    (cross-build macos-target)

[macos]
build-macos: \
    (native-build macos-target)

build-all: \
    build-x86_64-unknown-linux-gnu \
    build-macos
