/**
 * Route contract (FROZEN).
 *
 * `ROUTES` holds the route *patterns* (with params) — use them to build
 * navigation. `buildPath` holds typed helpers to generate concrete URLs.
 */
export const ROUTES = {
  home: '/',
  search: '/search',
  library: '/library',
  playlist: '/playlist/:id',
  album: '/album/:id',
  artist: '/artist/:id',
  settings: '/settings',
  /** OAuth (PKCE) redirect target. Not in the product nav. */
  authCallback: '/callback',
  notFound: '*',
} as const;

export type RouteKey = keyof typeof ROUTES;

export const buildPath = {
  home: () => '/',
  search: (query?: string) =>
    query ? `/search?q=${encodeURIComponent(query)}` : '/search',
  library: () => '/library',
  playlist: (id: string) => `/playlist/${encodeURIComponent(id)}`,
  album: (id: string) => `/album/${encodeURIComponent(id)}`,
  artist: (id: string) => `/artist/${encodeURIComponent(id)}`,
  settings: () => '/settings',
} as const;
