/**
 * Prism — Player store (FROZEN shape).
 *
 * Mirrors the Web Playback SDK state + the Connect device state. All playback
 * *commands* go through `src/lib/spotify/client.ts`; this store only holds the
 * reactive representation.
 */

import { create } from 'zustand';

import type { PlaybackState, PlayerDevice } from '@/lib/spotify/types';

export interface PlayerStoreState {
  playback: PlaybackState | null;
  device: PlayerDevice | null;
  /** id of the in-app SDK device once it is ready. */
  sdkDeviceId: string | null;
  isReady: boolean;
  isPlaying: boolean;
  /** Local, optimistic progress in ms (ticked by the UI). */
  progressMs: number;
  /** 0..100 */
  volume: number;
  setPlayback: (playback: PlaybackState | null) => void;
  setDevice: (device: PlayerDevice | null) => void;
  setSdkDeviceId: (deviceId: string | null) => void;
  setReady: (isReady: boolean) => void;
  setPlaying: (isPlaying: boolean) => void;
  setProgressMs: (progressMs: number) => void;
  setVolume: (volume: number) => void;
  reset: () => void;
}

const initialState = {
  playback: null,
  device: null,
  sdkDeviceId: null,
  isReady: false,
  isPlaying: false,
  progressMs: 0,
  volume: 70,
};

export const usePlayerStore = create<PlayerStoreState>((set) => ({
  ...initialState,
  setPlayback: (playback) =>
    set({
      playback,
      isPlaying: playback?.is_playing ?? false,
      progressMs: playback?.progress_ms ?? 0,
    }),
  setDevice: (device) => set({ device }),
  setSdkDeviceId: (sdkDeviceId) => set({ sdkDeviceId }),
  setReady: (isReady) => set({ isReady }),
  setPlaying: (isPlaying) => set({ isPlaying }),
  setProgressMs: (progressMs) => set({ progressMs }),
  setVolume: (volume) => set({ volume: Math.min(100, Math.max(0, volume)) }),
  reset: () => set({ ...initialState }),
}));
