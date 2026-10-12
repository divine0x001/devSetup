# Prism

An **ultra-customizable Spotify web player** — 100% client-side (no backend),
themed with pure CSS variables and CSS Modules, powered by the Spotify Web API
(Authorization Code + **PKCE**) and the Web Playback SDK.

> This package is intentionally **outside** the repository's Cargo workspace
> (`crates/*`). It is a standalone Vite + React app.

## Stack

| Concern    | Choice                                             |
| ---------- | -------------------------------------------------- |
| Build      | Vite 5                                             |
| UI         | React 18 + TypeScript (strict)                     |
| Routing    | react-router-dom v6                                |
| State      | zustand                                            |
| Data       | @tanstack/react-query                              |
| Animation  | framer-motion                                      |
| Icons      | lucide-react                                       |
| Styling    | Pure CSS: design tokens + CSS Modules (**no Tailwind**) |

## Getting started

```bash
# from apps/prism
npm install
cp .env.example .env.local   # then fill in VITE_SPOTIFY_CLIENT_ID
npm run dev                  # http://127.0.0.1:5173
```

### Spotify setup

1. Create an app at <https://developer.spotify.com/dashboard>.
2. Add the redirect URI **exactly** as in `.env.local`
   (default: `http://127.0.0.1:5173/callback` — Spotify requires the loopback IP,
   not `localhost`).
3. Copy the Client ID into `.env.local` as `VITE_SPOTIFY_CLIENT_ID`.

No client secret is needed or used.

## Scripts

| Script              | What it does                                        |
| ------------------- | --------------------------------------------------- |
| `npm run dev`       | Start the Vite dev server                           |
| `npm run build`     | Type-check then produce a production build in `dist`|
| `npm run preview`   | Preview the production build                         |
| `npm run typecheck` | `tsc --noEmit` for both the app and the Vite config |

## Project layout

```
src/
  app/        providers + routes              (Worker 1)
  theme/      design tokens + theme contract (Worker 3, types.ts frozen by W1)
  store/      zustand stores                  (Worker 2)
  lib/        spotify client + auth (PKCE)    (Worker 2)
  hooks/      public hooks                    (Worker 2)
  components/ layout + ui primitives          (Worker 4)
  features/   player, library, customizer     (Workers 3 & 5)
```

**Read [`CONTRACT.md`](./CONTRACT.md)** before writing code: it freezes the
folder ownership, the `--prism-*` token list, the shared types and the public
component/hook/store signatures so several developers can work in parallel
without conflicts.

## Conventions

- Import alias `@/` → `src/`.
- One component per folder, `PascalCase.tsx` + `*.module.css`.
- Style exclusively through `--prism-*` tokens — never hard-code colors.
- Do **not** edit `package.json` unless you are Worker 1 (dependency owner).
