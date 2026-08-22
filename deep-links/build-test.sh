#!/usr/bin/env zsh

#
# Depp link handling requires sage to be installed so the handler can be registered. 
# This script builds the app and optionally installs it to /Applications and launches 
# it once to register the sage: URL scheme.
#
set -euo pipefail

usage() {
  echo "Usage: $0 [--full]"
  echo "  (default)  build only"
  echo "  --full     also install to /Applications and launch once to register the sage: URL scheme"
  exit 1
}

FULL=false
for arg in "$@"; do
  case "$arg" in
    --full) FULL=true ;;
    -h|--help) usage ;;
    *) echo "Unknown option: $arg" >&2; usage ;;
  esac
done

SCRIPT_DIR="${0:A:h}"
REPO_ROOT="${SCRIPT_DIR:h}"
APP_NAME="Sage.app"
BUNDLE_SRC="$REPO_ROOT/target/debug/bundle/macos/$APP_NAME"
APPLICATIONS_DEST="/Applications/$APP_NAME"

cd "$REPO_ROOT"

export SDKROOT="$(xcrun --show-sdk-path)"

echo "==> Building frontend + builtin apps"
pnpm run build

echo "==> Building Tauri debug bundle"
(cd src-tauri && npx tauri build --debug)

if [[ ! -d "$BUNDLE_SRC" ]]; then
  echo "Bundle not found at $BUNDLE_SRC" >&2
  exit 1
fi

if [[ "$FULL" == false ]]; then
  echo "==> Done (build only). Re-run with --full to install to /Applications and register the sage: URL scheme."
  exit 0
fi

echo "==> Installing to /Applications"
rm -rf "$APPLICATIONS_DEST"
cp -R "$BUNDLE_SRC" "$APPLICATIONS_DEST"

echo "==> Launching once to register the sage: URL scheme"
open "$APPLICATIONS_DEST"

echo "==> Done. Open sage-scheme-handler-test.html in a browser and click a link to test."
