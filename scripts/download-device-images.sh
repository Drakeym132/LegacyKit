#!/usr/bin/env bash
# Downloads pre-A8 (legacy) device images from littlebyteorg/apple-device-images
# into public/devices/. Run once; commit the resulting PNGs.
#
# The upstream repo stores some devices as `device/{id}/{Color}.png`,
# some as flat `device/{id}.png`, and the oldest as `device-lowres/{id}.png`.
# We try each path in order and save the first hit as `{id}.png`.

set -euo pipefail

cd "$(dirname "$0")/.."
DEST="public/devices"
mkdir -p "$DEST"

CDN="https://img.appledb.dev"
RAW="https://raw.githubusercontent.com/littlebyteorg/apple-device-images/main"

# productType:defaultColor — color is the canonical AppleDB filename (no extension).
# Empty color means try only the flat-file paths.
DEVICES=(
  "iPhone1,1:"
  "iPhone1,2:"
  "iPhone2,1:"
  "iPhone3,1:"
  "iPhone3,2:"
  "iPhone3,3:"
  "iPhone4,1:"
  "iPhone5,1:"
  "iPhone5,2:"
  "iPhone5,3:White"
  "iPhone5,4:White"
  "iPhone6,1:Space Gray"
  "iPhone6,2:Space Gray"

  "iPad1,1:Black"
  "iPad2,1:Black"
  "iPad2,2:Black"
  "iPad2,3:Black"
  "iPad2,4:Black"
  "iPad2,5:Space Gray"
  "iPad2,6:Space Gray"
  "iPad2,7:Space Gray"
  "iPad3,1:Black"
  "iPad3,2:Black"
  "iPad3,3:Black"
  "iPad3,4:Black"
  "iPad3,5:Black"
  "iPad3,6:Black"
  "iPad4,1:Space Gray"
  "iPad4,2:Space Gray"
  "iPad4,3:Space Gray"
  "iPad4,4:Space Gray"
  "iPad4,5:Space Gray"
  "iPad4,6:Space Gray"
  "iPad4,7:Space Gray"
  "iPad4,8:Space Gray"
  "iPad4,9:Space Gray"

  "iPod1,1:Black"
  "iPod2,1:Black"
  "iPod3,1:Black"
  "iPod4,1:Black"
  "iPod5,1:Space Gray"
)

urlencode() {
  python3 -c 'import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1], safe="/:,"))' "$1"
}

try_download() {
  local url
  url=$(urlencode "$1")
  local out="$2"
  local code
  code=$(curl -sSL -o "$out.tmp" -w "%{http_code}" "$url" || echo "000")
  if [[ "$code" == "200" ]] && [[ -s "$out.tmp" ]] && file "$out.tmp" | grep -q "PNG image"; then
    mv "$out.tmp" "$out"
    return 0
  fi
  rm -f "$out.tmp"
  return 1
}

ok=0
fail=0
for entry in "${DEVICES[@]}"; do
  id="${entry%%:*}"
  color="${entry#*:}"
  out="$DEST/$id.png"

  # Prefer full-res from raw GitHub. Fall back to the CDN @256 if full-res
  # is unavailable, then to the flat device-lowres file (the only source for
  # iPhones 1.2 through 5.2).
  candidates=()
  if [[ -n "$color" ]]; then
    candidates+=("$RAW/device/$id/$color.png")
  fi
  candidates+=("$RAW/device/$id.png")
  if [[ -n "$color" ]]; then
    candidates+=("$CDN/device@256/$id/$color.png")
  fi
  candidates+=("$RAW/device-lowres/$id.png")

  hit=0
  for url in "${candidates[@]}"; do
    if try_download "$url" "$out"; then
      printf "  ok   %-12s  %s\n" "$id" "$(basename "$url")"
      ok=$((ok + 1))
      hit=1
      break
    fi
  done
  if [[ "$hit" -eq 0 ]]; then
    printf "  MISS %-12s  (no source matched)\n" "$id"
    fail=$((fail + 1))
  fi
done

# Variants that are visually identical to a sibling — copy the primary's image.
ALIASES=(
  "iPhone5,4=iPhone5,3"
  "iPhone6,2=iPhone6,1"
  "iPad2,2=iPad2,1"
  "iPad2,3=iPad2,1"
  "iPad2,4=iPad2,1"
  "iPad2,6=iPad2,5"
  "iPad2,7=iPad2,5"
  "iPad3,2=iPad3,1"
  "iPad3,3=iPad3,1"
  "iPad3,4=iPad3,1"
  "iPad3,5=iPad3,1"
  "iPad3,6=iPad3,1"
  "iPad4,2=iPad4,1"
  "iPad4,3=iPad4,1"
  "iPad4,5=iPad4,4"
  "iPad4,6=iPad4,4"
  "iPad4,8=iPad4,7"
  "iPad4,9=iPad4,7"
)

aliased=0
for entry in "${ALIASES[@]}"; do
  variant="${entry%%=*}"
  primary="${entry#*=}"
  if [[ -f "$DEST/$primary.png" ]] && [[ ! -f "$DEST/$variant.png" ]]; then
    cp "$DEST/$primary.png" "$DEST/$variant.png"
    printf "  alias %-12s -> %s\n" "$variant" "$primary"
    aliased=$((aliased + 1))
  fi
done

echo
echo "$ok downloaded, $aliased aliased, $fail missing -> $DEST"
