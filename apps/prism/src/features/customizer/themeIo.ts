/**
 * Prism — Theme import / export / random helpers (W3).
 *
 * Serializes a `Theme` to JSON, parses user-provided JSON back into a validated
 * `Theme`, and generates "Surprends-moi" random themes. No DOM access at module
 * scope so the pure helpers stay testable; only the download/copy helpers are
 * browser-bound.
 */

import {
  createTheme,
  type ColorMode,
  type ColorTokens,
  type Density,
  type LayoutVariant,
  type RadiusScale,
  type Theme,
  type ThemePreset,
  type ThemeTokensPatch,
} from '@/theme/types';

/* -------------------------------------------------------------------------- */
/* Small validation helpers                                                   */
/* -------------------------------------------------------------------------- */

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

const COLOR_MODES: readonly ColorMode[] = ['dark', 'light', 'system'];
const LAYOUTS: readonly LayoutVariant[] = ['sidebar', 'topnav', 'hybrid', 'compact'];
const DENSITIES: readonly Density[] = ['comfortable', 'cozy', 'compact'];
const RADII: readonly RadiusScale[] = ['sharp', 'soft', 'round', 'pill'];

function asColorMode(value: unknown, fallback: ColorMode): ColorMode {
  return typeof value === 'string' && (COLOR_MODES as readonly string[]).includes(value)
    ? (value as ColorMode)
    : fallback;
}
function asLayout(value: unknown): LayoutVariant | undefined {
  return typeof value === 'string' && (LAYOUTS as readonly string[]).includes(value)
    ? (value as LayoutVariant)
    : undefined;
}
function asDensity(value: unknown): Density | undefined {
  return typeof value === 'string' && (DENSITIES as readonly string[]).includes(value)
    ? (value as Density)
    : undefined;
}
function asRadius(value: unknown): RadiusScale | undefined {
  return typeof value === 'string' && (RADII as readonly string[]).includes(value)
    ? (value as RadiusScale)
    : undefined;
}

/* -------------------------------------------------------------------------- */
/* Serialize / parse                                                          */
/* -------------------------------------------------------------------------- */

/** Pretty JSON snapshot of a theme (what `Exporter` downloads / copies). */
export function serializeTheme(theme: Theme): string {
  return JSON.stringify(theme, null, 2);
}

/**
 * Parse a JSON string (a previously exported `Theme`, or a hand-written
 * `ThemePreset`) into a fully resolved `Theme`. Throws on malformed JSON.
 */
export function parseTheme(text: string): Theme {
  let data: unknown;
  try {
    data = JSON.parse(text);
  } catch {
    throw new Error('JSON invalide — vérifiez votre thème.');
  }
  if (!isRecord(data)) {
    throw new Error('Le thème doit être un objet JSON.');
  }

  const tokens = isRecord(data.tokens) ? (data.tokens as ThemeTokensPatch) : {};

  const preset: ThemePreset = {
    id: typeof data.id === 'string' && data.id ? data.id : `imported-${Date.now().toString(36)}`,
    name: typeof data.name === 'string' && data.name ? data.name : 'Thème importé',
    description:
      typeof data.description === 'string' && data.description
        ? data.description
        : 'Importé depuis un fichier JSON.',
    colorMode: asColorMode(data.colorMode, 'dark'),
    tokens,
    layout: asLayout(data.layout),
    density: asDensity(data.density),
    radiusScale: asRadius(data.radiusScale),
  };

  return createTheme(preset);
}

/** Trigger a browser download of the theme as a `.prism-theme.json` file. */
export function downloadTheme(theme: Theme): void {
  const blob = new Blob([serializeTheme(theme)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = `${theme.id}.prism-theme.json`;
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  URL.revokeObjectURL(url);
}

/** Copy text to the clipboard, with a legacy fallback. Returns success. */
export async function copyText(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    // fall through to the legacy path
  }
  try {
    const field = document.createElement('textarea');
    field.value = text;
    field.style.position = 'fixed';
    field.style.opacity = '0';
    document.body.appendChild(field);
    field.select();
    const ok = document.execCommand('copy');
    document.body.removeChild(field);
    return ok;
  } catch {
    return false;
  }
}

/* -------------------------------------------------------------------------- */
/* Random theme ("Surprends-moi")                                             */
/* -------------------------------------------------------------------------- */

function hslToHex(h: number, s: number, l: number): string {
  const sat = s / 100;
  const lum = l / 100;
  const c = (1 - Math.abs(2 * lum - 1)) * sat;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = lum - c / 2;
  let r = 0;
  let g = 0;
  let b = 0;
  if (h < 60) [r, g, b] = [c, x, 0];
  else if (h < 120) [r, g, b] = [x, c, 0];
  else if (h < 180) [r, g, b] = [0, c, x];
  else if (h < 240) [r, g, b] = [0, x, c];
  else if (h < 300) [r, g, b] = [x, 0, c];
  else [r, g, b] = [c, 0, x];
  const channel = (v: number) =>
    Math.round((v + m) * 255)
      .toString(16)
      .padStart(2, '0');
  return `#${channel(r)}${channel(g)}${channel(b)}`;
}

function pick<T>(items: readonly T[]): T {
  return items[Math.floor(Math.random() * items.length)];
}

const rand = (min: number, max: number): number => min + Math.random() * (max - min);

function randomColors(): { colorMode: ColorMode; colors: ColorTokens } {
  const hue = Math.round(rand(0, 360));
  const accentHue = (hue + Math.round(rand(-45, 45)) + 360) % 360;
  const light = Math.random() > 0.78;

  if (light) {
    return {
      colorMode: 'light',
      colors: {
        bg: hslToHex(hue, 30, 97),
        bgElevated: '#ffffff',
        surface: '#ffffff',
        surfaceHover: hslToHex(hue, 28, 94),
        surfaceActive: hslToHex(hue, 26, 89),
        surfaceSunken: hslToHex(hue, 26, 93),
        overlay: `hsl(${hue} 35% 20% / 0.28)`,
        accent: hslToHex(accentHue, 70, 42),
        accentHover: hslToHex(accentHue, 72, 50),
        accentActive: hslToHex(accentHue, 72, 34),
        accentContrast: '#ffffff',
        text: hslToHex(hue, 30, 14),
        textMuted: hslToHex(hue, 16, 40),
        textSubtle: hslToHex(hue, 14, 56),
        border: hslToHex(hue, 22, 87),
        borderStrong: hslToHex(hue, 20, 78),
        success: hslToHex(145, 55, 38),
        warning: hslToHex(42, 80, 45),
        danger: hslToHex(0, 68, 48),
        focusRing: hslToHex((accentHue + 180) % 360, 60, 45),
      },
    };
  }

  return {
    colorMode: 'dark',
    colors: {
      bg: hslToHex(hue, 28, 6),
      bgElevated: hslToHex(hue, 26, 9),
      surface: hslToHex(hue, 24, 12),
      surfaceHover: hslToHex(hue, 24, 16),
      surfaceActive: hslToHex(hue, 22, 20),
      surfaceSunken: hslToHex(hue, 30, 4),
      overlay: `hsl(${hue} 30% 3% / 0.72)`,
      accent: hslToHex(accentHue, 72, 62),
      accentHover: hslToHex(accentHue, 78, 70),
      accentActive: hslToHex(accentHue, 70, 52),
      accentContrast: hslToHex(accentHue, 55, 10),
      text: hslToHex(hue, 20, 96),
      textMuted: hslToHex(hue, 14, 70),
      textSubtle: hslToHex(hue, 12, 52),
      border: hslToHex(hue, 20, 19),
      borderStrong: hslToHex(hue, 20, 28),
      success: hslToHex(145, 60, 55),
      warning: hslToHex(42, 85, 62),
      danger: hslToHex(0, 72, 62),
      focusRing: hslToHex((accentHue + 180) % 360, 70, 66),
    },
  };
}

/** Generate a coherent, surprising theme. Used by the "Surprends-moi" action. */
export function randomTheme(): Theme {
  const { colorMode, colors } = randomColors();
  const preset: ThemePreset = {
    id: `custom-random-${Date.now().toString(36)}`,
    name: 'Surprise',
    description: 'Thème généré aléatoirement par Prism.',
    colorMode,
    tokens: { colors },
    layout: pick(LAYOUTS),
    density: pick(DENSITIES),
    radiusScale: pick(RADII),
  };
  return createTheme(preset);
}
