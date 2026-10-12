import { useEffect, useLayoutEffect, type ReactNode } from 'react';

import { useMediaQuery } from '@/hooks';
import { useThemeStore } from '@/store/themeStore';
import { useCustomizerStore } from '@/theme/customizerStore';
import {
  DEFAULT_EFFECTS,
  themeToCssVariables,
  type ColorMode,
  type Theme,
  type ThemeProjection,
} from '@/theme/types';

// Side-effect import: the ambient/effect stylesheet consumed by this provider.
import '@/theme/effects.css';

export interface ThemeProviderProps {
  children: ReactNode;
}

type ResolvedMode = 'dark' | 'light';

const useIsomorphicLayoutEffect =
  typeof window !== 'undefined' ? useLayoutEffect : useEffect;

function resolveMode(mode: ColorMode, prefersDark: boolean): ResolvedMode {
  if (mode === 'system') return prefersDark ? 'dark' : 'light';
  return mode;
}

/**
 * Project the active theme (+ customizer effects) onto the document root.
 *
 * Writes every `--prism-*` custom property, mirrors the behavioral knobs as
 * `data-*` attributes, keeps `color-scheme` in sync for native controls and
 * refreshes the browser theme-color. Kept as a pure DOM routine so it can also
 * run synchronously before React mounts (avoids a flash of the default theme).
 */
function applyThemeToDocument(
  theme: Theme,
  projection: ThemeProjection,
  resolved: ResolvedMode,
): void {
  const root = document.documentElement;

  const vars = themeToCssVariables(theme, projection);
  for (const [name, value] of Object.entries(vars)) {
    root.style.setProperty(name, value);
  }

  root.dataset.theme = theme.id;
  root.dataset.colorMode = theme.colorMode;
  root.dataset.colorResolved = resolved;
  root.dataset.layout = theme.layout;
  root.dataset.density = theme.density;
  root.dataset.radius = theme.radiusScale;
  root.dataset.bg = projection.effects?.background ?? DEFAULT_EFFECTS.background;
  root.dataset.glow = (projection.effects?.glow ?? false) ? 'true' : 'false';
  root.dataset.backdrop = (projection.effects?.backdropBlur ?? false)
    ? 'true'
    : 'false';

  root.style.colorScheme = resolved;

  const meta = document.querySelector('meta[name="theme-color"]');
  if (meta) meta.setAttribute('content', theme.tokens.colors.bg);
}

// Apply the persisted theme as early as possible — before React's first paint.
// zustand's persist middleware hydrates synchronously from localStorage, so the
// values are already available here.
if (typeof window !== 'undefined' && typeof document !== 'undefined') {
  const theme = useThemeStore.getState().theme;
  const { effects, typeScale, motionSpeed, transitions } =
    useCustomizerStore.getState();
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  applyThemeToDocument(
    theme,
    { effects, typeScale, motionSpeed, transitions },
    resolveMode(theme.colorMode, prefersDark),
  );
}

/**
 * Applies the active `Theme` to the document at runtime:
 *  - writes every `--prism-*` custom property on <html> (overriding tokens.css),
 *  - sets `data-theme`, `data-color-mode`, `data-color-resolved`, `data-layout`,
 *    `data-density`, `data-radius` (plus the customizer's `data-bg`/`data-glow`),
 *  - resolves `colorMode: 'system'` through `prefers-color-scheme` and reacts to
 *    OS changes,
 *  - keeps `color-scheme` and the theme-color meta in sync.
 *
 * Uses a layout effect so updates are committed before the browser paints.
 */
export function ThemeProvider({ children }: ThemeProviderProps) {
  const theme = useThemeStore((state) => state.theme);
  const effects = useCustomizerStore((state) => state.effects);
  const typeScale = useCustomizerStore((state) => state.typeScale);
  const motionSpeed = useCustomizerStore((state) => state.motionSpeed);
  const transitions = useCustomizerStore((state) => state.transitions);

  const prefersDark = useMediaQuery('(prefers-color-scheme: dark)');
  const resolved = resolveMode(theme.colorMode, prefersDark);

  useIsomorphicLayoutEffect(() => {
    applyThemeToDocument(
      theme,
      { effects, typeScale, motionSpeed, transitions },
      resolved,
    );
  }, [theme, effects, typeScale, motionSpeed, transitions, resolved]);

  return <>{children}</>;
}
