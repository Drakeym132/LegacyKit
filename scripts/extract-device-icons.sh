#!/usr/bin/env bash
# Extracts macOS-bundled, high-resolution device icons (1024x1024 PNG with
# transparency) from the system MobileDevices.bundle and writes them to
# public/devices/{productType}.png so the UI can render the matching art.
#
# Source bundle (read-only system path):
#   /System/Library/Templates/Data/System/Library/CoreServices/CoreTypes.bundle/
#   Contents/Library/MobileDevices.bundle/Contents/Resources
#
# Run from anywhere: `./scripts/extract-device-icons.sh`

set -euo pipefail

cd "$(dirname "$0")/.."

SRC="/System/Library/Templates/Data/System/Library/CoreServices/CoreTypes.bundle/Contents/Library/MobileDevices.bundle/Contents/Resources"
DEST="public/devices"
TMP="$(mktemp -d -t legacykit-icns)"
trap 'rm -rf "$TMP"' EXIT

if [[ ! -d "$SRC" ]]; then
  echo "ERROR: MobileDevices.bundle not found at:" >&2
  echo "  $SRC" >&2
  echo "This script requires macOS." >&2
  exit 1
fi

mkdir -p "$DEST"

# productType -> icns basename (without "com.apple." prefix or ".icns" suffix).
# Each icns yields a 1024x1024 PNG with full transparency.
MAP=(
  "iPhone1,1=iphone"
  "iPhone1,2=iphone-3g"
  "iPhone2,1=iphone-3g"
  "iPhone3,1=iphone-4-black"
  "iPhone3,2=iphone-4-black"
  "iPhone3,3=iphone-4-cdma-black"
  "iPhone4,1=iphone-4-black"
  "iPhone5,1=iphone-5-black"
  "iPhone5,2=iphone-5-black"
  "iPhone5,3=iphone-5c-f5f4f7"
  "iPhone5,4=iphone-5c-f5f4f7"
  "iPhone6,1=iphone-5s-99989b"
  "iPhone6,2=iphone-5s-99989b"

  "iPad1,1=ipad"
  "iPad2,1=ipad-2-black"
  "iPad2,2=ipad-2-black"
  "iPad2,3=ipad-2-black"
  "iPad2,4=ipad-2-black"
  "iPad2,5=ipad-mini-black-wifi"
  "iPad2,6=ipad-mini-black-wifi"
  "iPad2,7=ipad-mini-black-wifi"
  "iPad3,1=ipad-2-black"
  "iPad3,2=ipad-2-black"
  "iPad3,3=ipad-2-black"
  "iPad3,4=ipad-2-black"
  "iPad3,5=ipad-2-black"
  "iPad3,6=ipad-2-black"
  "iPad4,1=ipad-air-99989b"
  "iPad4,2=ipad-air-99989b"
  "iPad4,3=ipad-air-99989b"
  "iPad4,4=ipad-mini2-99989b"
  "iPad4,5=ipad-mini2-99989b"
  "iPad4,6=ipad-mini2-99989b"
  "iPad4,7=ipad-mini3-wifi-b4b5b9"
  "iPad4,8=ipad-mini3-wifi-b4b5b9"
  "iPad4,9=ipad-mini3-wifi-b4b5b9"

  "iPod1,1=ipod-touch"
  "iPod2,1=ipod-touch-2"
  "iPod3,1=ipod-touch-2"
  "iPod4,1=ipod-touch-4-black"
  "iPod5,1=ipod-touch-5-slate"
)

extract_largest() {
  local icns="$1"
  local out="$2"
  local set_dir="$TMP/$(basename "$icns" .icns).iconset"

  rm -rf "$set_dir"
  if ! iconutil -c iconset -o "$set_dir" "$icns" 2>/dev/null; then
    return 1
  fi
  # Prefer 1024 (512@2x), then 512, then 256@2x, etc.
  for cand in icon_512x512@2x.png icon_512x512.png icon_256x256@2x.png \
              icon_256x256.png icon_128x128@2x.png icon_128x128.png; do
    if [[ -f "$set_dir/$cand" ]]; then
      cp "$set_dir/$cand" "$out"
      return 0
    fi
  done
  return 1
}

ok=0
fail=0
for entry in "${MAP[@]}"; do
  id="${entry%%=*}"
  name="${entry#*=}"
  icns="$SRC/com.apple.$name.icns"
  out="$DEST/$id.png"

  if [[ ! -f "$icns" ]]; then
    printf "  MISS  %-12s  com.apple.%s.icns not found\n" "$id" "$name"
    fail=$((fail + 1))
    continue
  fi

  if extract_largest "$icns" "$out"; then
    size=$(sips -g pixelWidth "$out" 2>/dev/null | awk '/pixelWidth/ {print $2}')
    printf "  ok    %-12s  %-40s  %sx%s\n" "$id" "com.apple.$name.icns" "$size" "$size"
    ok=$((ok + 1))
  else
    printf "  FAIL  %-12s  could not extract from com.apple.%s.icns\n" "$id" "$name"
    fail=$((fail + 1))
  fi
done

echo
echo "$ok extracted, $fail failed -> $DEST"
