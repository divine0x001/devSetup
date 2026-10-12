/**
 * Prism — Player feature: smooth playback position.
 *
 * The player store's `progressMs` is only refreshed by polling (every few
 * seconds), so a raw read makes the progress bar jump. This hook interpolates
 * locally between store updates while playback is active.
 */

import { useEffect, useRef, useState } from 'react';

import { usePlayerStore } from '@/store/playerStore';

const TICK_MS = 250;

/** Returns an interpolated playback position in milliseconds. */
export function useSmoothProgress(): number {
  const progressMs = usePlayerStore((state) => state.progressMs);
  const isPlaying = usePlayerStore((state) => state.isPlaying);
  const durationMs = usePlayerStore((state) => state.playback?.item?.duration_ms ?? 0);

  const [display, setDisplay] = useState(progressMs);
  const baseRef = useRef({ progressMs, at: Date.now() });

  // Re-anchor whenever the store reports a fresh position.
  useEffect(() => {
    baseRef.current = { progressMs, at: Date.now() };
    setDisplay(progressMs);
  }, [progressMs]);

  useEffect(() => {
    if (!isPlaying) return;
    const id = window.setInterval(() => {
      const elapsed = Date.now() - baseRef.current.at;
      const next = baseRef.current.progressMs + elapsed;
      setDisplay(durationMs > 0 ? Math.min(next, durationMs) : next);
    }, TICK_MS);
    return () => window.clearInterval(id);
  }, [isPlaying, durationMs]);

  return display;
}
