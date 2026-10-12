/**
 * Prism — Shared Spotify data types (FROZEN subset).
 *
 * These are the minimal shapes shared across workers. They intentionally model
 * only the fields Prism actually uses. Worker 2 may add more types in sibling
 * files but must not change these.
 *
 * @see https://developer.spotify.com/documentation/web-api/reference
 */

/* -------------------------------------------------------------------------- */
/* Common                                                                      */
/* -------------------------------------------------------------------------- */

export interface SpotifyImage {
  url: string;
  height: number | null;
  width: number | null;
}

export interface ExternalUrls {
  spotify: string;
}

export interface SpotifyExternalIds {
  isrc?: string;
  ean?: string;
  upc?: string;
}

/** Standard paged response envelope. */
export interface SpotifyPaginated<T> {
  href: string;
  items: T[];
  limit: number;
  next: string | null;
  offset: number;
  previous: string | null;
  total: number;
}

/** A paging object with only metadata (e.g. playlist.tracks). */
export interface SpotifyPagingRef {
  href: string;
  total: number;
}

/* -------------------------------------------------------------------------- */
/* Core entities                                                               */
/* -------------------------------------------------------------------------- */

export interface SimplifiedArtist {
  id: string;
  name: string;
  type: 'artist';
  uri: string;
  href?: string;
  images?: SpotifyImage[];
  genres?: string[];
  external_urls?: ExternalUrls;
}

export interface SimplifiedAlbum {
  id: string;
  name: string;
  album_type: 'album' | 'single' | 'compilation';
  artists: SimplifiedArtist[];
  images: SpotifyImage[];
  release_date: string;
  release_date_precision: 'year' | 'month' | 'day';
  total_tracks: number;
  uri: string;
  type: 'album';
  href?: string;
  external_urls?: ExternalUrls;
}

export interface SimplifiedTrack {
  id: string;
  name: string;
  artists: SimplifiedArtist[];
  album: SimplifiedAlbum;
  duration_ms: number;
  explicit: boolean;
  track_number: number;
  disc_number: number;
  uri: string;
  type: 'track';
  href?: string;
  is_playable?: boolean;
  preview_url: string | null;
  external_urls?: ExternalUrls;
  external_ids?: SpotifyExternalIds;
}

export interface SimplifiedPlaylist {
  id: string;
  name: string;
  description: string | null;
  images: SpotifyImage[];
  owner: {
    id: string;
    display_name: string | null;
  };
  tracks: SpotifyPagingRef;
  public: boolean | null;
  collaborative: boolean;
  uri: string;
  type: 'playlist';
  href?: string;
  external_urls?: ExternalUrls;
}

/** A track saved in the user's library. */
export interface SavedTrack {
  added_at: string;
  track: SimplifiedTrack;
}

/* -------------------------------------------------------------------------- */
/* Player                                                                      */
/* -------------------------------------------------------------------------- */

export interface PlayerDevice {
  id: string | null;
  is_active: boolean;
  is_private_session: boolean;
  is_restricted: boolean;
  name: string;
  type: string;
  volume_percent: number | null;
  supports_volume: boolean;
}

export type RepeatState = 'off' | 'track' | 'context';

export interface PlaybackState {
  device: PlayerDevice;
  repeat_state: RepeatState;
  shuffle_state: boolean;
  is_playing: boolean;
  progress_ms: number | null;
  /** Currently playing track. Episodes are out of scope for now. */
  item: SimplifiedTrack | null;
  currently_playing_type: 'track' | 'episode' | 'ad' | 'unknown';
  timestamp: number;
}

/* -------------------------------------------------------------------------- */
/* User & auth                                                                 */
/* -------------------------------------------------------------------------- */

export interface UserProfile {
  id: string;
  display_name: string | null;
  uri: string;
  type: 'user';
  images: SpotifyImage[];
  href?: string;
  email?: string;
  country?: string;
  product?: 'free' | 'premium' | 'open';
  followers?: { href: string | null; total: number };
  external_urls?: ExternalUrls;
}

/** Raw token endpoint response. */
export interface SpotifyTokenResponse {
  access_token: string;
  token_type: 'Bearer';
  expires_in: number;
  refresh_token?: string;
  scope: string;
}

/** Token as persisted by Prism (adds an absolute expiry timestamp). */
export interface SpotifyTokens {
  accessToken: string;
  refreshToken: string | null;
  tokenType: 'Bearer';
  scope: string;
  /** Epoch ms at which the access token expires. */
  expiresAt: number;
}

/* -------------------------------------------------------------------------- */
/* Search                                                                      */
/* -------------------------------------------------------------------------- */

export interface SearchResults {
  tracks?: SpotifyPaginated<SimplifiedTrack>;
  albums?: SpotifyPaginated<SimplifiedAlbum>;
  artists?: SpotifyPaginated<SimplifiedArtist>;
  playlists?: SpotifyPaginated<SimplifiedPlaylist>;
}
