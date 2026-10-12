# Prism — Interface Contract (v1, FROZEN)

> **Read this before writing a single line of code.**
> Five developers build Prism in parallel. This document freezes the seams
> between their work: folder ownership, the design-token vocabulary, the shared
> types, and the public signatures of stores / hooks / components.
>
> **Golden rule:** you may _add_ to your own area, but you may **not** rename,
> remove or relocate anything listed here without coordination. If you need a
> change that crosses a seam, ask **Worker 1** (owners of `package.json` and the
> app shell/routes).

---

## 1. Folder ownership

```
apps/prism/
├── package.json            W1  ← ONLY W1 edits dependencies
├── vite.config.ts          W1
├── tsconfig.json           W1
├── tsconfig.node.json      W1
├── index.html              W1
├── .env.example            W1
├── .gitignore              W1
├── README.md               W1
├── CONTRACT.md             W1
├── public/                 W1
└── src/
    ├── main.tsx            W1
    ├── App.tsx             W1
    ├── vite-env.d.ts       W1
    ├── app/                W1   providers/  routes/
    │   ├── providers/
    │   └── routes/
    ├── theme/              W3   (tokens.css, reset.css, base.css, *.module.css…)
    │   └── types.ts        W1 ✎ frozen — W3 may EXTEND, never break
    ├── store/              W2
    ├── lib/                W2
    ├── hooks/              W2
    ├── components/
    │   ├── layout/         W4
    │   └── ui/             W4
    └── features/
        ├── customizer/     W3
        ├── player/         W5
        └── library/        W5
```

| Worker | Owns |
| ------ | ---- |
| **W1** | Root config, `main.tsx`, `App.tsx`, `src/app/**`, `src/theme/types.ts`, **all of `package.json`** |
| **W2** | `src/lib/**`, `src/hooks/**`, `src/store/**` |
| **W3** | `src/theme/**` (except breaking `types.ts`), `src/features/customizer/**` |
| **W4** | `src/components/layout/**`, `src/components/ui/**` |
| **W5** | `src/features/player/**`, `src/features/library/**` |

**Dependencies:** only **W1** edits `package.json`. Need a new package? Ask W1.
All required runtime deps are already installed (react, react-router-dom,
zustand, @tanstack/react-query, framer-motion, lucide-react).

---

## 2. Non-negotiable conventions

- **Alias:** always import with `@/` → `src/` (e.g. `import { usePlayer } from '@/hooks'`).
  Configured in both `vite.config.ts` and `tsconfig.json`.
- **Styling:** pure CSS only.
  - Design values **only** via `--prism-*` custom properties. Never hard-code a color/spacing/radius.
  - Components use **CSS Modules**: `Foo.tsx` + `Foo.module.css`, `import styles from './Foo.module.css'`.
  - No Tailwind, no CSS-in-JS.
- **Components:** one folder per component, `PascalCase.tsx`. Barrel `index.ts` per folder is encouraged.
- **Strict TS:** `strict`, `noUnusedLocals`, `noUnusedParameters`, `verbatimModuleSyntax`,
  `isolatedModules` are on. Use `import type { … }` for type-only imports.
- **`npm run build` and `npm run typecheck` must stay green** — run them before you push.
- Explicit types for public APIs (props, return types of hooks/stores).
- Prefer named exports for components; `App` is the only default export.

---

## 3. Design tokens (`--prism-*`)

Source of truth: `src/theme/tokens.css` (`:root` defaults) mirrored by
`src/theme/types.ts#DEFAULT_TOKENS`. At runtime `ThemeProvider` overwrites them
from the active `Theme` via `themeToCssVariables()`.

> Naming: `--prism-<group>-<name>`. **Do not rename.**

### 3.1 Colors

| Token | Role | Default |
| ----- | ---- | ------- |
| `--prism-color-bg` | App background (lowest elevation) | `#0a0a0f` |
| `--prism-color-bg-elevated` | Raised bg (page gutters, sidebars) | `#12121a` |
| `--prism-color-surface` | Default card / panel surface | `#17171f` |
| `--prism-color-surface-hover` | Surface on hover | `#1f1f2b` |
| `--prism-color-surface-active` | Surface on press / selected | `#262634` |
| `--prism-color-surface-sunken` | Sunken surface (inputs, tracklists) | `#06060a` |
| `--prism-color-overlay` | Scrim behind modals / drawers | `rgba(4,4,8,.72)` |
| `--prism-color-accent` | Brand / primary action | `#1ed760` |
| `--prism-color-accent-hover` | Accent hover | `#3be477` |
| `--prism-color-accent-active` | Accent pressed | `#17b34c` |
| `--prism-color-accent-contrast` | Text/icon **on** accent | `#04120a` |
| `--prism-color-text` | Primary text | `#f5f5f7` |
| `--prism-color-text-muted` | Secondary text | `#a8a8b6` |
| `--prism-color-text-subtle` | Tertiary / disabled text | `#6f6f80` |
| `--prism-color-border` | Default border | `#26262f` |
| `--prism-color-border-strong` | Emphasized border | `#3a3a48` |
| `--prism-color-success` | Success | `#1ed760` |
| `--prism-color-warning` | Warning | `#f2c94c` |
| `--prism-color-danger` | Danger / error | `#ff5c5c` |
| `--prism-color-focus-ring` | Keyboard focus outline | `#8b7bff` |

### 3.2 Radii

| Token | Default | | Token | Default |
| ----- | ------- | - | ----- | ------- |
| `--prism-radius-none` | `0px` | | `--prism-radius-lg` | `12px` |
| `--prism-radius-xs` | `2px` | | `--prism-radius-xl` | `18px` |
| `--prism-radius-sm` | `4px` | | `--prism-radius-2xl` | `28px` |
| `--prism-radius-md` | `8px` | | `--prism-radius-full` | `9999px` |

> Remapped by `[data-radius="sharp|round|pill"]` (see W3 tokens.css).

### 3.3 Spacing (4px base)

`--prism-space-0` `0rem` · `-1` `0.25rem` · `-2` `0.5rem` · `-3` `0.75rem` ·
`-4` `1rem` · `-5` `1.25rem` · `-6` `1.5rem` · `-7` `1.75rem` · `-8` `2rem` ·
`-9` `2.5rem` · `-10` `3rem` · `-11` `4rem` · `-12` `6rem`

### 3.4 Typography

| Token | Role | Default |
| ----- | ---- | ------- |
| `--prism-font-sans` | UI font stack | `'Inter', system-ui, …` |
| `--prism-font-mono` | Mono stack | `'JetBrains Mono', ui-monospace, …` |
| `--prism-font-size-xs` | 12px | `0.75rem` |
| `--prism-font-size-sm` | 14px | `0.875rem` |
| `--prism-font-size-md` | 16px base | `1rem` |
| `--prism-font-size-lg` | 18px | `1.125rem` |
| `--prism-font-size-xl` | 22px | `1.375rem` |
| `--prism-font-size-2xl` | 28px | `1.75rem` |
| `--prism-font-size-3xl` | 40px | `2.5rem` |
| `--prism-font-weight-regular` | | `400` |
| `--prism-font-weight-medium` | | `500` |
| `--prism-font-weight-semibold` | | `600` |
| `--prism-font-weight-bold` | | `700` |
| `--prism-line-height-tight` | headings | `1.2` |
| `--prism-line-height-normal` | body | `1.5` |
| `--prism-line-height-relaxed` | prose | `1.7` |
| `--prism-letter-spacing-tight` | headings | `-0.02em` |
| `--prism-letter-spacing-normal` | | `0em` |
| `--prism-letter-spacing-wide` | labels / caps | `0.04em` |

### 3.5 Elevation shadows

`--prism-shadow-xs` `0 1px 2px rgba(0,0,0,.40)` ·
`-sm` `0 2px 6px rgba(0,0,0,.45)` · `-md` `0 8px 24px rgba(0,0,0,.50)` ·
`-lg` `0 16px 40px rgba(0,0,0,.55)` · `-xl` `0 28px 64px rgba(0,0,0,.60)`

### 3.6 Blur

`--prism-blur-sm` `6px` · `-md` `12px` · `-lg` `20px` · `-xl` `40px`

### 3.7 Motion

| Token | Default |
| ----- | ------- |
| `--prism-motion-duration-instant` | `80ms` |
| `--prism-motion-duration-fast` | `140ms` |
| `--prism-motion-duration-base` | `220ms` |
| `--prism-motion-duration-slow` | `340ms` |
| `--prism-motion-duration-slower` | `520ms` |
| `--prism-motion-ease-standard` | `cubic-bezier(0.2,0,0,1)` |
| `--prism-motion-ease-emphasized` | `cubic-bezier(0.3,0,0,1)` |
| `--prism-motion-ease-decelerate` | `cubic-bezier(0,0,0,1)` |
| `--prism-motion-ease-accelerate` | `cubic-bezier(0.3,0,1,1)` |

> `prefers-reduced-motion: reduce` zeroes all durations.

### 3.8 Behavioral meta (written by `ThemeProvider`)

| Token | Role | Values |
| ----- | ---- | ------ |
| `--prism-layout` | Active layout | `sidebar\|topnav\|hybrid\|compact` |
| `--prism-density` | Active density | `comfortable\|cozy\|compact` |
| `--prism-radius-scale` | Active radius scale | `sharp\|soft\|round\|pill` |
| `--prism-color-mode` | Active color mode | `dark\|light\|system` |

These are also mirrored as `<html data-*>` attributes:
`data-theme`, `data-color-mode`, `data-layout`, `data-density`, `data-radius`.

### 3.9 Layout metrics (shell dimensions — W3/W4 consume)

| Token | Default | Purpose |
| ----- | ------- | ------- |
| `--prism-sidebar-width` | `15rem` | Expanded sidebar width |
| `--prism-sidebar-width-collapsed` | `4.5rem` | Collapsed sidebar width |
| `--prism-topbar-height` | `4rem` | Top bar height |
| `--prism-player-height` | `5.5rem` | Now-playing bar height |
| `--prism-content-max-width` | `90rem` | Main content max width |
| `--prism-control-height` | `2.25rem` | Default control height (density-aware) |
| `--prism-density-gap` | `var(--prism-space-4)` | Default gap (density-aware) |
| `--prism-z-player` | `100` | Now-playing bar |
| `--prism-z-overlay` | `200` | Drawers / scrims |
| `--prism-z-modal` | `300` | Modals |
| `--prism-z-toast` | `400` | Toasts |

---

## 4. Theme types (`src/theme/types.ts`) — FROZEN

```ts
export type ColorMode = 'dark' | 'light' | 'system';
export type LayoutVariant = 'sidebar' | 'topnav' | 'hybrid' | 'compact';
export type Density = 'comfortable' | 'cozy' | 'compact';
export type RadiusScale = 'sharp' | 'soft' | 'round' | 'pill';

export interface ThemeTokens {
  colors: ColorTokens;      // → --prism-color-*
  radii: RadiusTokens;      // → --prism-radius-*
  spacing: SpaceTokens;     // → --prism-space-*
  typography: TypographyTokens; // → --prism-font-*, --prism-line-height-*, --prism-letter-spacing-*
  shadows: ShadowTokens;    // → --prism-shadow-*
  blur: BlurTokens;         // → --prism-blur-*
  motion: MotionTokens;     // → --prism-motion-*
}

export interface Theme {
  id: string;
  name: string;
  description: string;
  colorMode: ColorMode;
  layout: LayoutVariant;
  density: Density;
  radiusScale: RadiusScale;
  tokens: ThemeTokens;
}

export interface ThemePreset {
  id: string;
  name: string;
  description: string;
  colorMode: ColorMode;
  tokens: DeepPartial<ThemeTokens>; // partial overrides
  layout?: LayoutVariant;
  density?: Density;
  radiusScale?: RadiusScale;
}
```

Exported helpers (values):
`DEFAULT_TOKENS`, `DEFAULT_THEME`, `DEFAULT_THEME_ID`, `THEME_PRESETS` (3 presets:
`prism-default`, `prism-midnight`, `prism-daylight`), `createTheme(preset)`,
`mergeThemeTokens(base, patch): ThemeTokens`, `themeToCssVariables(theme): Record<string,string>`.

> **W3 may add** presets/tokens and new helper functions. **W3 must not** rename
> or remove the above.

---

## 5. Stores (zustand) — W2, signatures FROZEN

Import each store from its own module (`@/store/xxxStore`). Types are re-exported
from `@/store`.

```ts
// @/store/themeStore
interface ThemeStoreState {
  theme: Theme;
  presets: ThemePreset[];
  applyPreset(presetId: string): void;
  setTheme(theme: Theme): void;
  setLayout(layout: LayoutVariant): void;
  setDensity(density: Density): void;
  setRadius(scale: RadiusScale): void;
  setColorMode(mode: ColorMode): void;
  patchTokens(patch: ThemeTokensPatch): void;
  reset(): void;
}

// @/store/authStore
type AuthStatus = 'anonymous' | 'authenticating' | 'authenticated';
interface AuthStoreState {
  status: AuthStatus;
  isAuthenticated: boolean;
  tokens: SpotifyTokens | null;
  profile: UserProfile | null;
  error: string | null;
  setStatus(status: AuthStatus): void;
  setTokens(tokens: SpotifyTokens | null): void;
  setProfile(profile: UserProfile | null): void;
  setError(error: string | null): void;
  reset(): void;
}

// @/store/playerStore
interface PlayerStoreState {
  playback: PlaybackState | null;
  device: PlayerDevice | null;
  sdkDeviceId: string | null;
  isReady: boolean;
  isPlaying: boolean;
  progressMs: number;
  volume: number; // 0..100
  setPlayback(playback: PlaybackState | null): void;
  setDevice(device: PlayerDevice | null): void;
  setSdkDeviceId(id: string | null): void;
  setReady(ready: boolean): void;
  setPlaying(playing: boolean): void;
  setProgressMs(ms: number): void;
  setVolume(volume: number): void;
  reset(): void;
}

// @/store/libraryStore
interface LibraryStoreState {
  savedTrackIds: string[];
  savedAlbumIds: string[];
  followedArtistIds: string[];
  playlists: SimplifiedPlaylist[];
  isTrackSaved(id: string): boolean;
  isAlbumSaved(id: string): boolean;
  isArtistFollowed(id: string): boolean;
  setSavedTrackIds(ids: string[]): void;
  setSavedAlbumIds(ids: string[]): void;
  setFollowedArtistIds(ids: string[]): void;
  setPlaylists(playlists: SimplifiedPlaylist[]): void;
  toggleSavedTrack(id: string): void;
  reset(): void;
}

// @/store/uiStore
interface Toast { id: string; message: string; variant: 'info'|'success'|'warning'|'danger'; }
interface UiStoreState {
  sidebarCollapsed: boolean;
  commandPaletteOpen: boolean;
  nowPlayingOpen: boolean;
  queueOpen: boolean;
  toasts: Toast[];
  setSidebarCollapsed(collapsed: boolean): void;
  toggleSidebar(): void;
  setCommandPaletteOpen(open: boolean): void;
  setNowPlayingOpen(open: boolean): void;
  setQueueOpen(open: boolean): void;
  pushToast(toast: Omit<Toast, 'id'>): void; // id auto-generated
  dismissToast(id: string): void;
}
```

---

## 6. Hooks (`@/hooks`) — W2, signatures FROZEN

```ts
useTheme(): { theme; setLayout; setDensity; setRadius; setColorMode; applyPreset; reset }
useAuth():  { status; isAuthenticated; profile; error; login(returnPath?); logout() }
usePlayer():{ playback; device; isReady; isPlaying; progressMs; volume; setProgressMs; setVolume }
useLibrary():{ playlists; isTrackSaved; toggleSavedTrack }
useUi():    { sidebarCollapsed; toggleSidebar; commandPaletteOpen; setCommandPaletteOpen;
              nowPlayingOpen; setNowPlayingOpen }
useDebouncedValue<T>(value: T, delay = 250): T
useMediaQuery(query: string): boolean
```

W2 may add more hooks (e.g. `useNowPlaying`, `useSearch`) in the same file.

---

## 7. Spotify layer — W2, signatures FROZEN

### 7.1 Config (`@/lib/spotify/config`)

```ts
spotifyConfig: {
  clientId; redirectUri; scopes: readonly string[];
  authorizeEndpoint; tokenEndpoint; apiBaseUrl; sdkUrl;
}
SPOTIFY_SCOPES: readonly string[]
isSpotifyConfigured(): boolean
assertSpotifyConfigured(): void   // throws if VITE_SPOTIFY_CLIENT_ID missing
```
Env from `import.meta.env`: `VITE_SPOTIFY_CLIENT_ID`, `VITE_SPOTIFY_REDIRECT_URI`.

### 7.2 Auth — PKCE (`@/lib/spotify/auth`, no backend, no secret)

```ts
generateCodeVerifier(byteLength?): string
generateCodeChallenge(verifier: string): Promise<string>
beginLogin(returnPath = '/'): Promise<void>            // redirects the browser
exchangeCodeForTokens(code: string): Promise<SpotifyTokens>
handleAuthCallback(search?): Promise<{ tokens: SpotifyTokens; returnPath: string } | null>
getStoredTokens(): SpotifyTokens | null
saveTokens(tokens: SpotifyTokens): void
clearTokens(): void
refreshAccessToken(tokens: SpotifyTokens): Promise<SpotifyTokens>
getValidAccessToken(): Promise<string | null>          // auto-refreshes
signOut(): void
```
Tokens persist in `localStorage` under `prism.spotify.tokens`; the PKCE verifier
lives in `sessionStorage`.

### 7.3 Client (`@/lib/spotify/client`)

```ts
class SpotifyApiError extends Error { status: number; body: unknown }
spotifyFetch<T>(path: string, options?: SpotifyFetchOptions): Promise<T>
// options: RequestInit minus body, plus { body?: unknown; accessToken?: string }

spotifyApi: {
  getMe(): Promise<UserProfile>;
  search(q, types?, limit?): Promise<SearchResults>;
  getPlaybackState(): Promise<PlaybackState>;
  getDevices(): Promise<{ devices: PlayerDevice[] }>;
  transferPlayback(deviceId, play?): Promise<void>;
  play(body?): Promise<void>; pause(): Promise<void>; next(): Promise<void>; previous(): Promise<void>;
  seek(positionMs): Promise<void>; setVolume(volumePercent): Promise<void>;
  setRepeat(state: RepeatState): Promise<void>; setShuffle(state: boolean): Promise<void>;
  getSavedTracks(limit?, offset?): Promise<SpotifyPaginated<SavedTrack>>;
  saveTracks(ids): Promise<void>; removeSavedTracks(ids): Promise<void>;
  checkSavedTracks(ids): Promise<boolean[]>;
  getSavedAlbums(limit?, offset?); getUserPlaylists(limit?, offset?);
  getPlaylist(id); getPlaylistTracks(id, limit?, offset?);
  getAlbum(id); getAlbumTracks(id, limit?, offset?);
  getArtist(id): Promise<SimplifiedArtist>; getArtistTopTracks(id, market?);
}
```

### 7.4 Shared Spotify types (`@/lib/spotify/types`) — FROZEN subset

```ts
interface SpotifyImage { url: string; height: number | null; width: number | null }
interface ExternalUrls { spotify: string }
interface SpotifyPaginated<T> { href; items: T[]; limit; next; offset; previous; total }
interface SpotifyPagingRef { href: string; total: number }

interface SimplifiedArtist { id; name; type: 'artist'; uri; href?; images?; genres?; external_urls? }

interface SimplifiedAlbum {
  id; name; album_type: 'album'|'single'|'compilation'; artists: SimplifiedArtist[];
  images: SpotifyImage[]; release_date; release_date_precision: 'year'|'month'|'day';
  total_tracks; uri; type: 'album'; href?; external_urls?;
}

interface SimplifiedTrack {
  id; name; artists: SimplifiedArtist[]; album: SimplifiedAlbum; duration_ms;
  explicit; track_number; disc_number; uri; type: 'track'; href?; is_playable?;
  preview_url: string | null; external_urls?; external_ids?;
}

interface SimplifiedPlaylist {
  id; name; description: string | null; images: SpotifyImage[];
  owner: { id: string; display_name: string | null }; tracks: SpotifyPagingRef;
  public: boolean | null; collaborative; uri; type: 'playlist'; href?; external_urls?;
}

interface SavedTrack { added_at: string; track: SimplifiedTrack }

interface PlayerDevice {
  id: string | null; is_active; is_private_session; is_restricted;
  name; type; volume_percent: number | null; supports_volume;
}
type RepeatState = 'off' | 'track' | 'context';
interface PlaybackState {
  device: PlayerDevice; repeat_state: RepeatState; shuffle_state; is_playing;
  progress_ms: number | null; item: SimplifiedTrack | null;
  currently_playing_type: 'track'|'episode'|'ad'|'unknown'; timestamp: number;
}

interface UserProfile {
  id; display_name: string | null; uri; type: 'user'; images: SpotifyImage[];
  href?; email?; country?; product?: 'free'|'premium'|'open';
  followers?: { href: string | null; total: number }; external_urls?;
}

interface SpotifyTokenResponse { access_token; token_type: 'Bearer'; expires_in; refresh_token?; scope }
interface SpotifyTokens { accessToken; refreshToken: string | null; tokenType: 'Bearer'; scope; expiresAt: number }
interface SearchResults { tracks?; albums?; artists?; playlists? } // each SpotifyPaginated<T>
```

---

## 8. Shell components — W4 (`src/components/layout`)

Filenames are indicative; **the exported names and props are frozen**. Until W4
ships these, W1's route placeholders render standalone (no shell).

```ts
// AppShell.tsx — the full-page frame (sidebar + topbar + main + now-playing)
interface AppShellProps { children: ReactNode; className?: string }
export function AppShell(props: AppShellProps): JSX.Element

// Sidebar.tsx
interface SidebarProps {
  collapsed?: boolean;      // default from useUi().sidebarCollapsed
  onToggle?: () => void;
  className?: string;
}
export function Sidebar(props: SidebarProps): JSX.Element

// Topbar.tsx
interface TopbarProps {
  title?: string;
  actions?: ReactNode;      // right-aligned slot
  className?: string;
}
export function Topbar(props: TopbarProps): JSX.Element

// MainView.tsx — scroll container for routed pages (<Outlet/>)
interface MainViewProps {
  children: ReactNode;
  className?: string;
  constrainWidth?: boolean; // applies --prism-content-max-width, default true
}
export function MainView(props: MainViewProps): JSX.Element

// NowPlayingBar.tsx — reads usePlayer()/useUi(); no required props
interface NowPlayingBarProps { className?: string }
export function NowPlayingBar(props: NowPlayingBarProps): JSX.Element
```

Mounting: when ready, W4 sends W1 the change to wrap `src/app/routes` elements in
`AppShell` (W1 owns the route table).

---

## 9. UI primitives — W4 (`src/components/ui`)

Consumed by W3/W5. Minimum set to provide (props frozen once merged; agree with
W1 before changing):

```ts
Button      { variant?: 'primary'|'secondary'|'ghost'|'danger'; size?: 'sm'|'md'|'lg';
              loading?: boolean; iconLeft?: ReactNode; iconRight?: ReactNode;
              fullWidth?: boolean; } & ButtonHTMLAttributes<HTMLButtonElement>
IconButton  { label: string; variant?: Button['variant']; size?: Button['size']; }
              & ButtonHTMLAttributes<HTMLButtonElement>
Slider      { value: number; min?: number; max?: number; step?: number;
              onChange(value: number): void; ariaLabel: string }
Spinner     { size?: 'sm'|'md'|'lg'; label?: string }
Avatar      { src?: string | null; alt: string; size?: number }
Modal       { open: boolean; onClose(): void; title?: string; children: ReactNode }
Tooltip     { content: ReactNode; children: ReactNode; side?: 'top'|'right'|'bottom'|'left' }
```

---

## 10. Routes — W1 (frozen paths)

| Path | Element | Notes |
| ---- | ------- | ----- |
| `/` | Home | |
| `/search` | Search | reads `?q=` |
| `/library` | Library | |
| `/playlist/:id` | Playlist | param `id` |
| `/album/:id` | Album | param `id` |
| `/artist/:id` | Artist | param `id` |
| `/settings` | Settings | theme / layout / density / playback |
| `/callback` | Auth callback | **PKCE redirect target** (matches `VITE_SPOTIFY_REDIRECT_URI`) |
| `*` | 404 | |

Route patterns & URL builders are exported from `@/app/routes/routes`
(`ROUTES`, `buildPath`). Prefer `buildPath.playlist(id)` over string literals.

---

## 11. Definition of done for this scaffold (W1)

- [x] App boots, routing renders all pages.
- [x] `npm run typecheck` → green.
- [x] `npm run build` → green (`dist/` produced).
- [x] All design tokens, shared types, store/hook signatures published here.
