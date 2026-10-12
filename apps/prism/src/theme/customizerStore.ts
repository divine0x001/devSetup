/**
 * Prism — Customizer store (W3).
 *
 * Owns the *non-token* customization that the frozen theme store cannot hold:
 * ambient effects, typography scale, motion speed and user-saved presets.
 * Persisted to localStorage (`prism.customizer`) so it survives reloads.
 *
 * `ThemeProvider` subscribes to this store and projects it onto the document
 * through `themeToCssVariables(theme, projection)`. The customizer drives it.
 */

import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';

import { DEFAULT_EFFECTS, type Theme, type ThemeEffects } from '@/theme/types';

export interface CustomizerStoreState {
  /** Ambient backdrop + effects. */
  effects: ThemeEffects;
  /** Global typography scale multiplier (1 = untouched). */
  typeScale: number;
  /** Motion duration multiplier (1 = untouched). */
  motionSpeed: number;
  /** When false, all interface transitions collapse to instant. */
  transitions: boolean;
  /** Themes saved by the user ("Enregistrer comme preset perso"). */
  customPresets: Theme[];

  setEffect: <K extends keyof ThemeEffects>(key: K, value: ThemeEffects[K]) => void;
  patchEffects: (patch: Partial<ThemeEffects>) => void;
  setTypeScale: (value: number) => void;
  setMotionSpeed: (value: number) => void;
  setTransitions: (value: boolean) => void;
  resetEffects: () => void;

  saveCustomPreset: (theme: Theme) => void;
  removeCustomPreset: (id: string) => void;
  clearCustomPresets: () => void;
}

export const useCustomizerStore = create<CustomizerStoreState>()(
  persist(
    (set) => ({
      effects: DEFAULT_EFFECTS,
      typeScale: 1,
      motionSpeed: 1,
      transitions: true,
      customPresets: [],

      setEffect: (key, value) =>
        set((state) => ({ effects: { ...state.effects, [key]: value } })),
      patchEffects: (patch) =>
        set((state) => ({ effects: { ...state.effects, ...patch } })),
      setTypeScale: (typeScale) => set({ typeScale }),
      setMotionSpeed: (motionSpeed) => set({ motionSpeed }),
      setTransitions: (transitions) => set({ transitions }),
      resetEffects: () =>
        set({
          effects: DEFAULT_EFFECTS,
          typeScale: 1,
          motionSpeed: 1,
          transitions: true,
        }),

      saveCustomPreset: (theme) =>
        set((state) => {
          const id = theme.id.startsWith('custom-')
            ? theme.id
            : `custom-${Date.now().toString(36)}`;
          const existing = state.customPresets.filter((preset) => preset.id !== id);
          return { customPresets: [...existing, { ...theme, id }] };
        }),
      removeCustomPreset: (id) =>
        set((state) => ({
          customPresets: state.customPresets.filter((preset) => preset.id !== id),
        })),
      clearCustomPresets: () => set({ customPresets: [] }),
    }),
    {
      name: 'prism.customizer',
      storage: createJSONStorage(() => localStorage),
      version: 1,
    },
  ),
);
