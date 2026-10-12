import { useEffect, useState } from 'react';

export interface DrawerPresence {
  /** Keep the drawer mounted (including during its exit animation). */
  mounted: boolean;
  /** True once the enter transition should play. */
  entered: boolean;
}

/**
 * Drives the slide-over lifecycle so the drawer can animate both in and out
 * while still unmounting completely when closed.
 */
export function useDrawerPresence(open: boolean, exitMs = 260): DrawerPresence {
  const [mounted, setMounted] = useState(open);
  const [entered, setEntered] = useState(false);

  useEffect(() => {
    if (open) {
      setMounted(true);
      const frame = requestAnimationFrame(() => setEntered(true));
      return () => cancelAnimationFrame(frame);
    }

    setEntered(false);
    const timer = window.setTimeout(() => setMounted(false), exitMs);
    return () => window.clearTimeout(timer);
  }, [open, exitMs]);

  return { mounted, entered };
}
