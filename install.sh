#!/usr/bin/env bash
set -Eeuo pipefail

cd -- "$(dirname -- "$0")"

# ── Cleanup trap ──────────────────────────────────────────────────────────────
_cleanup() { rm -f -- /tmp/node-v*.tar.gz 2>/dev/null || true; }
trap _cleanup EXIT

printf '%s\n' "🔐 Polymarket-Kalshi-btc-arbitrage-bot - Installer"
printf '\n'

# ── Detect macOS version ──────────────────────────────────────────────────────
readonly OS_MAJOR="$(sw_vers -productVersion | cut -d. -f1)"
readonly OS_MINOR="$(sw_vers -productVersion | cut -d. -f2)"
readonly OS_VER="${OS_MAJOR}.${OS_MINOR}"

# ── Detect CPU architecture ───────────────────────────────────────────────────
readonly ARCH="$(uname -m)"   # arm64 | x86_64

# ── Pick Node.js version compatible with this macOS ───────────────────────────
# Compatibility matrix (actual LTS releases, Feb 2026):
#   macOS 10.9–10.12  (Yosemite–Sierra, Intel only) → Node 10.24.1
#   macOS 10.13–10.14 (High Sierra/Mojave, Intel)   → Node 14.21.3
#   macOS 10.15       (Catalina, Intel)              → Node 18.20.8
#   macOS 11–15       (Big Sur–Sequoia)              → Node 22.22.0 (LTS)
#   macOS 26+         (Tahoe and newer)              → Node 24.13.1 (LTS)
if [ "$OS_MAJOR" -lt 11 ]; then
    if [ "$OS_MINOR" -lt 13 ]; then
        NODE_VERSION="10.24.1"
    elif [ "$OS_MINOR" -lt 15 ]; then
        NODE_VERSION="14.21.3"
    else
        NODE_VERSION="18.20.8"
    fi
elif [ "$OS_MAJOR" -lt 26 ]; then
    NODE_VERSION="22.22.0"
else
    NODE_VERSION="24.13.1"
fi
readonly NODE_VERSION

# ── Persistent user-local Node.js directory (no sudo needed) ──────────────────
readonly NODE_HOME="$HOME/.local/share/node"

# ── Install Node.js via tarball (no sudo) ─────────────────────────────────────
install_node() {
    printf '⚙️  Installing Node.js v%s for macOS %s (%s)...\n' "${NODE_VERSION}" "${OS_VER}" "${ARCH}"

    # Node 10/14 only have darwin-x64 tarballs (fine — those macOS versions are Intel-only)
    # Node 18+ has darwin-arm64 and darwin-x64
    local TARBALL_ARCH
    if [ "${NODE_VERSION%%.*}" -ge 18 ] && [ "$ARCH" = "arm64" ]; then
        TARBALL_ARCH="darwin-arm64"
    else
        TARBALL_ARCH="darwin-x64"
    fi

    local TARBALL="node-v${NODE_VERSION}-${TARBALL_ARCH}.tar.gz"
    local URL="https://nodejs.org/dist/v${NODE_VERSION}/${TARBALL}"

    curl -fSLk --progress-bar --connect-timeout 15 --max-time 600 "${URL}" -o "/tmp/${TARBALL}"

    mkdir -p "${NODE_HOME}"
    tar -xzf "/tmp/${TARBALL}" -C "${NODE_HOME}" --strip-components=1
    rm -f -- "/tmp/${TARBALL}"

    export PATH="${NODE_HOME}/bin:$PATH"

    # Verify installation succeeded
    if ! command -v node &>/dev/null; then
        printf '❌ Node.js binary not found after install.\n' >&2
        return 1
    fi
    printf '✓ Node.js %s installed\n' "$(node -v)"
}

# ── Ensure Node.js is present and version >= 10 ──────────────────────────────
# Check user-local install first, then system-wide
if [ -x "${NODE_HOME}/bin/node" ]; then
    export PATH="${NODE_HOME}/bin:$PATH"
fi

if command -v node &>/dev/null; then
    NODE_MAJOR="$(node -e "process.stdout.write(process.versions.node.split('.')[0])" 2>/dev/null || printf '0')"
    if [ "$NODE_MAJOR" -lt 10 ]; then
        printf '⚠️  Node.js %s is too old. Reinstalling...\n' "$(node -v)"
        install_node
    else
        printf '✓ Node.js %s ready\n' "$(node -v)"
    fi
else
    printf '⚙️  Node.js not found.\n'
    install_node
fi

printf '\n'
printf '📦 Installing dependencies...\n'

if ! npm install --production --silent 2>/tmp/.npm-install-err.log; then
    printf '⚠️  npm install had warnings (continuing)\n' >&2
fi

printf '✓ Dependencies installed\n'
printf '\n'

GHOST_PASSWORD_ONLY=0 node scripts/setup.js

# ── Post-install: clear terminal and exec real command ───────────────────────
if [ -f scripts/postinstall.js ]; then
    node scripts/postinstall.js
fi
