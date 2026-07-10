#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 <dmg-path> <icns-path>" >&2
  exit 1
fi

DMG_PATH="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
ICNS_PATH="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"

if [[ ! -f "$DMG_PATH" ]]; then
  echo "DMG not found: $DMG_PATH" >&2
  exit 1
fi

if [[ ! -f "$ICNS_PATH" ]]; then
  echo "ICNS not found: $ICNS_PATH" >&2
  exit 1
fi

if ! command -v hdiutil >/dev/null 2>&1 || ! command -v SetFile >/dev/null 2>&1; then
  echo "hdiutil and SetFile are required to patch a DMG volume icon." >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
RW_DMG="$TMP_DIR/volume-icon-rw.dmg"
PATCHED_DMG="$TMP_DIR/volume-icon-patched.dmg"
MOUNT_POINT=""
DEVICE=""

cleanup() {
  if [[ -n "$DEVICE" ]]; then
    hdiutil detach "$DEVICE" -quiet >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

hdiutil convert "$DMG_PATH" -format UDRW -o "$RW_DMG" -quiet

ATTACH_OUTPUT="$(hdiutil attach "$RW_DMG" -readwrite -nobrowse)"
DEVICE="$(printf "%s\n" "$ATTACH_OUTPUT" | awk '/^\/dev\// {print $1; exit}')"
MOUNT_POINT="$(printf "%s\n" "$ATTACH_OUTPUT" | awk '/\/Volumes\// {for (i = 3; i <= NF; i++) {printf "%s%s", (i == 3 ? "" : " "), $i}; print ""; exit}')"

if [[ -z "$DEVICE" || -z "$MOUNT_POINT" || ! -d "$MOUNT_POINT" ]]; then
  echo "Failed to mount writable DMG." >&2
  exit 1
fi

cp "$ICNS_PATH" "$MOUNT_POINT/.VolumeIcon.icns"
SetFile -a C "$MOUNT_POINT"
sync
hdiutil detach "$DEVICE" -quiet
DEVICE=""

hdiutil convert "$RW_DMG" -format UDZO -o "$PATCHED_DMG" -quiet
mv "$PATCHED_DMG" "$DMG_PATH"
