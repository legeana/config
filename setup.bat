@echo off
setlocal

winget install gerardog.gsudo

set BOOTSTRAP=lontra-bootstrap
set BINARY=lontra
set ROOT=%~dp0
set SRC=%ROOT%
set BUILD=%SRC%\target\release
set CACHED_BOOTSTRAP=%BUILD%\%BOOTSTRAP%
set CACHED_BINARY=%BUILD%\%BINARY%

echo "Running in %ROOT%"

:: Environment used by lontra binary.
set LONTRA_CONFIG_ROOT=%ROOT%

:: Setup using native tools.
cargo run --manifest-path="%SRC%\Cargo.toml" --locked --package="%BOOTSTRAP%" --release -- %* || exit /b 1
cargo build --manifest-path="%SRC%\Cargo.toml" --locked --package="%BINARY%" --release || exit /b 1
gsudo %CACHED_BINARY% %* || exit /b 1
