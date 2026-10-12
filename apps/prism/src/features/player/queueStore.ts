/**
 * Prism — Player feature: local play queue.
 *
 * The Web API exposes no queue endpoint, so Prism keeps its own client-side
 * queue. It is intentionally session-scoped (not persisted) and mirrors what
 * the user has lined up from the library views.
 */

import { create } from 'zustand';

import type { SimplifiedTrack } from '@/lib/spotify/types';

export interface QueueStoreState {
  queue: SimplifiedTrack[];
  /** Index of the currently playing item within `queue`, or -1. */
  currentIndex: number;
  addToQueue: (track: SimplifiedTrack) => void;
  /** Inserts right after the current item (or at the top when idle). */
  addNext: (track: SimplifiedTrack) => void;
  removeFromQueue: (index: number) => void;
  clearQueue: () => void;
  /** Replaces the queue and marks `startIndex` as current. */
  setQueue: (tracks: SimplifiedTrack[], startIndex?: number) => void;
  setCurrentIndex: (index: number) => void;
}

export const useQueueStore = create<QueueStoreState>((set) => ({
  queue: [],
  currentIndex: -1,
  addToQueue: (track) => set((state) => ({ queue: [...state.queue, track] })),
  addNext: (track) =>
    set((state) => {
      const insertAt = state.currentIndex >= 0 ? state.currentIndex + 1 : 0;
      const queue = [...state.queue];
      queue.splice(insertAt, 0, track);
      return { queue };
    }),
  removeFromQueue: (index) =>
    set((state) => {
      const queue = state.queue.filter((_, i) => i !== index);
      let currentIndex = state.currentIndex;
      if (index < currentIndex) currentIndex -= 1;
      else if (index === currentIndex) currentIndex = Math.min(currentIndex, queue.length - 1);
      return { queue, currentIndex };
    }),
  clearQueue: () => set({ queue: [], currentIndex: -1 }),
  setQueue: (tracks, startIndex = 0) =>
    set({ queue: tracks, currentIndex: tracks.length > 0 ? startIndex : -1 }),
  setCurrentIndex: (currentIndex) => set({ currentIndex }),
}));
