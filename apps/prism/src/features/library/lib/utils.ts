/**
 * Prism — Library feature: tiny shared utilities.
 *
 * Kept dependency-free so both the library and player features can import them
 * without pulling in React.
 */

/** Joins truthy class names (a minimal `clsx`). */
export function cn(...classes: Array<string | false | null | undefined>): string {
  return classes.filter(Boolean).join(' ');
}

/**
 * Swallows a promise rejection. Playback commands are fire-and-forget from the
 * UI's perspective — a failed seek/volume call should never surface an
 * unhandled rejection.
 */
export function fireAndForget(promise: Promise<unknown>): void {
  promise.catch(() => undefined);
}
