# devSetup

Monorepo d'outillage de développement. Il regroupe le workspace Rust **devx** (CLI, core,
fichiers, tools et TUI) ainsi que plusieurs applications (SPA web et paquets Swift).

## Installation

Installer le binaire `devx` sur un autre poste (Linux/macOS), sans cloner le dépôt.
Le script installe Rust via rustup s'il est absent, puis compile et installe `devx`
dans `~/.cargo/bin` :

```sh
curl -fsSL https://raw.githubusercontent.com/divine0x001/devSetup/main/devx/install.sh | sh
```

Depuis un checkout local :

```sh
./devx/install.sh
```

Assure-toi que `~/.cargo/bin` est dans ton `PATH` (ajoute
`export PATH="$HOME/.cargo/bin:$PATH"` à ton `~/.bashrc` ou `~/.zshrc`).

## Layout

- `devx/` — workspace Rust (crates : `devx-cli`, `devx-core`, `devx-files`, `devx-tools`,
  `devx-tui`), configuré via `devx/devx.toml`.
- `apps/prism/` — SPA React + Vite + TypeScript.
- `apps/OpenCodeLauncher/` — Swift Package.
- `apps/SupervisorBar/` — Swift Package.

## Commandes

### devx (depuis `devx/`)

| Action | Commande |
| ------ | -------- |
| build  | `cargo build --workspace` |
| test   | `cargo test --workspace` |
| lint   | `cargo clippy --workspace --all-targets -- -D warnings` |
| fmt    | `cargo fmt --all --check` |

### prism (depuis `apps/prism/`)

| Action    | Commande |
| --------- | -------- |
| dev       | `npm run dev` |
| typecheck | `npm run typecheck` |
| build     | `npm run build` (inclut déjà le typecheck) |

### Apps Swift (depuis `apps/<App>/`)

| Action | Commande |
| ------ | -------- |
| build  | `swift build` |
| test   | `swift test` |

## Configuration devx

Le workspace Rust est piloté par `devx/devx.toml`, qui définit les tasks `build`, `test`,
`lint` et `fmt` (voir le tableau devx ci-dessus pour les commandes associées).
