/**
 * Prism — Playback command hooks.
 *
 * Thin, memoized wrappers over `spotifyApi` so components never touch the
 * client directly. Volume is mirrored optimistically into the player store.
 */

import { useCallback } from 'react';

import { spotifyApi } from '@/lib/spotify/client';
import { usePlayerStore } from '@/store/playerStore';

import type { RepeatState } from '@/lib/spotify/types';

export interface PlayBody {
  uris?: string[];
  context_uri?: string;
  offset?: { position: number };
}

export interface PlaybackControls {
  play: (body?: PlayBody) => Promise<void>;
  pause: () => Promise<void>;
  next: () => Promise<void>;
  previous: () => Promise<void>;
  seek: (positionMs: number) => Promise<void>;
  setVolume: (volume: number) => Promise<void>;
  setRepeat: (state: RepeatState) => Promise<void>;
  setShuffle: (state: boolean) => Promise<void>;
}

export function usePlaybackControls(): PlaybackControls {
  const setVolume = usePlayerStore((state) => state.setVolume);

  const play = useCallback((body?: PlayBody) => spotifyApi.play(body), []);
  const pause = useCallback(() => spotifyApi.pause(), []);
  const next = useCallback(() => spotifyApi.next(), []);
  const previous = useCallback(() => spotifyApi.previous(), []);
  const seek = useCallback((positionMs: number) => spotifyApi.seek(positionMs), []);
  const handleSetVolume = useCallback(
    async (volume: number) => {
      const clamped = Math.min(100, Math.max(0, volume));
      setVolume(clamped); // optimistic local mirror
      await spotifyApi.setVolume(clamped);
    },
    [setVolume],
  );
  const setRepeat = useCallback(
    (state: RepeatState) => spotifyApi.setRepeat(state),
    [],
  );
  const setShuffle = useCallback(
    (state: boolean) => spotifyApi.setShuffle(state),
    [],
  );

  return {
    play,
    pause,
    next,
    previous,
    seek,
    setVolume: handleSetVolume,
    setRepeat,
    setShuffle,
  };
}