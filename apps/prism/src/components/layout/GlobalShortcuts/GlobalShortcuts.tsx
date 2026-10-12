import { useEffect, useRef } from 'react';

import { usePlayer, usePlaybackControls, useUi } from '@/hooks';

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(
    target.closest(
      'input, textarea, select, [contenteditable="true"], [contenteditable=""], [contenteditable="plaintext-only"], [role="textbox"]',
    ),
  );
}

function isInteractiveTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest('button, a, [role="button"], [role="menuitem"], summary'));
}

/**
 * Headless component wiring global keyboard shortcuts:
 * - ⌘K / Ctrl+K toggles the command palette.
 * - Space toggles playback (ignored while typing or when a control is focused).
 *
 * State is read through a ref so the window listener is attached exactly once
 * per mount and never re-subscribed on re-render (idempotent).
 */
export function GlobalShortcuts() {
  const { commandPaletteOpen, setCommandPaletteOpen } = useUi();
  const { isPlaying } = usePlayer();
  const controls = usePlaybackControls();

  const stateRef = useRef({ commandPaletteOpen, isPlaying, controls });
  useEffect(() => {
    stateRef.current = { commandPaletteOpen, isPlaying, controls };
  }, [commandPaletteOpen, isPlaying, controls]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.repeat) return;

      const isPlainSpace =
        event.code === 'Space' &&
        !event.metaKey &&
        !event.ctrlKey &&
        !event.altKey &&
        !event.shiftKey;

      if (isPlainSpace && (isEditableTarget(event.target) || isInteractiveTarget(event.target))) {
        return;
      }

      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setCommandPaletteOpen(!stateRef.current.commandPaletteOpen);
        return;
      }

      if (isPlainSpace) {
        event.preventDefault();
        const { isPlaying: playing, controls: playbackControls } = stateRef.current;
        const promise = playing ? playbackControls.pause() : playbackControls.play();
        promise.catch(() => undefined);
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [setCommandPaletteOpen]);

  return null;
}
