#!/usr/bin/env bash
# The hub's parts: the engine, the viewer and the scenarios kept in their own repositories
# (parts.list). For each: clone it when it is missing, link a scenario's project folders into
# projects/, and say which version is checked out against the version the list names.
#
#   deploy/parts.sh          clone what is missing, make the links, report
#   deploy/parts.sh --pin    also check out the listed version of each part; refuses a part
#                            with uncommitted changes
#
# A working copy is usually on a branch ahead of the listed tag; that is reported, not changed.
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
PIN=0; [ "${1:-}" = "--pin" ] && PIN=1
grep -v '^\s*#' "$HERE/parts.list" | grep -v '^\s*$' | while read -r dir repo version links; do
  path="$HERE/$dir"
  if [ ! -d "$path/.git" ]; then
    mkdir -p "$(dirname "$path")"
    git clone -q "https://github.com/$repo.git" "$path"
    git -C "$path" -c advice.detachedHead=false checkout -q "$version"
    echo "cloned   $dir from $repo at $version"
  elif [ "$PIN" = 1 ]; then
    if [ -n "$(git -C "$path" status --porcelain)" ]; then echo "refused  $dir has uncommitted changes" >&2; exit 1; fi
    git -C "$path" fetch -q --tags
    git -C "$path" -c advice.detachedHead=false checkout -q "$version"
  fi
  at="$(git -C "$path" describe --tags --always --dirty 2>/dev/null || echo unknown)"
  if [ "$at" = "$version" ]; then echo "ok       $dir at $version"; else echo "differs  $dir is at $at, the list says $version"; fi
  for link in $links; do
    from="${link%%=*}"; to="$HERE/projects/${link##*=}"
    mkdir -p "$(dirname "$to")"
    if [ -L "$to" ] || [ ! -e "$to" ]; then ln -sfn "$(realpath --relative-to="$(dirname "$to")" "$path/$from")" "$to"
    else echo "refused  $to exists and is not a link" >&2; exit 1; fi
  done
done
