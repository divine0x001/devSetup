/**
 * Prism — Typed Spotify Web API wrapper (100% client-side).
 *
 * Every call goes through `spotifyFetch`, which attaches the current access
 * token (refreshing it when needed) and normalizes errors into
 * `SpotifyApiError`. A 401 triggers a single refresh-and-retry before giving up.
 *
 * Signatures are FROZEN; Worker 2 may add methods but not change existing ones.
 */

import { getStoredTokens, getValidAccessToken, refreshAccessToken } from './auth';
import { spotifyConfig } from './config';
import { demoFetch, isDemoMode } from './demo';

import type {
  PlaybackState,
  PlayerDevice,
  RepeatState,
  SavedTrack,
  SearchResults,
  SimplifiedAlbum,
  SimplifiedArtist,
  SimplifiedPlaylist,
  SimplifiedTrack,
  SpotifyPaginated,
  UserProfile,
} from './types';

/* -------------------------------------------------------------------------- */
/* Errors                                                                      */
/* -------------------------------------------------------------------------- */

export class SpotifyApiError extends Error {
  readonly status: number;
  readonly body: unknown;

  constructor(message: string, status: number, body?: unknown) {
    super(message);
    this.name = 'SpotifyApiError';
    this.status = status;
    this.body = body;
  }
}

/* -------------------------------------------------------------------------- */
/* Core fetch                                                                  */
/* -------------------------------------------------------------------------- */

export interface SpotifyFetchOptions extends Omit<RequestInit, 'body'> {
  body?: unknown;
  /** Explicit token override (skips the store lookup). */
  accessToken?: string;
}

const NO_CONTENT = 204;

/**
 * Fetches a Spotify Web API path (e.g. "/me/player") and parses JSON.
 * Throws `SpotifyApiError` on non-2xx responses.
 */
export async function spotifyFetch<T>(
  path: string,
  options: SpotifyFetchOptions = {},
): Promise<T> {
  if (isDemoMode()) {
    return demoFetch(path, options) as T;
  }

  const { accessToken, body, headers, ...rest } = options;

  const token = accessToken ?? (await getValidAccessToken());
  if (!token) {
    throw new SpotifyApiError('Prism: not authenticated.', 401);
  }

  const url = path.startsWith('http') ? path : `${spotifyConfig.apiBaseUrl}${path}`;

  const request = (bearer: string): Promise<Response> =>
    fetch(url, {
      ...rest,
      headers: {
        Authorization: `Bearer ${bearer}`,
        ...(body !== undefined ? { 'Content-Type': 'application/json' } : {}),
        ...headers,
      },
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });

  let response = await request(token);

  // A 401 means the server rejected the token (revoked, or expired between our
  // local check and the request). Refresh once and retry; on failure we fall
  // through to the error path below with the original 401.
  if (response.status === 401 && !accessToken) {
    const stored = getStoredTokens();
    if (stored?.refreshToken) {
      try {
        const refreshed = await refreshAccessToken(stored);
        response = await request(refreshed.accessToken);
      } catch {
        // Refresh failed — keep the 401 response for the error path below.
      }
    }
  }

  if (!response.ok) {
    let errorBody: unknown;
    try {
      errorBody = await response.json();
    } catch {
      errorBody = undefined;
    }
    throw new SpotifyApiError(
      `Spotify API error ${response.status} on ${path}`,
      response.status,
      errorBody,
    );
  }

  if (response.status === NO_CONTENT || response.headers.get('content-length') === '0') {
    return undefined as T;
  }

  return (await response.json()) as T;
}

/* -------------------------------------------------------------------------- */
/* Pagination helper                                                           */
/* -------------------------------------------------------------------------- */

/**
 * Fetches every page of a paginated endpoint into a single array.
 * Stops when the API reports no more items or `maxItems` is reached.
 */
export async function fetchAllPages<T>(
  fetchPage: (offset: number) => Promise<SpotifyPaginated<T>>,
  _pageSize = 50,
  maxItems = 1000,
): Promise<T[]> {
  const items: T[] = [];
  let offset = 0;
  while (offset < maxItems) {
    const page = await fetchPage(offset);
    items.push(...page.items);
    if (items.length >= page.total || page.items.length === 0 || !page.next) break;
    offset += page.items.length;
  }
  return items;
}

/* -------------------------------------------------------------------------- */
/* Query-string helper                                                         */
/* -------------------------------------------------------------------------- */

function query(params: Record<string, string | number | boolean | undefined>): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined) search.set(key, String(value));
  }
  const qs = search.toString();
  return qs ? `?${qs}` : '';
}

/* -------------------------------------------------------------------------- */
/* API surface (FROZEN subset)                                                 */
/* -------------------------------------------------------------------------- */

export const spotifyApi = {
  /* --- User ------------------------------------------------------------- */
  getMe: () => spotifyFetch<UserProfile>('/me'),

  /* --- Search ----------------------------------------------------------- */
  search: (q: string, types = 'track,album,artist,playlist', limit = 20) =>
    spotifyFetch<SearchResults>(
      `/search${query({ q, type: types, limit })}`,
    ),

  /* --- Playback --------------------------------------------------------- */
  getPlaybackState: () => spotifyFetch<PlaybackState>('/me/player'),
  getDevices: () =>
    spotifyFetch<{ devices: PlayerDevice[] }>('/me/player/devices'),
  transferPlayback: (deviceId: string, play = true) =>
    spotifyFetch<void>('/me/player', {
      method: 'PUT',
      body: { device_ids: [deviceId], play },
    }),
  play: (body?: { uris?: string[]; context_uri?: string; offset?: { position: number } }) =>
    spotifyFetch<void>('/me/player/play', { method: 'PUT', body: body ?? {} }),
  pause: () => spotifyFetch<void>('/me/player/pause', { method: 'PUT' }),
  next: () => spotifyFetch<void>('/me/player/next', { method: 'POST' }),
  previous: () => spotifyFetch<void>('/me/player/previous', { method: 'POST' }),
  seek: (positionMs: number) =>
    spotifyFetch<void>(`/me/player/seek${query({ position_ms: positionMs })}`, {
      method: 'PUT',
    }),
  setVolume: (volumePercent: number) =>
    spotifyFetch<void>(`/me/player/volume${query({ volume_percent: volumePercent })}`, {
      method: 'PUT',
    }),
  setRepeat: (state: RepeatState) =>
    spotifyFetch<void>(`/me/player/repeat${query({ state })}`, { method: 'PUT' }),
  setShuffle: (state: boolean) =>
    spotifyFetch<void>(`/me/player/shuffle${query({ state })}`, { method: 'PUT' }),

  /* --- Library ---------------------------------------------------------- */
  getSavedTracks: (limit = 20, offset = 0) =>
    spotifyFetch<SpotifyPaginated<SavedTrack>>(
      `/me/tracks${query({ limit, offset })}`,
    ),
  saveTracks: (ids: string[]) =>
    spotifyFetch<void>('/me/tracks', { method: 'PUT', body: { ids } }),
  removeSavedTracks: (ids: string[]) =>
    spotifyFetch<void>('/me/tracks', { method: 'DELETE', body: { ids } }),
  checkSavedTracks: (ids: string[]) =>
    spotifyFetch<boolean[]>(`/me/tracks/contains${query({ ids: ids.join(',') })}`),
  getSavedAlbums: (limit = 20, offset = 0) =>
    spotifyFetch<SpotifyPaginated<{ added_at: string; album: SimplifiedAlbum }>>(
      `/me/albums${query({ limit, offset })}`,
    ),
  getUserPlaylists: (limit = 20, offset = 0) =>
    spotifyFetch<SpotifyPaginated<SimplifiedPlaylist>>(
      `/me/playlists${query({ limit, offset })}`,
    ),

  /* --- Catalog ---------------------------------------------------------- */
  getPlaylist: (id: string) =>
    spotifyFetch<SimplifiedPlaylist>(`/playlists/${id}`),
  getPlaylistTracks: (id: string, limit = 50, offset = 0) =>
    spotifyFetch<SpotifyPaginated<{ added_at: string; track: SimplifiedTrack }>>(
      `/playlists/${id}/tracks${query({ limit, offset })}`,
    ),
  getAlbum: (id: string) =>
    spotifyFetch<SimplifiedAlbum & { tracks: SpotifyPaginated<SimplifiedTrack> }>(
      `/albums/${id}`,
    ),
  getAlbumTracks: (id: string, limit = 50, offset = 0) =>
    spotifyFetch<SpotifyPaginated<SimplifiedTrack>>(
      `/albums/${id}/tracks${query({ limit, offset })}`,
    ),
  getArtist: (id: string) => spotifyFetch<SimplifiedArtist>(`/artists/${id}`),
  getArtistTopTracks: (id: string, market = 'from_token') =>
    spotifyFetch<{ tracks: SimplifiedTrack[] }>(
      `/artists/${id}/top-tracks${query({ market })}`,
    ),
} as const;

export type SpotifyApi = typeof spotifyApi;