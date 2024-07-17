@echo off
setlocal

winget install gerardog.gsudo

set BOOTSTRAP=pikaconfig-bootstrap
set BINARY=pikaconfig
set ROOT=%~dp0
set SRC=%ROOT%
set BUILD=%SRC%\target\release
set CACHED_BOOTSTRAP=%BUILD%\%BOOTSTRAP%
set CACHED_BINARY=%BUILD%\%BINARY%

echo "Running in %ROOT%"

:: Environment used by pikaconfig binary.
set PIKACONFIG_CONFIG_ROOT=%ROOT%

:: Build environment.
:: Keep in sync with .cargo/config.toml.
:: Required because Cargo doesn't load config.toml unless run from the project
:: root.
set OS_STR_BYTES_CHECKED_CONVERSIONS=1

:: Setup using native tools.
cargo run --manifest-path="%SRC%\Cargo.toml" --package="%BOOTSTRAP%" --release -- %* || exit /b 1
cargo build --manifest-path="%SRC%\Cargo.toml" --package="%BINARY%" --release || exit /b 1
gsudo %CACHED_BINARY% %* || exit /b 1
