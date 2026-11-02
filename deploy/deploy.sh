#!/usr/bin/env bash
# Deploy the read-only copy: build the viewer here, rsync the built pages, the server and
# its dependencies, the projects with their runs, and the docs to /srv/esim on the VM,
# then restart the unit. Push is not deploy; this is run by hand on Anselm's say.
#
#   deploy/deploy.sh esimv1
set -euo pipefail
VM=${1:?vm name}
HERE=$(cd "$(dirname "$0")/.." && pwd)
HOST="exedev@$VM.exe.xyz"
RS="rsync -az --delete -e 'ssh -o ServerAliveInterval=5'"

(cd "$HERE/web" && npm run build >/dev/null)
# the static fallbacks the read-only server reads: kinds, projects index, outlines
(cd "$HERE" && ./target/release/esim kinds --json > docs/kinds.json && ./target/release/esim projects --json > projects/index.json)

ssh -o ServerAliveInterval=5 "$HOST" 'mkdir -p /srv/esim/web'
eval $RS "$HERE/web/dist" "$HERE/web/server.mjs" "$HERE/web/package.json" "$HOST:/srv/esim/web/"
eval $RS --exclude='fields/*/*.f32.tmp' "$HERE/projects/" "$HOST:/srv/esim/projects/"
eval $RS "$HERE/docs/" "$HOST:/srv/esim/docs/"
# the server reads these for the country picker and outlines when the geojson cache is absent
# dependencies from package.json on the VM (socket.io and the SDK), then restart
ssh -o ServerAliveInterval=5 "$HOST" 'cd /srv/esim/web && npm install --omit=dev --no-audit --no-fund >/dev/null && sudo systemctl restart esim.service && sleep 2 && systemctl is-active esim.service'
echo "https://$VM.exe.xyz/"
