#!/usr/bin/env bash
# Share cards: a 1200 by 630 PNG per project from its cover, with the title on a band below,
# written to projects/<area>/<name>/card.png, and the site's own at web/public/card.png.
# The server's head tags point links at them (web/lib/meta.mjs), so a project unfurls on
# LinkedIn, X or Slack as its picture and its name. ImageMagick and the Inter font; run from
# the repo root, by deploy.sh before the rsync, or by hand after a cover changes.
set -euo pipefail
HERE=$(cd "$(dirname "$0")/.." && pwd)
FONT=/usr/share/fonts/opentype/inter/Inter-Medium.otf
[ -f "$FONT" ] || FONT=DejaVu-Sans
BG='#0c0d0f'; FG='#f2f2f2'; MUTED='#9aa0a6'; BRAND='#0e7c86'
card() { # cover title byline out
  local cover=$1 title=$2 byline=$3 out=$4
  convert -size 1200x630 xc:"$BG" \
    \( "$cover" -resize 1200x468^ -gravity center -extent 1200x468 \) -gravity north -geometry +0+0 -composite \
    \( -size 1200x4 xc:"$BRAND" \) -gravity north -geometry +0+468 -composite \
    \( -size 1104x76 -background none -fill "$FG" -font "$FONT" -pointsize 38 -gravity west caption:"$title" \) -gravity northwest -geometry +48+492 -composite \
    \( -size 1104x30 -background none -fill "$MUTED" -font "$FONT" -pointsize 20 -gravity west caption:"$byline" \) -gravity northwest -geometry +48+576 -composite \
    -strip "$out"
}
# one value of a top-level key in a project.toml (a quoted string on one line)
val() { grep -m1 "^$2 *= *\"" "$1" | sed 's/^[^=]*= *"\(.*\)"[^"]*$/\1/' ; }
n=0
for t in "$HERE"/projects/*/*/project.toml; do
  dir=$(dirname "$t"); cover=$(val "$t" cover); title=$(val "$t" title); area=$(basename "$(dirname "$dir")"); name=$(basename "$dir")
  [ -n "$cover" ] && [ -f "$dir/$cover" ] || continue
  [ -n "$title" ] || title="$area/$name"
  card "$dir/$cover" "$title" "esim · $area/$name · a scenario, not a forecast" "$dir/card.png"
  n=$((n+1))
done
site="$HERE/projects/anselm/planet-2100/cover.png"
[ -f "$site" ] && card "$site" "esim: a whole-system simulator" "modules of science on one clock, reading real records, with ledgers that must balance" "$HERE/web/public/card.png"
echo "cards: $n projects and the site"
