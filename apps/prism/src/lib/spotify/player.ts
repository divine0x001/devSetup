/**
 * Prism — Web Playback SDK wrapper.
 *
 * The Spotify Web Playback SDK ships as a global script (no npm package), so we
 * load it once via `loadSpotifySdk()` and expose a typed `Spotify.Player`
 * factory. `window.Spotify` is declared here so the rest of the app never needs
 * `any` casts.
 *
 * @see https://developer.spotify.com/documentation/web-playback-sdk
 */

import { spotifyConfig } from './config';

/* -------------------------------------------------------------------------- */
/* SDK types (subset of the Web Playback SDK surface)                          */
/* -------------------------------------------------------------------------- */

export interface SpotifySdkImage {
  url: string;
  height?: number | null;
  width?: number | null;
}

export interface SpotifySdkArtist {
  name: string;
  uri: string;
  id?: string;
}

export interface SpotifySdkAlbum {
  name: string;
  uri: string;
  images?: SpotifySdkImage[];
}

export interface SpotifySdkTrack {
  uri: string;
  id: string | null;
  type: 'track' | 'episode' | 'ad';
  uid: string;
  name: string;
  duration_ms: number;
  artists: SpotifySdkArtist[];
  album: SpotifySdkAlbum;
  is_playable: boolean;
  is_episode: boolean;
  linked_from?: { uri: string; id: string };
}

/** Playback state as reported by the SDK (differs from the Web API shape). */
export interface SpotifySdkPlaybackState {
  context: { uri: string | null; metadata: Record<string, unknown> } | null;
  disallows: {
    pausing?: boolean;
    resuming?: boolean;
    seeking?: boolean;
    skipping_prev?: boolean;
    skipping_next?: boolean;
    toggling_repeat_context?: boolean;
    toggling_repeat_track?: boolean;
    toggling_shuffle?: boolean;
    transferring_playback?: boolean;
  };
  duration: number;
  paused: boolean;
  position: number;
  repeat_mode: 0 | 1 | 2;
  shuffle: boolean;
  timestamp: number;
  track_window: {
    current_track: SpotifySdkTrack;
    previous_tracks: SpotifySdkTrack[];
    next_tracks: SpotifySdkTrack[];
  };
}

export interface SpotifyPlayerError {
  message: string;
}

/** Event name → payload map for `addListener`/`removeListener`. */
export interface SpotifyPlayerEventMap {
  ready: { device_id: string };
  not_ready: { device_id: string };
  player_state_changed: SpotifySdkPlaybackState | null;
  initialization_error: SpotifyPlayerError;
  authentication_error: SpotifyPlayerError;
  account_error: SpotifyPlayerError;
  playback_error: SpotifyPlayerError;
}

export interface SpotifyPlayerInit {
  name: string;
  getOAuthToken: (callback: (token: string) => void) => void;
  volume?: number;
}

/** Typed surface of the SDK `Spotify.Player` instance. */
export interface SpotifyPlayerInstance {
  connect(): Promise<boolean>;
  disconnect(): void;
  addListener<K extends keyof SpotifyPlayerEventMap>(
    event: K,
    callback: (payload: SpotifyPlayerEventMap[K]) => void,
  ): boolean;
  removeListener<K extends keyof SpotifyPlayerEventMap>(
    event: K,
    callback?: (payload: SpotifyPlayerEventMap[K]) => void,
  ): boolean;
  getCurrentState(): Promise<SpotifySdkPlaybackState | null>;
  setName(name: string): Promise<void>;
  setVolume(volume: number): Promise<void>;
  getVolume(): Promise<number>;
  pause(): Promise<void>;
  resume(): Promise<void>;
  togglePlay(): Promise<void>;
  seek(positionMs: number): Promise<void>;
  previousTrack(): Promise<void>;
  nextTrack(): Promise<void>;
  setShuffle(shuffle: boolean): Promise<void>;
  setRepeat(mode: 0 | 1 | 2): Promise<void>;
  getShuffle(): Promise<boolean>;
  getRepeat(): Promise<0 | 1 | 2>;
  activateElement(): Promise<void>;
}

declare global {
  interface Window {
    Spotify?: {
      Player: new (init: SpotifyPlayerInit) => SpotifyPlayerInstance;
    };
  }
}

/* -------------------------------------------------------------------------- */
/* Script loading                                                              */
/* -------------------------------------------------------------------------- */

let sdkPromise: Promise<void> | null = null;

/**
 * Ensures the Web Playback SDK script is loaded and `window.Spotify.Player`
 * is available. Idempotent: concurrent callers share the same promise.
 */
export function loadSpotifySdk(): Promise<void> {
  if (sdkPromise) return sdkPromise;

  sdkPromise = new Promise<void>((resolve, reject) => {
    if (typeof window === 'undefined') {
      reject(new Error('Prism: Web Playback SDK requires a browser.'));
      return;
    }
    if (window.Spotify?.Player) {
      resolve();
      return;
    }

    const script = document.createElement('script');
    script.src = spotifyConfig.sdkUrl;
    script.async = true;
    script.onload = () => {
      // The SDK sets window.Spotify asynchronously after the script executes,
      // so poll briefly instead of assuming onload === ready.
      const startedAt = Date.now();
      const poll = window.setInterval(() => {
        if (window.Spotify?.Player) {
          window.clearInterval(poll);
          resolve();
        } else if (Date.now() - startedAt > 10_000) {
          window.clearInterval(poll);
          reject(new Error('Prism: Web Playback SDK timed out.'));
        }
      }, 50);
    };
    script.onerror = () => {
      reject(new Error('Prism: failed to load the Web Playback SDK script.'));
    };
    document.head.appendChild(script);
  });

  return sdkPromise;
}

/* -------------------------------------------------------------------------- */
/* Player factory                                                              */
/* -------------------------------------------------------------------------- */

/**
 * Creates a Web Playback SDK player. Requires the SDK to be loaded first
 * (call `loadSpotifySdk()`).
 */
export function createSpotifyPlayer(init: SpotifyPlayerInit): SpotifyPlayerInstance {
  if (!window.Spotify?.Player) {
    throw new Error('Prism: Web Playback SDK not loaded (call loadSpotifySdk first).');
  }
  return new window.Spotify.Player(init);
}