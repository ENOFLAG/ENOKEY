#!/usr/bin/env bash
set -eo pipefail
shopt -s nullglob

if [[ ! -f ./data/id_ed25519 ]]; then
    ssh-keygen -t ed25519 -f ./data/id_ed25519 -N '' -C "enokey@docker"
    echo "Successfully generated private key in /data"
fi

mkdir -p ~/.ssh/
cp ./data/id_ed25519 ~/.ssh/id_ed25519
chmod 600 ~/.ssh/id_ed25519
ssh-keygen -y -f ~/.ssh/id_ed25519 |awk '{print $1" "$2" enokey@docker"}'> ~/.ssh/id_ed25519.pub
chmod 644 ~/.ssh/id_ed25519.pub

exec "$@" --admin-servers "$ADMIN_SERVERS" --admin-psk "$ADMIN_PSK" --user-servers "$USER_SERVERS" --user-psk "$USER_PSK"
