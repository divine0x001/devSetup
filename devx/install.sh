#!/bin/sh
# Installe devx (binaire `devx`) depuis GitHub.
#
# Usage (dépôt complet devSetup, workspace Rust dans devx/) :
#   curl -fsSL https://raw.githubusercontent.com/divine0x001/devSetup/main/devx/install.sh | sh
#
# Variables (optionnelles) :
#   DEVX_REPO   dépôt GitHub "user/repo"      (défaut : divine0x001/devSetup)
#   DEVX_REF    branche ou tag                (défaut : main)
#   DEVX_SUBDIR sous-dossier du workspace Rust (défaut : devx ; vide si à la racine)
#
# Lancé depuis un checkout local (à côté de Cargo.toml), installe sans réseau.

set -eu

DEVX_REPO="${DEVX_REPO:-divine0x001/devSetup}"
DEVX_REF="${DEVX_REF:-main}"
DEVX_SUBDIR="${DEVX_SUBDIR:-devx}"
CRATE_PATH="crates/devx-cli"

log() { printf '%s\n' "$*" >&2; }

ensure_rust() {
    if command -v cargo >/dev/null 2>&1; then
        return
    fi
    log "→ Rust introuvable, installation via rustup…"
    curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain stable
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
}

install_from_git() {
    if ! command -v git >/dev/null 2>&1; then
        log "error: git est requis pour installer depuis GitHub" >&2
        exit 1
    fi
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT INT TERM
    log "→ récupération https://github.com/${DEVX_REPO}.git (${DEVX_REF})…"
    git clone --depth 1 --branch "$DEVX_REF" \
        "https://github.com/${DEVX_REPO}.git" "$tmp/repo"
    log "→ compilation (release)…"
    cargo install --path "$tmp/repo/${DEVX_SUBDIR:+$DEVX_SUBDIR/}${CRATE_PATH}" --force
}

# Détecte un checkout local (script lancé en tant que fichier, à côté du workspace).
script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" 2>/dev/null && pwd)" || script_dir=""
if [ -n "$script_dir" ] && [ -f "$script_dir/Cargo.toml" ]; then
    ensure_rust
    log "→ installation locale depuis ${script_dir}…"
    cargo install --path "$script_dir/${CRATE_PATH}" --force
else
    ensure_rust
    install_from_git
fi

if command -v devx >/dev/null 2>&1; then
    log "✓ devx installé : $(command -v devx)"
else
    log "✓ devx installé — assure-toi que ~/.cargo/bin est dans ton PATH"
fi
