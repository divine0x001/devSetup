/**
 * Prism — Offline demo mode (QA / screenshots).
 *
 * Renders the whole UI navigable without a Spotify client id. Activated by
 * `VITE_PRISM_DEMO=1` (build-time) or `localStorage['prism.demo'] === '1'`
 * (runtime). When active, `spotifyFetch` short-circuits to `demoFetch` and the
 * auth layer hands out a fake session, so no view or hook needs to change.
 *
 * Everything here is deterministic: stable ids (`demo-*-N`), generated SVG
 * covers, and a "living" playback state whose progress advances with the clock.
 */

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
  SpotifyTokens,
  UserProfile,
} from './types';

/* -------------------------------------------------------------------------- */
/* Mode switch                                                                 */
/* -------------------------------------------------------------------------- */

export const DEMO_ENV_FLAG = 'VITE_PRISM_DEMO';
export const DEMO_STORAGE_KEY = 'prism.demo';

function envFlag(): string | undefined {
  try {
    const env = import.meta.env as unknown as Record<string, string | undefined>;
    return env[DEMO_ENV_FLAG];
  } catch {
    return undefined;
  }
}

/** True when demo mode is forced by env or opted-in via localStorage. */
export function isDemoMode(): boolean {
  if (envFlag() === '1') return true;
  try {
    if (typeof localStorage !== 'undefined' && localStorage.getItem(DEMO_STORAGE_KEY) === '1') {
      return true;
    }
  } catch {
    // localStorage unavailable (privacy mode): fall through.
  }
  return false;
}

export function enableDemoMode(): void {
  try {
    localStorage.setItem(DEMO_STORAGE_KEY, '1');
  } catch {
    // ignore
  }
}

export function disableDemoMode(): void {
  try {
    localStorage.removeItem(DEMO_STORAGE_KEY);
  } catch {
    // ignore
  }
}

/* -------------------------------------------------------------------------- */
/* Deterministic helpers                                                       */
/* -------------------------------------------------------------------------- */

function hash(input: string): number {
  let h = 2166136261;
  for (let i = 0; i < input.length; i += 1) {
    h ^= input.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

/** Aesthetic gradient palettes — [accent, deep] per tile. */
const PALETTES: ReadonlyArray<readonly [string, string]> = [
  ['#1ed760', '#06210f'],
  ['#8b7bff', '#150a30'],
  ['#ff5c8a', '#2a0616'],
  ['#f2c94c', '#2a1f05'],
  ['#4cc9f0', '#031f2b'],
  ['#ff8a4c', '#2a1205'],
  ['#5cffb0', '#032018'],
  ['#c084fc', '#1b0630'],
  ['#4c6fff', '#060f30'],
  ['#ff4c6a', '#2a050f'],
];

/** SVG data-URI cover generated from a seed (stable per id). */
function svgCover(seed: string, size = 320): string {
  const h = hash(seed);
  const [c1, c2] = PALETTES[h % PALETTES.length];
  const angle = h % 360;
  const cx = 18 + (h % 64);
  const cy = 18 + ((h >> 3) % 64);
  const r = 18 + ((h >> 6) % 42);
  const opacity = 0.18 + ((h >> 9) % 30) / 100;
  const svg =
    `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 100 100">` +
    `<defs><linearGradient id="g" gradientTransform="rotate(${angle} 0.5 0.5)">` +
    `<stop offset="0" stop-color="${c1}"/><stop offset="1" stop-color="${c2}"/>` +
    `</linearGradient></defs>` +
    `<rect width="100" height="100" fill="url(#g)"/>` +
    `<circle cx="${cx}" cy="${cy}" r="${r}" fill="#ffffff" opacity="${opacity}"/>` +
    `<circle cx="${100 - cx}" cy="${100 - cy}" r="${(r * 0.5).toFixed(1)}" fill="${c1}" opacity="0.32"/>` +
    `</svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

function cover(seed: string, size = 320) {
  return { url: svgCover(seed, size), height: size, width: size };
}

const DEMO_API = 'https://api.spotify.com/v1';

function paginate<T>(
  all: T[],
  limit: number,
  offset: number,
  href: string,
): SpotifyPaginated<T> {
  const items = all.slice(offset, offset + limit);
  const nextOffset = offset + limit;
  return {
    href,
    items,
    limit,
    next: nextOffset < all.length ? `${href}?offset=${nextOffset}&limit=${limit}` : null,
    offset,
    previous: offset > 0 ? `${href}?offset=${Math.max(0, offset - limit)}&limit=${limit}` : null,
    total: all.length,
  };
}

function addedAt(index: number): string {
  const day = String((index % 27) + 1).padStart(2, '0');
  const month = String((index % 11) + 1).padStart(2, '0');
  const year = 2023 + (index % 3);
  return `${year}-${month}-${day}T09:00:00Z`;
}

/* -------------------------------------------------------------------------- */
/* Dataset                                                                     */
/* -------------------------------------------------------------------------- */

interface ArtistDef {
  name: string;
  genres: string[];
}

const ARTIST_DEFS: ArtistDef[] = [
  { name: 'Nova Cascade', genres: ['dream pop', 'synthwave'] },
  { name: 'The Paper Foxes', genres: ['indie rock', 'garage'] },
  { name: 'Iris Melt', genres: ['shoegaze', 'ambient'] },
  { name: 'KODE 9TH', genres: ['electronic', 'house'] },
  { name: 'Marlowe Grey', genres: ['singer-songwriter', 'folk'] },
  { name: 'Static Bloom', genres: ['alt pop', 'electropop'] },
  { name: 'Ran Kaze', genres: ['techno', 'minimal'] },
  { name: 'Velvet Arcade', genres: ['funk', 'nu-disco'] },
  { name: 'Orla & the Tide', genres: ['indie folk', 'chamber pop'] },
  { name: 'Midnight Transit', genres: ['trip hop', 'downtempo'] },
];

interface AlbumDef {
  title: string;
  artist: number;
  year: string;
  type: 'album' | 'single' | 'compilation';
  tracks: string[];
}

const ALBUM_DEFS: AlbumDef[] = [
  { title: 'Neon Meridian', artist: 0, year: '2024', type: 'album', tracks: ['Neon Meridian', 'Ice Circuit', 'Glass Coastline', 'Afterglow Drive'] },
  { title: 'Paper Trails', artist: 1, year: '2023', type: 'album', tracks: ['Paper Trails', 'Rooftop Riot', 'Cardboard Crowns', 'Backseat Static'] },
  { title: 'Soft Static', artist: 2, year: '2022', type: 'album', tracks: ['Soft Static', 'Cathode Ray', 'Slow Collapse', 'Lavender Noise'] },
  { title: 'Grid Bloom', artist: 3, year: '2024', type: 'album', tracks: ['Grid Bloom', 'Low Orbit', 'Frequency Drift', 'Night Protocol'] },
  { title: 'Letters to Nowhere', artist: 4, year: '2021', type: 'album', tracks: ['Letters to Nowhere', 'Winter Postcard', 'The Long Way Home', 'Paper Moon'] },
  { title: 'Bloom//Decay', artist: 5, year: '2023', type: 'album', tracks: ['Bloom//Decay', 'Sugar Riot', 'Plastic Halo', 'Supercut'] },
  { title: 'Minimal Horizon', artist: 6, year: '2022', type: 'album', tracks: ['Minimal Horizon', 'Concrete Sunrise', 'Parallel Lines', 'Deep Field'] },
  { title: 'Velvet Hours', artist: 7, year: '2020', type: 'album', tracks: ['Velvet Hours', 'Disco Fever Dream', 'Moonlit Groove', 'Golden Hour'] },
  { title: 'Tidal Rooms', artist: 8, year: '2023', type: 'album', tracks: ['Tidal Rooms', 'Harbour Lights', 'Salt & Seaglass', 'The Undertow'] },
  { title: 'Last Departure', artist: 9, year: '2024', type: 'album', tracks: ['Last Departure', 'Transit Blues', 'Platform 9', 'Homeward'] },
];

/* --- Build artists -------------------------------------------------------- */

export const demoArtists: SimplifiedArtist[] = ARTIST_DEFS.map((def, i) => {
  const id = `demo-artist-${i + 1}`;
  return {
    id,
    name: def.name,
    type: 'artist',
    uri: `spotify:artist:${id}`,
    href: `${DEMO_API}/artists/${id}`,
    images: [cover(`artist-${id}`, 320)],
    genres: def.genres,
    external_urls: { spotify: `https://open.spotify.com/artist/${id}` },
  };
});

/* --- Build albums --------------------------------------------------------- */

export const demoAlbums: SimplifiedAlbum[] = ALBUM_DEFS.map((def, i) => {
  const id = `demo-album-${i + 1}`;
  const artist = demoArtists[def.artist];
  const releaseMonth = String(((hash(id) % 12) + 1)).padStart(2, '0');
  const releaseDay = String(((hash(id) % 27) + 1)).padStart(2, '0');
  return {
    id,
    name: def.title,
    album_type: def.type,
    artists: [artist],
    images: [cover(`album-${id}`, 320)],
    release_date: `${def.year}-${releaseMonth}-${releaseDay}`,
    release_date_precision: 'day',
    total_tracks: def.tracks.length,
    uri: `spotify:album:${id}`,
    type: 'album',
    href: `${DEMO_API}/albums/${id}`,
    external_urls: { spotify: `https://open.spotify.com/album/${id}` },
  };
});

/* --- Build tracks --------------------------------------------------------- */

export const demoTracks: SimplifiedTrack[] = [];

ALBUM_DEFS.forEach((def, albumIndex) => {
  const album = demoAlbums[albumIndex];
  def.tracks.forEach((title, trackIndex) => {
    const n = demoTracks.length + 1;
    const id = `demo-track-${n}`;
    const h = hash(id);
    const explicit = h % 5 === 0;
    demoTracks.push({
      id,
      name: title,
      artists: album.artists,
      album,
      duration_ms: 150_000 + (h % 180_000),
      explicit,
      track_number: trackIndex + 1,
      disc_number: 1,
      uri: `spotify:track:${id}`,
      type: 'track',
      href: `${DEMO_API}/tracks/${id}`,
      is_playable: true,
      preview_url: null,
      external_urls: { spotify: `https://open.spotify.com/track/${id}` },
      external_ids: { isrc: `DEMO${String(n).padStart(5, '0')}` },
    });
  });
});

/* --- Build playlists ------------------------------------------------------ */

interface PlaylistDef {
  name: string;
  description: string;
  trackIndexes: number[];
}

const PLAYLIST_DEFS: PlaylistDef[] = [
  { name: 'Focus Drift', description: 'Deep work, deep focus.', trackIndexes: [0, 4, 8, 12, 16, 20] },
  { name: 'Late Night Coding', description: 'Synthwave for the small hours.', trackIndexes: [3, 7, 11, 15, 27, 31] },
  { name: 'Morning Coffee', description: 'Warm indie to start the day.', trackIndexes: [24, 28, 32, 36, 5, 9] },
  { name: 'Neon Commute', description: 'Electro bangers for the train.', trackIndexes: [12, 13, 14, 15, 0, 1] },
  { name: 'Rainy Day', description: 'Slow, soft and reflective.', trackIndexes: [16, 17, 18, 19, 20, 21] },
  { name: 'All-Out Energy', description: 'Turn it up.', trackIndexes: [8, 9, 10, 11, 32, 33] },
];

export const demoPlaylists: SimplifiedPlaylist[] = PLAYLIST_DEFS.map((def, i) => {
  const id = `demo-playlist-${i + 1}`;
  return {
    id,
    name: def.name,
    description: def.description,
    images: [cover(`playlist-${id}`, 320)],
    owner: { id: 'prism-demo', display_name: 'Prism Demo' },
    tracks: { href: `${DEMO_API}/playlists/${id}/tracks`, total: def.trackIndexes.length },
    public: true,
    collaborative: false,
    uri: `spotify:playlist:${id}`,
    type: 'playlist',
    href: `${DEMO_API}/playlists/${id}`,
    external_urls: { spotify: `https://open.spotify.com/playlist/${id}` },
  };
});

const PLAYLIST_TRACK_INDEXES: Record<string, number[]> = Object.fromEntries(
  demoPlaylists.map((p, i) => [p.id, PLAYLIST_DEFS[i].trackIndexes]),
);

function albumTrackIds(albumId: string): string[] {
  return demoTracks.filter((t) => t.album.id === albumId).map((t) => t.id);
}

function artistTrackIds(artistId: string): string[] {
  return demoTracks.filter((t) => t.artists.some((a) => a.id === artistId)).map((t) => t.id);
}

/* --- Mutable library state ------------------------------------------------ */

const savedTrackIds = new Set<string>(
  demoTracks.filter((t) => hash(t.id) % 3 === 0).map((t) => t.id),
);
const savedAlbumIds = new Set<string>(demoAlbums.filter((_, i) => i % 2 === 0).map((a) => a.id));
const followedArtistIds = new Set<string>(
  demoArtists.filter((_, i) => i % 3 === 0).map((a) => a.id),
);

/* -------------------------------------------------------------------------- */
/* Profile & tokens                                                            */
/* -------------------------------------------------------------------------- */

export function createDemoProfile(): UserProfile {
  return {
    id: 'prism-demo',
    display_name: 'Prism Demo',
    uri: 'spotify:user:prism-demo',
    type: 'user',
    images: [cover('user-prism-demo', 300)],
    href: `${DEMO_API}/users/prism-demo`,
    email: 'demo@prism.app',
    country: 'FR',
    product: 'premium',
    followers: { href: null, total: 42 },
    external_urls: { spotify: 'https://open.spotify.com/user/prism-demo' },
  };
}

const DEMO_TOKEN = 'prism-demo-access-token';

/** Fake, far-future session so nothing ever attempts a refresh. */
export function createDemoTokens(): SpotifyTokens {
  return {
    accessToken: DEMO_TOKEN,
    refreshToken: 'prism-demo-refresh-token',
    tokenType: 'Bearer',
    scope: '',
    expiresAt: Date.now() + 365 * 24 * 60 * 60 * 1000,
  };
}

/* -------------------------------------------------------------------------- */
/* Living playback state                                                       */
/* -------------------------------------------------------------------------- */

interface DemoPlayerState {
  trackIndex: number;
  isPlaying: boolean;
  baseProgressMs: number;
  capturedAt: number;
  volume: number;
  shuffle: boolean;
  repeat: RepeatState;
  deviceId: string;
}

const demoDevices: PlayerDevice[] = [
  {
    id: 'demo-device-web',
    is_active: true,
    is_private_session: false,
    is_restricted: false,
    name: 'Prism Web Player',
    type: 'Computer',
    volume_percent: 70,
    supports_volume: true,
  },
  {
    id: 'demo-device-phone',
    is_active: false,
    is_private_session: false,
    is_restricted: false,
    name: 'Pixel 8',
    type: 'Smartphone',
    volume_percent: 55,
    supports_volume: true,
  },
  {
    id: 'demo-device-speaker',
    is_active: false,
    is_private_session: false,
    is_restricted: false,
    name: 'Living Room',
    type: 'Speaker',
    volume_percent: 40,
    supports_volume: true,
  },
];

const playerState: DemoPlayerState = {
  trackIndex: 0,
  isPlaying: true,
  baseProgressMs: 0,
  capturedAt: Date.now(),
  volume: 70,
  shuffle: false,
  repeat: 'off',
  deviceId: 'demo-device-web',
};

function trackDuration(index: number): number {
  return demoTracks[index]?.duration_ms ?? 180_000;
}

function reflectPlaying(): void {
  playerState.capturedAt = Date.now();
}

/** Advances the clock and rolls over tracks; returns the live progress (ms). */
function syncProgress(): number {
  if (!playerState.isPlaying) {
    return Math.min(playerState.baseProgressMs, trackDuration(playerState.trackIndex));
  }
  const now = Date.now();
  let progress = playerState.baseProgressMs + (now - playerState.capturedAt);
  const duration = trackDuration(playerState.trackIndex);
  if (progress >= duration) {
    if (playerState.repeat === 'track') {
      progress %= duration;
      playerState.baseProgressMs = progress;
      playerState.capturedAt = now;
    } else {
      playerState.trackIndex = (playerState.trackIndex + 1) % demoTracks.length;
      playerState.baseProgressMs = 0;
      playerState.capturedAt = now;
      progress = 0;
    }
  }
  return progress;
}

interface DemoPlayBody {
  uris?: string[];
  context_uri?: string;
  offset?: { position: number };
}

function resolveTracksFromBody(body?: DemoPlayBody): number[] | null {
  if (!body) return null;
  if (body.uris && body.uris.length > 0) {
    const indexes = body.uris
      .map((uri) => demoTracks.findIndex((t) => t.uri === uri || t.id === uri))
      .filter((i) => i >= 0);
    return indexes.length > 0 ? indexes : null;
  }
  if (body.context_uri) {
    const uri = body.context_uri;
    const playlist = demoPlaylists.find((p) => p.uri === uri || p.id === uri);
    if (playlist) {
      const indexes = PLAYLIST_TRACK_INDEXES[playlist.id] ?? [];
      const start = body.offset?.position ?? 0;
      return indexes.slice(start).concat(indexes.slice(0, start));
    }
    const album = demoAlbums.find((a) => a.uri === uri || a.id === uri);
    if (album) {
      const indexes = demoTracks
        .map((t, i) => ({ t, i }))
        .filter(({ t }) => t.album.id === album.id)
        .map(({ i }) => i);
      const start = body.offset?.position ?? 0;
      return indexes.slice(start).concat(indexes.slice(0, start));
    }
  }
  return null;
}

const playQueue: number[] = demoTracks.map((_, i) => i);
let queuePosition = 0;

function setQueue(indexes: number[] | null, startIndex = 0): void {
  if (!indexes || indexes.length === 0) {
    playerState.trackIndex = playQueue[queuePosition] ?? playerState.trackIndex;
    return;
  }
  playQueue.length = 0;
  playQueue.push(...indexes);
  queuePosition = startIndex;
  playerState.trackIndex = playQueue[queuePosition];
}

function demoPlay(body?: DemoPlayBody): void {
  const indexes = resolveTracksFromBody(body);
  if (indexes) {
    setQueue(indexes, 0);
    playerState.baseProgressMs = 0;
  }
  playerState.isPlaying = true;
  reflectPlaying();
}

function demoPause(): void {
  const progress = syncProgress();
  playerState.baseProgressMs = progress;
  playerState.isPlaying = false;
  reflectPlaying();
}

function demoNext(): void {
  queuePosition = (queuePosition + 1) % playQueue.length;
  playerState.trackIndex = playQueue[queuePosition];
  playerState.baseProgressMs = 0;
  reflectPlaying();
}

function demoPrevious(): void {
  if (syncProgress() > 3_000) {
    playerState.baseProgressMs = 0;
    reflectPlaying();
    return;
  }
  queuePosition = (queuePosition - 1 + playQueue.length) % playQueue.length;
  playerState.trackIndex = playQueue[queuePosition];
  playerState.baseProgressMs = 0;
  reflectPlaying();
}

function demoSeek(positionMs: number): void {
  playerState.baseProgressMs = Math.max(0, Math.min(positionMs, trackDuration(playerState.trackIndex)));
  reflectPlaying();
}

function demoSetVolume(volume: number): void {
  playerState.volume = Math.max(0, Math.min(100, volume));
}

function demoTransferPlayback(deviceId: string, play: boolean): void {
  if (demoDevices.some((d) => d.id === deviceId)) {
    playerState.deviceId = deviceId;
  }
  playerState.isPlaying = play;
  reflectPlaying();
}

function demoGetPlaybackState(): PlaybackState {
  const progress = syncProgress();
  const track = demoTracks[playerState.trackIndex] ?? null;
  const baseDevice = demoDevices.find((d) => d.id === playerState.deviceId) ?? demoDevices[0];
  return {
    device: {
      ...baseDevice,
      is_active: true,
      volume_percent: playerState.volume,
    },
    repeat_state: playerState.repeat,
    shuffle_state: playerState.shuffle,
    is_playing: playerState.isPlaying,
    progress_ms: progress,
    item: track,
    currently_playing_type: 'track',
    timestamp: Date.now(),
  };
}

/* -------------------------------------------------------------------------- */
/* Request router (transparent injection point)                                */
/* -------------------------------------------------------------------------- */

export interface DemoRequestOptions {
  method?: string;
  body?: unknown;
}

function searchAll(q: string, types: string, limit: number): SearchResults {
  const needle = q.trim().toLowerCase();
  const match = (haystack: string): boolean => haystack.toLowerCase().includes(needle);
  const wanted = new Set(types.split(',').map((t) => t.trim()));
  const result: SearchResults = {};

  if (wanted.has('track')) {
    const items = demoTracks.filter((t) => match(t.name) || t.artists.some((a) => match(a.name)));
    result.tracks = paginate(items, limit, 0, `${DEMO_API}/search`);
  }
  if (wanted.has('album')) {
    const items = demoAlbums.filter((a) => match(a.name) || a.artists.some((x) => match(x.name)));
    result.albums = paginate(items, limit, 0, `${DEMO_API}/search`);
  }
  if (wanted.has('artist')) {
    const items = demoArtists.filter((a) => match(a.name));
    result.artists = paginate(items, limit, 0, `${DEMO_API}/search`);
  }
  if (wanted.has('playlist')) {
    const items = demoPlaylists.filter((p) => match(p.name));
    result.playlists = paginate(items, limit, 0, `${DEMO_API}/search`);
  }
  return result;
}

function numberParam(params: URLSearchParams, key: string, fallback: number): number {
  const raw = params.get(key);
  if (raw === null) return fallback;
  const parsed = Number(raw);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function idsParam(params: URLSearchParams): string[] {
  const raw = params.get('ids');
  return raw ? raw.split(',').filter(Boolean) : [];
}

/**
 * Resolves a Web API path against the in-memory demo dataset. Used by
 * `spotifyFetch` as a transparent short-circuit when `isDemoMode()` is true.
 */
export function demoFetch(path: string, options: DemoRequestOptions = {}): unknown {
  const method = (options.method ?? 'GET').toUpperCase();
  const [pathname, search = ''] = path.split('?');
  const params = new URLSearchParams(search);
  const body = (options.body ?? {}) as Record<string, unknown>;

  /* --- User ------------------------------------------------------------- */
  if (pathname === '/me') return createDemoProfile();

  /* --- Search ----------------------------------------------------------- */
  if (pathname === '/search') {
    return searchAll(
      params.get('q') ?? '',
      params.get('type') ?? 'track,album,artist,playlist',
      numberParam(params, 'limit', 20),
    );
  }

  /* --- Playback --------------------------------------------------------- */
  if (pathname === '/me/player') {
    if (method === 'PUT') {
      const deviceIds = (body.device_ids as string[] | undefined) ?? [];
      const play = body.play === undefined ? true : Boolean(body.play);
      if (deviceIds[0]) demoTransferPlayback(deviceIds[0], play);
      return undefined;
    }
    return demoGetPlaybackState();
  }
  if (pathname === '/me/player/devices') {
    return { devices: demoDevices.map((d) => ({ ...d, is_active: d.id === playerState.deviceId })) };
  }
  if (pathname === '/me/player/play') {
    demoPlay(body as DemoPlayBody);
    return undefined;
  }
  if (pathname === '/me/player/pause') {
    demoPause();
    return undefined;
  }
  if (pathname === '/me/player/next') {
    demoNext();
    return undefined;
  }
  if (pathname === '/me/player/previous') {
    demoPrevious();
    return undefined;
  }
  if (pathname === '/me/player/seek') {
    demoSeek(numberParam(params, 'position_ms', 0));
    return undefined;
  }
  if (pathname === '/me/player/volume') {
    demoSetVolume(numberParam(params, 'volume_percent', playerState.volume));
    return undefined;
  }
  if (pathname === '/me/player/repeat') {
    const state = params.get('state');
    if (state === 'off' || state === 'track' || state === 'context') {
      playerState.repeat = state;
    }
    return undefined;
  }
  if (pathname === '/me/player/shuffle') {
    playerState.shuffle = params.get('state') === 'true';
    return undefined;
  }

  /* --- Library ---------------------------------------------------------- */
  if (pathname === '/me/tracks') {
    if (method === 'PUT' || method === 'DELETE') {
      const ids = (body.ids as string[] | undefined) ?? [];
      for (const id of ids) {
        if (method === 'PUT') savedTrackIds.add(id);
        else savedTrackIds.delete(id);
      }
      return undefined;
    }
    const items: SavedTrack[] = demoTracks
      .filter((t) => savedTrackIds.has(t.id))
      .map((t, i) => ({ added_at: addedAt(i), track: t }));
    return paginate(
      items,
      numberParam(params, 'limit', 20),
      numberParam(params, 'offset', 0),
      `${DEMO_API}/me/tracks`,
    );
  }
  if (pathname === '/me/tracks/contains') {
    return idsParam(params).map((id) => savedTrackIds.has(id));
  }
  if (pathname === '/me/albums') {
    const items = demoAlbums
      .filter((a) => savedAlbumIds.has(a.id))
      .map((a, i) => ({ added_at: addedAt(i), album: a }));
    return paginate(
      items,
      numberParam(params, 'limit', 20),
      numberParam(params, 'offset', 0),
      `${DEMO_API}/me/albums`,
    );
  }
  if (pathname === '/me/playlists') {
    return paginate(
      demoPlaylists,
      numberParam(params, 'limit', 20),
      numberParam(params, 'offset', 0),
      `${DEMO_API}/me/playlists`,
    );
  }
  if (pathname === '/me/following') {
    const items = demoArtists.filter((a) => followedArtistIds.has(a.id));
    return {
      artists: paginate(
        items,
        numberParam(params, 'limit', 20),
        numberParam(params, 'offset', 0),
        `${DEMO_API}/me/following`,
      ),
    };
  }

  /* --- Playlists -------------------------------------------------------- */
  const playlistTracksMatch = /^\/playlists\/([^/]+)\/tracks$/.exec(pathname);
  if (playlistTracksMatch) {
    const id = playlistTracksMatch[1];
    const indexes = PLAYLIST_TRACK_INDEXES[id] ?? [];
    const items = indexes.map((trackIndex, i) => ({
      added_at: addedAt(i),
      track: demoTracks[trackIndex],
    }));
    return paginate(
      items,
      numberParam(params, 'limit', 50),
      numberParam(params, 'offset', 0),
      `${DEMO_API}${pathname}`,
    );
  }
  const playlistMatch = /^\/playlists\/([^/]+)$/.exec(pathname);
  if (playlistMatch) {
    const playlist = demoPlaylists.find((p) => p.id === playlistMatch[1]);
    if (!playlist) throw new Error(`Prism demo: playlist ${playlistMatch[1]} not found.`);
    return playlist;
  }

  /* --- Albums ----------------------------------------------------------- */
  const albumTracksMatch = /^\/albums\/([^/]+)\/tracks$/.exec(pathname);
  if (albumTracksMatch) {
    const ids = albumTrackIds(albumTracksMatch[1]);
    const items = ids
      .map((id) => demoTracks.find((t) => t.id === id))
      .filter((t): t is SimplifiedTrack => Boolean(t));
    return paginate(
      items,
      numberParam(params, 'limit', 50),
      numberParam(params, 'offset', 0),
      `${DEMO_API}${pathname}`,
    );
  }
  const albumMatch = /^\/albums\/([^/]+)$/.exec(pathname);
  if (albumMatch) {
    const album = demoAlbums.find((a) => a.id === albumMatch[1]);
    if (!album) throw new Error(`Prism demo: album ${albumMatch[1]} not found.`);
    const ids = albumTrackIds(album.id);
    const items = ids
      .map((id) => demoTracks.find((t) => t.id === id))
      .filter((t): t is SimplifiedTrack => Boolean(t));
    return {
      ...album,
      tracks: paginate(items, 50, 0, `${DEMO_API}/albums/${album.id}/tracks`),
    };
  }

  /* --- Artists ---------------------------------------------------------- */
  const topTracksMatch = /^\/artists\/([^/]+)\/top-tracks$/.exec(pathname);
  if (topTracksMatch) {
    const ids = artistTrackIds(topTracksMatch[1]);
    const tracks = ids
      .map((id) => demoTracks.find((t) => t.id === id))
      .filter((t): t is SimplifiedTrack => Boolean(t))
      .slice(0, 10);
    return { tracks };
  }
  const artistMatch = /^\/artists\/([^/]+)$/.exec(pathname);
  if (artistMatch) {
    const artist = demoArtists.find((a) => a.id === artistMatch[1]);
    if (!artist) throw new Error(`Prism demo: artist ${artistMatch[1]} not found.`);
    return artist;
  }

  throw new Error(`Prism demo: unhandled path ${method} ${pathname}.`);
}
