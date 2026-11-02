#!/usr/bin/env bash
# Provision an exe.dev VM to serve a read-only copy of esim: Node, a system user's
# folder under /srv/esim, a systemd unit running web/server.mjs with ESIM_READONLY=1,
# and the port shared publicly through exe.dev's proxy (HTTPS at https://<vm>.exe.xyz/).
# No Rust and no data cache: a read-only copy serves files the last runs wrote.
#
#   deploy/provision.sh esimv1
set -euo pipefail
VM=${1:?vm name}
# direct ssh, not the gateway relay: the relay flattens newlines
SSH="ssh -o ServerAliveInterval=5 -o StrictHostKeyChecking=accept-new exedev@$VM.exe.xyz"

$SSH 'bash -s' <<'REMOTE'
set -e
command -v node >/dev/null || { curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash - && sudo apt-get install -y nodejs; }
sudo mkdir -p /srv/esim && sudo chown -R $USER:$USER /srv/esim
sudo tee /etc/systemd/system/esim.service >/dev/null <<UNIT
[Unit]
Description=esim read-only viewer and project server
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=/srv/esim/web
Environment=ESIM_READONLY=1
Environment=PORT=5181
ExecStart=/usr/bin/node server.mjs
Restart=on-failure
RestartSec=3

[Install]
WantedBy=multi-user.target
UNIT
sudo systemctl daemon-reload
sudo systemctl enable esim.service
node --version
REMOTE
ssh -o ServerAliveInterval=5 exe.dev share port "$VM" 5181
ssh -o ServerAliveInterval=5 exe.dev share set-public "$VM"
echo "provisioned $VM; now deploy/deploy.sh $VM"
