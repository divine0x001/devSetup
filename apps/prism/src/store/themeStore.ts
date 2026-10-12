/**
 * Prism — Theme store (FROZEN shape).
 *
 * The single place that owns the active `Theme`. `ThemeProvider` subscribes to
 * `state.theme` and projects it onto `--prism-*` CSS variables. The customizer
 * (Worker 3) drives this store through `applyPreset` / `patchTokens`.
 *
 * The active theme is persisted to localStorage (`prism.theme`) so a custom
 * theme survives reloads. Presets always come from code (`THEME_PRESETS`).
 */

import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';

import {
  createTheme,
  DEFAULT_THEME,
  THEME_PRESETS,
  mergeThemeTokens,
  type ColorMode,
  type Density,
  type LayoutVariant,
  type RadiusScale,
  type Theme,
  type ThemePreset,
  type ThemeTokensPatch,
} from '@/theme/types';

export interface ThemeStoreState {
  theme: Theme;
  presets: ThemePreset[];
  applyPreset: (presetId: string) => void;
  setTheme: (theme: Theme) => void;
  setLayout: (layout: LayoutVariant) => void;
  setDensity: (density: Density) => void;
  setRadius: (radiusScale: RadiusScale) => void;
  setColorMode: (colorMode: ColorMode) => void;
  patchTokens: (patch: ThemeTokensPatch) => void;
  reset: () => void;
}

function findPreset(presetId: string): ThemePreset {
  return (
    THEME_PRESETS.find((preset) => preset.id === presetId) ?? THEME_PRESETS[0]
  );
}

export const useThemeStore = create<ThemeStoreState>()(
  persist(
    (set) => ({
      theme: DEFAULT_THEME,
      presets: THEME_PRESETS,
      applyPreset: (presetId) => set({ theme: createTheme(findPreset(presetId)) }),
      setTheme: (theme) => set({ theme }),
      setLayout: (layout) => set((state) => ({ theme: { ...state.theme, layout } })),
      setDensity: (density) => set((state) => ({ theme: { ...state.theme, density } })),
      setRadius: (radiusScale) =>
        set((state) => ({ theme: { ...state.theme, radiusScale } })),
      setColorMode: (colorMode) =>
        set((state) => ({ theme: { ...state.theme, colorMode } })),
      patchTokens: (patch) =>
        set((state) => ({
          theme: { ...state.theme, tokens: mergeThemeTokens(state.theme.tokens, patch) },
        })),
      reset: () => set({ theme: DEFAULT_THEME }),
    }),
    {
      name: 'prism.theme',
      storage: createJSONStorage(() => localStorage),
      version: 1,
      // Only the active theme is persisted; presets are always code-defined.
      partialize: (state) => ({ theme: state.theme }),
    },
  ),
);