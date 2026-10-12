/**
 * Prism — Spotify client configuration (FROZEN).
 *
 * Everything is resolved from Vite env vars at build time. No backend, no
 * client secret: authentication uses Authorization Code + PKCE.
 */

export interface SpotifyConfig {
  /** Spotify app Client ID (public). */
  clientId: string;
  /** Redirect URI registered in the Spotify dashboard. */
  redirectUri: string;
  /** OAuth scopes requested at login. */
  scopes: readonly string[];
  authorizeEndpoint: string;
  tokenEndpoint: string;
  /** Web API base URL. */
  apiBaseUrl: string;
  /** Web Playback SDK script URL. */
  sdkUrl: string;
}

/** OAuth scopes Prism needs. Extend here, not inline. */
export const SPOTIFY_SCOPES = [
  'user-read-private',
  'user-read-email',
  'streaming',
  'user-read-playback-state',
  'user-modify-playback-state',
  'user-read-currently-playing',
  'user-library-read',
  'user-library-modify',
  'playlist-read-private',
  'playlist-read-collaborative',
  'playlist-modify-private',
  'playlist-modify-public',
  'user-follow-read',
  'user-follow-modify',
  'user-top-read',
] as const;

const FALLBACK_REDIRECT_URI = 'http://127.0.0.1:5173/callback';

export const spotifyConfig: SpotifyConfig = {
  clientId: import.meta.env.VITE_SPOTIFY_CLIENT_ID ?? '',
  redirectUri: import.meta.env.VITE_SPOTIFY_REDIRECT_URI || FALLBACK_REDIRECT_URI,
  scopes: SPOTIFY_SCOPES,
  authorizeEndpoint: 'https://accounts.spotify.com/authorize',
  tokenEndpoint: 'https://accounts.spotify.com/api/token',
  apiBaseUrl: 'https://api.spotify.com/v1',
  sdkUrl: 'https://sdk.scdn.co/spotify-player.js',
};

/** True when `VITE_SPOTIFY_CLIENT_ID` has been provided. */
export function isSpotifyConfigured(): boolean {
  return spotifyConfig.clientId.trim().length > 0;
}

/** Throws a user-actionable error when the app is not configured yet. */
export function assertSpotifyConfigured(): void {
  if (!isSpotifyConfigured()) {
    throw new Error(
      'Prism is not configured: set VITE_SPOTIFY_CLIENT_ID in .env.local (see .env.example).',
    );
  }
}
