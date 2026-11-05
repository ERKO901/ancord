#!/bin/sh
# Instalátor nodu ancord. Použití:
#   curl -fsSL https://github.com/ERKO901/ancord/releases/latest/download/install.sh | sh
# Instaluje jen do jedné složky ($ANCORD_HOME, výchozí ~/.ancord) bez sudo a bez úprav shellu;
# mimo ni vznikne jen odkaz ~/.local/bin/ancord. Opětovné spuštění = aktualizace (nastavení i klíč zůstanou).
# Proměnné: ANCORD_HOME, ANCORD_BIN_DIR, ANCORD_VERSION (např. node-v0.1.0), ANCORD_BASE_URL (zrcadlo/test).
set -eu

REPO=ERKO901/ancord
HOME_DIR=${ANCORD_HOME:-$HOME/.ancord}
BIN_DIR=${ANCORD_BIN_DIR:-$HOME/.local/bin}
if [ -n "${ANCORD_BASE_URL:-}" ]; then BASE=$ANCORD_BASE_URL
elif [ -n "${ANCORD_VERSION:-}" ]; then BASE=https://github.com/$REPO/releases/download/$ANCORD_VERSION
else BASE=https://github.com/$REPO/releases/latest/download; fi

# Kořeny důvěry produkční sítě (veřejné údaje, viz ARCHITECTURE.md §12.5).
WORKER_URL=https://ancord.melonet.cz
WORKER_PUBKEY=MpUwUO74HgS5ZvLi1guR_aajIEZCoP4olm6gKlfnIo0

# cloudflared: pevná verze a SHA-256 (stejné jako node/Dockerfile).
CF_VERSION=2026.10.3
CF_SHA256_AMD64=053b80b0a23031a038fdbf3c073f59f82b8f1f152aa67971d7383ffafc0dbab0
CF_SHA256_ARM64=89297ebfb9e8c99c0dc013a1140601453cf3798d8306c5591651f4a3f5210be8

say() { printf '%s\n' "$*"; }
die() { printf 'chyba: %s\n' "$*" >&2; exit 1; }

[ "$(uname -s)" = Linux ] || die "podporován je jen Linux (na Windows použijte WSL2, na macOS Docker, viz dev/README.md)"
[ "$(id -u)" -ne 0 ] || die "nespouštějte jako root; node běží pod běžným uživatelem a instalace nepotřebuje sudo"
case $(uname -m) in
  x86_64|amd64) ARCH=x86_64; CF_ARCH=amd64; CF_SUM=$CF_SHA256_AMD64 ;;
  aarch64|arm64) ARCH=aarch64; CF_ARCH=arm64; CF_SUM=$CF_SHA256_ARM64 ;;
  *) die "nepodporovaná architektura $(uname -m)" ;;
esac
for c in tar sha256sum; do command -v "$c" >/dev/null 2>&1 || die "chybí příkaz $c"; done
if command -v curl >/dev/null 2>&1; then fetch() { curl -fsSL "$1" -o "$2"; }
elif command -v wget >/dev/null 2>&1; then fetch() { wget -q "$1" -O "$2"; }
else die "chybí curl nebo wget"; fi

TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT INT TERM
verify() { # soubor očekávaný_součet
  [ "$(sha256sum "$1" | cut -d' ' -f1)" = "$2" ] || die "kontrolní součet $(basename "$1") nesedí, instalace přerušena"
}

TARBALL=ancord-node-linux-$ARCH.tar.gz
say "Stahuji ancord ($ARCH)…"
fetch "$BASE/$TARBALL" "$TMP/$TARBALL" || die "stažení $BASE/$TARBALL selhalo"
fetch "$BASE/SHA256SUMS" "$TMP/SHA256SUMS" || die "stažení SHA256SUMS selhalo"
WANT=$(awk -v f="$TARBALL" '$2 == f {print $1}' "$TMP/SHA256SUMS")
[ -n "$WANT" ] || die "SHA256SUMS neobsahuje $TARBALL"
verify "$TMP/$TARBALL" "$WANT"
mkdir "$TMP/x"; tar -xzf "$TMP/$TARBALL" -C "$TMP/x"
for f in ancord-node ancord VERSION; do [ -f "$TMP/x/$f" ] || die "archiv neobsahuje $f"; done

mkdir -p "$HOME_DIR/bin" "$HOME_DIR/data"; chmod 700 "$HOME_DIR" "$HOME_DIR/data"
: >"$HOME_DIR/.ancord-home"   # značka, podle které odinstalace pozná vlastní složku
# Atomická výměna (mv = rename), aby šla aktualizace spustit z běžícího `ancord`.
for f in ancord-node ancord; do install -m 0755 "$TMP/x/$f" "$HOME_DIR/bin/$f.new"; mv -f "$HOME_DIR/bin/$f.new" "$HOME_DIR/bin/$f"; done
install -m 0644 "$TMP/x/VERSION" "$HOME_DIR/VERSION"

CF_CUR=$("$HOME_DIR/bin/cloudflared" --version 2>/dev/null || true)
case $CF_CUR in *"$CF_VERSION"*) ;; *)
  say "Stahuji cloudflared $CF_VERSION…"
  fetch "https://github.com/cloudflare/cloudflared/releases/download/$CF_VERSION/cloudflared-linux-$CF_ARCH" "$TMP/cloudflared" \
    || die "stažení cloudflared selhalo"
  verify "$TMP/cloudflared" "$CF_SUM"
  install -m 0755 "$TMP/cloudflared" "$HOME_DIR/bin/cloudflared.new"; mv -f "$HOME_DIR/bin/cloudflared.new" "$HOME_DIR/bin/cloudflared" ;;
esac

if [ ! -f "$HOME_DIR/config.env" ]; then
  printf 'WORKER_URL=%s\nWORKER_PUBKEY=%s\n' "$WORKER_URL" "$WORKER_PUBKEY" >"$HOME_DIR/config.env"
  chmod 600 "$HOME_DIR/config.env"
fi

mkdir -p "$BIN_DIR"
if [ -e "$BIN_DIR/ancord" ] && [ "$(readlink -f "$BIN_DIR/ancord")" != "$(readlink -f "$HOME_DIR/bin/ancord")" ]; then
  say "Pozor: $BIN_DIR/ancord už existuje a není z této instalace, nepřepisuji. Panel spustíte jako $HOME_DIR/bin/ancord."
else
  ln -sf "$HOME_DIR/bin/ancord" "$BIN_DIR/ancord"
fi

say "ancord $(cat "$HOME_DIR/VERSION") nainstalován do $HOME_DIR"
[ -n "${ANCORD_NO_HINTS:-}" ] && exit 0
case ":$PATH:" in *":$BIN_DIR:"*) ;; *) say "Přidejte $BIN_DIR do PATH (např. do ~/.profile), nebo spouštějte $HOME_DIR/bin/ancord." ;; esac
say "Dál: spusťte příkaz  ancord  (menu, nastavení výkonu, start nodu). Odinstalace: ancord uninstall"
