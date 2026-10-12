/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Spotify application Client ID (see .env.example). */
  readonly VITE_SPOTIFY_CLIENT_ID: string;
  /** Redirect URI registered in the Spotify dashboard. */
  readonly VITE_SPOTIFY_REDIRECT_URI: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
