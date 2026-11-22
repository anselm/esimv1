#!/usr/bin/env bash
# Deploy the read-only copy: build the viewer here, upload every project's runs to the bucket,
# rsync the built pages, the server and its dependencies, the projects without their runs, and
# the docs to /srv/esim on the VM, then restart the unit. The VM reads runs from the bucket
# (web/lib/store.mjs). Push is not deploy; this is run by hand on Anselm's say.
#
#   deploy/deploy.sh esimv1
#   STORE_FIRST=1 deploy/deploy.sh esimv1    the VM asks the bucket before its own disk (to test the bucket while runs are still on the VM)
#   PRUNE_RUNS=1 deploy/deploy.sh esimv1     also remove run folders from the VM, once the bucket is known to serve them
set -euo pipefail
VM=${1:?vm name}
HERE=$(cd "$(dirname "$0")/.." && pwd)
HOST="exedev@$VM.exe.xyz"
RS="rsync -az --delete -e 'ssh -o ServerAliveInterval=5 -o StrictHostKeyChecking=accept-new'"

# the engine, the viewer and the scenarios kept in their own repositories must be here; say what version each is at
"$HERE/deploy/parts.sh"
"$HERE/deploy/cards.sh" >/dev/null
(cd "$HERE/web" && npm run build >/dev/null)
# the static fallbacks the read-only server reads: kinds, projects index, outlines
(cd "$HERE" && ./core/target/release/esim kinds --json > docs/kinds.json && ./core/target/release/esim projects --json > projects/index.json && ./core/target/release/esim sources --write >/dev/null && ./core/target/release/esim events --write >/dev/null)

# every project's runs to the bucket, gzipped, less snapshots (deploy/store.py; boto3 lives in a private environment)
STORE_PY=${STORE_PY:-$HOME/.cache/sausalito-venv/bin/python}
for meta in "$HERE"/projects/*/*/project.toml; do
  p=${meta#"$HERE"/projects/}; "$STORE_PY" "$HERE/deploy/store.py" put "${p%/project.toml}"
done
# the VM's own copy of the credentials (only what a reader of objects needs), and the unit told where it is
grep -E '^(access_key_id|secret_access_key|endpoint|bucket)=' "$HOME/.config/cf/r2" | ssh -o ServerAliveInterval=5 -o StrictHostKeyChecking=accept-new "$HOST" 'umask 077 && cat > /srv/esim/store.conf'
ssh -o ServerAliveInterval=5 "$HOST" "sudo mkdir -p /etc/systemd/system/esim.service.d && printf '[Service]\nEnvironment=ESIM_STORE=/srv/esim/store.conf\n${STORE_FIRST:+Environment=ESIM_STORE_FIRST=1\n}' | sudo tee /etc/systemd/system/esim.service.d/store.conf >/dev/null && sudo systemctl daemon-reload"

ssh -o ServerAliveInterval=5 -o StrictHostKeyChecking=accept-new "$HOST" 'mkdir -p /srv/esim/web'
eval $RS "$HERE/web/dist" "$HERE/web/lib" "$HERE/web/server.mjs" "$HERE/web/package.json" "$HOST:/srv/esim/web/"
# Project folders without their run folders: the runs are in the bucket, and each runs/index.json
# stays so the pages can list them. Run folders already on the VM are left alone unless PRUNE_RUNS=1.
eval $RS ${PRUNE_RUNS:+--delete-excluded} --exclude="'runs/*/'" --exclude='data/' --copy-links "$HERE/projects/" "$HOST:/srv/esim/projects/"
eval $RS "$HERE/docs/" "$HOST:/srv/esim/docs/"
# the engine's own documentation, which the server reads from the engine's folder
ssh -o ServerAliveInterval=5 "$HOST" 'mkdir -p /srv/esim/core'
eval $RS "$HERE/core/docs" "$HOST:/srv/esim/core/"
# the server reads these for the country picker and outlines when the geojson cache is absent
# dependencies from package.json on the VM (socket.io and the SDK), then restart
ssh -o ServerAliveInterval=5 "$HOST" 'cd /srv/esim/web && npm install --omit=dev --no-audit --no-fund >/dev/null && sudo systemctl restart esim.service && sleep 2 && systemctl is-active esim.service'
echo "https://$VM.exe.xyz/"
