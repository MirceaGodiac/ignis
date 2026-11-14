#!/usr/bin/env bash
# One-time (idempotent) setup of automatic deploys on the origin VM. Run as root from a checkout:
#
#   sudo infra/scripts/autodeploy-setup.sh            # key, known_hosts, config, units (timer off)
#   sudo infra/scripts/autodeploy-setup.sh --enable   # … and start the timer
#
# 1. Creates the read-only deploy key /etc/ignis/deploy_key (ed25519, root 0600) if missing and
#    prints its PUBLIC half: add it on GitHub (repository → Settings → Deploy keys, read access
#    only). The private half never leaves the VM.
# 2. Pins github.com's ed25519 host key in /etc/ignis/github_known_hosts, checked against the
#    fingerprint GitHub publishes (docs.github.com, "GitHub's SSH key fingerprints").
# 3. Writes /etc/ignis/autodeploy.env (defaults) if missing, installs the script as
#    /usr/local/sbin/ignis-autodeploy and the units from infra/deploy/systemd/.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
[ "$(id -u)" = 0 ] || { echo "run as root" >&2; exit 1; }
GITHUB_ED25519_FP="SHA256:+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU"

install -d -m 0755 /etc/ignis /var/lib/ignis-deploy
if [ ! -f /etc/ignis/deploy_key ]; then
  ssh-keygen -q -t ed25519 -N '' -C "ignis-origin deploy (read-only)" -f /etc/ignis/deploy_key
fi
chmod 0600 /etc/ignis/deploy_key

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
ssh-keyscan -t ed25519 github.com 2>/dev/null > "$tmp"
fp="$(ssh-keygen -lf "$tmp" | awk '{print $2}')"
[ "$fp" = "$GITHUB_ED25519_FP" ] || { echo "github.com host key $fp does not match $GITHUB_ED25519_FP" >&2; exit 1; }
install -m 0644 "$tmp" /etc/ignis/github_known_hosts

if [ ! -f /etc/ignis/autodeploy.env ]; then
  cat > /etc/ignis/autodeploy.env <<'EOF'
# /etc/ignis/autodeploy.env — read by ignis-autodeploy (infra/scripts/autodeploy.sh). No secrets.
REMOTE=git@github.com:MirceaGodiac/ignis.git
BRANCH=main
# Seconds to wait for running turns/jobs before postponing a deploy to the next timer run.
DRAIN_TIMEOUT=1800
# Free space (GB) the Docker disk needs before a build.
MIN_FREE_GB=8
EOF
fi

install -m 0755 "$root/infra/scripts/autodeploy.sh" /usr/local/sbin/ignis-autodeploy
for f in "$root"/infra/deploy/systemd/ignis-autodeploy.*; do
  install -m 0644 "$f" /etc/systemd/system/
done
systemctl daemon-reload
[ "${1:-}" = --enable ] && systemctl enable --now ignis-autodeploy.timer

echo "deploy key (add on GitHub as a read-only deploy key):"
cat /etc/ignis/deploy_key.pub
if GIT_SSH_COMMAND="ssh -i /etc/ignis/deploy_key -o IdentitiesOnly=yes -o UserKnownHostsFile=/etc/ignis/github_known_hosts -o BatchMode=yes" \
   git ls-remote "$(sed -n 's/^REMOTE=//p' /etc/ignis/autodeploy.env)" refs/heads/main >/dev/null 2>&1; then
  echo "GitHub access: ok"
else
  echo "GitHub access: not yet (add the key above, then re-run)"
fi
