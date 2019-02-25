#!/usr/bin/env bash
set -eo pipefail
shopt -s nullglob

if [[ ! -f ./data/id_ed25519 ]]; then
    ssh-keygen -t ed25519 -m pem -f ./data/id_ed25519
fi

mkdir -p ~/.ssh/
cp ./data/id_ed25519 ~/.ssh/id_ed25519
chmod 600 ~/.ssh/id_ed25519
ssh-keygen -y -f ~/.ssh/id_ed25519 > ~/.ssh/id_ed25519.pub
chmod 644 /.ssh/id_ed25519.pub

exec "$@"