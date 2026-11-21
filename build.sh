#!/bin/sh
# Compile Acaboulette en release pour Linux et pour Windows (compilation
# croisee, necessite mingw-w64-gcc), puis copie les executables dans dist/.
#
#   ./build.sh           compile
#   ./build.sh --clear   nettoie tout (target/ et dist/) avant de compiler
set -eu

cd "$(dirname "$0")"

NAME=packarch-acaboulette
WIN_TARGET=x86_64-pc-windows-gnu

CLEAR=0
for arg in "$@"; do
    case "$arg" in
        --clear) CLEAR=1 ;;
        -h | --help)
            echo "Usage : $0 [--clear]"
            exit 0
            ;;
        *)
            echo "Option inconnue : $arg (usage : $0 [--clear])" >&2
            exit 1
            ;;
    esac
done

if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    echo "x86_64-w64-mingw32-gcc introuvable : sudo pacman -S mingw-w64-gcc" >&2
    exit 1
fi
if command -v rustup >/dev/null 2>&1 &&
    ! rustup target list --installed | grep -qx "$WIN_TARGET"; then
    rustup target add "$WIN_TARGET"
fi

if [ "$CLEAR" -eq 1 ]; then
    echo "==> Nettoyage"
    cargo clean
    rm -rf dist
fi

echo "==> Linux"
cargo build --release

echo "==> Windows"
cargo build --release --target "$WIN_TARGET"

mkdir -p dist
cp "target/release/$NAME" dist/
cp "target/$WIN_TARGET/release/$NAME.exe" dist/

echo "==> Termine :"
ls -lh "dist/$NAME" "dist/$NAME.exe"
