/**
 * Prism — Theme contract (FROZEN).
 *
 * Worker 3 owns `src/theme/**` and may EXTEND this file (add presets, add
 * tokens) but must never rename or remove existing exports — Worker 2's stores
 * and Worker 4's components depend on them.
 *
 * The tokens defined here are the single source of truth for the design system.
 * `themeToCssVariables()` maps them to `--prism-*` custom properties in
 * `tokens.css`. Keep both in sync.
 */

/* -------------------------------------------------------------------------- */
/* Primitives                                                                  */
/* -------------------------------------------------------------------------- */

/** Content density of the shell. Consumed via `[data-density]` on <html>. */
export type Density = 'comfortable' | 'cozy' | 'compact';

/**
 * Radius scale applied to every rounded surface. Implemented by remapping
 * `--prism-radius-*` to a different step of the radius ramp.
 */
export type RadiusScale = 'sharp' | 'soft' | 'round' | 'pill';

/** Color scheme. `system` follows `prefers-color-scheme`. */
export type ColorMode = 'dark' | 'light' | 'system';

/** High-level shell layout. Consumed via `[data-layout]` on <html>. */
export type LayoutVariant = 'sidebar' | 'topnav' | 'hybrid' | 'compact';

/** Deep-partial helper used for preset overrides and patch actions. */
export type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends readonly unknown[] | Function
    ? T[K]
    : T[K] extends object
      ? DeepPartial<T[K]>
      : T[K];
};

/* -------------------------------------------------------------------------- */
/* Token groups                                                                */
/* -------------------------------------------------------------------------- */

/** Base colors, accent and semantic colors. -> `--prism-color-*` */
export interface ColorTokens {
  /** App background, bottom of the elevation ramp. */
  bg: string;
  /** Slightly raised background (page gutters, sidebars). */
  bgElevated: string;
  /** Default card / panel surface. */
  surface: string;
  /** Surface on hover. */
  surfaceHover: string;
  /** Surface on press / selected. */
  surfaceActive: string;
  /** Sunken surface (inputs, wells, tracklists). */
  surfaceSunken: string;
  /** Scrim behind modals / drawers. */
  overlay: string;
  accent: string;
  accentHover: string;
  accentActive: string;
  /** Text/icon color that sits on top of `accent`. */
  accentContrast: string;
  /** Primary text. */
  text: string;
  /** Secondary text. */
  textMuted: string;
  /** Tertiary / disabled text. */
  textSubtle: string;
  border: string;
  borderStrong: string;
  success: string;
  warning: string;
  danger: string;
  /** Visible outline for keyboard focus. */
  focusRing: string;
}

/** Corner radii ramp. -> `--prism-radius-*` */
export interface RadiusTokens {
  none: string;
  xs: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
  '2xl': string;
  full: string;
}

/** Spacing ramp, numeric keys 0..12 (4px base). -> `--prism-space-*` */
export type SpaceToken =
  | '0'
  | '1'
  | '2'
  | '3'
  | '4'
  | '5'
  | '6'
  | '7'
  | '8'
  | '9'
  | '10'
  | '11'
  | '12';
export type SpaceTokens = Record<SpaceToken, string>;

/** Typography. -> `--prism-font-*`, `--prism-line-height-*`, `--prism-letter-spacing-*` */
export interface TypographyTokens {
  fontSans: string;
  fontMono: string;
  sizeXs: string;
  sizeSm: string;
  sizeMd: string;
  sizeLg: string;
  sizeXl: string;
  size2xl: string;
  size3xl: string;
  weightRegular: string;
  weightMedium: string;
  weightSemibold: string;
  weightBold: string;
  lineHeightTight: string;
  lineHeightNormal: string;
  lineHeightRelaxed: string;
  letterSpacingTight: string;
  letterSpacingNormal: string;
  letterSpacingWide: string;
}

/** Elevation shadows. -> `--prism-shadow-*` */
export interface ShadowTokens {
  xs: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
}

/** Backdrop blur amounts (for frosted glass). -> `--prism-blur-*` */
export interface BlurTokens {
  sm: string;
  md: string;
  lg: string;
  xl: string;
}

/** Motion durations & easing curves. -> `--prism-motion-*` */
export interface MotionTokens {
  durationInstant: string;
  durationFast: string;
  durationBase: string;
  durationSlow: string;
  durationSlower: string;
  easeStandard: string;
  easeEmphasized: string;
  easeDecelerate: string;
  easeAccelerate: string;
}

/** The complete set of customizable design tokens. */
export interface ThemeTokens {
  colors: ColorTokens;
  radii: RadiusTokens;
  spacing: SpaceTokens;
  typography: TypographyTokens;
  shadows: ShadowTokens;
  blur: BlurTokens;
  motion: MotionTokens;
}

/* -------------------------------------------------------------------------- */
/* Ambient effects (W3 extension, applied by ThemeProvider)                    */
/* -------------------------------------------------------------------------- */

/** Decorative app background treatment. */
export type BackgroundStyle = 'solid' | 'gradient' | 'mesh' | 'noise';

/**
 * Non-token customization owned by the customizer (W3). Kept separate from the
 * frozen `ThemeTokens` shape so the store/preset seam stays intact.
 */
export interface ThemeEffects {
  /** Ambient backdrop style painted behind the shell. */
  background: BackgroundStyle;
  /** Strength (0..100) of the accent wash for `gradient` / `mesh`. */
  gradientIntensity: number;
  /** Opacity (0..100) of the film grain for `noise`. */
  noiseOpacity: number;
  /** Adds an accent bloom to the ambient backdrop & accent surfaces. */
  glow: boolean;
  /** Frosted-glass backdrop on Prism panels (customizer, drawers…). */
  backdropBlur: boolean;
}

export const DEFAULT_EFFECTS: ThemeEffects = {
  background: 'solid',
  gradientIntensity: 55,
  noiseOpacity: 12,
  glow: false,
  backdropBlur: false,
};

/**
 * Extra, non-token knobs projected onto the document alongside the theme
 * tokens. All fields are optional so `themeToCssVariables(theme)` stays valid.
 */
export interface ThemeProjection {
  effects?: Partial<ThemeEffects>;
  /** Global typography scale multiplier (1 = untouched). */
  typeScale?: number;
  /** Motion duration multiplier (1 = untouched). */
  motionSpeed?: number;
  /** When false, all motion durations collapse to 0 (fades off). */
  transitions?: boolean;
}

/* -------------------------------------------------------------------------- */
/* Theme & presets                                                             */
/* -------------------------------------------------------------------------- */

/** A fully resolved theme (tokens + behavioral knobs). */
export interface Theme {
  id: string;
  name: string;
  description: string;
  colorMode: ColorMode;
  layout: LayoutVariant;
  density: Density;
  radiusScale: RadiusScale;
  tokens: ThemeTokens;
}

/** A partial override that can be applied on top of the default tokens. */
export interface ThemePreset {
  id: string;
  name: string;
  description: string;
  colorMode: ColorMode;
  /** Deep-partial token overrides. */
  tokens: DeepPartial<ThemeTokens>;
  layout?: LayoutVariant;
  density?: Density;
  radiusScale?: RadiusScale;
  /** Suggested ambient/effect settings applied when the preset is selected. */
  effects?: Partial<ThemeEffects>;
  /** Suggested global typography scale. */
  typeScale?: number;
  /** Suggested motion speed multiplier. */
  motionSpeed?: number;
}

/** Patch type accepted by the theme store's `patchTokens`. */
export type ThemeTokensPatch = DeepPartial<ThemeTokens>;

/* -------------------------------------------------------------------------- */
/* Defaults                                                                    */
/* -------------------------------------------------------------------------- */

export const DEFAULT_TOKENS: ThemeTokens = {
  colors: {
    bg: '#0a0a0f',
    bgElevated: '#12121a',
    surface: '#17171f',
    surfaceHover: '#1f1f2b',
    surfaceActive: '#262634',
    surfaceSunken: '#06060a',
    overlay: 'rgba(4, 4, 8, 0.72)',
    accent: '#1ed760',
    accentHover: '#3be477',
    accentActive: '#17b34c',
    accentContrast: '#04120a',
    text: '#f5f5f7',
    textMuted: '#a8a8b6',
    textSubtle: '#6f6f80',
    border: '#26262f',
    borderStrong: '#3a3a48',
    success: '#1ed760',
    warning: '#f2c94c',
    danger: '#ff5c5c',
    focusRing: '#8b7bff',
  },
  radii: {
    none: '0px',
    xs: '2px',
    sm: '4px',
    md: '8px',
    lg: '12px',
    xl: '18px',
    '2xl': '28px',
    full: '9999px',
  },
  spacing: {
    '0': '0rem',
    '1': '0.25rem',
    '2': '0.5rem',
    '3': '0.75rem',
    '4': '1rem',
    '5': '1.25rem',
    '6': '1.5rem',
    '7': '1.75rem',
    '8': '2rem',
    '9': '2.5rem',
    '10': '3rem',
    '11': '4rem',
    '12': '6rem',
  },
  typography: {
    fontSans:
      "'Inter', system-ui, -apple-system, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif",
    fontMono:
      "'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
    sizeXs: '0.75rem',
    sizeSm: '0.875rem',
    sizeMd: '1rem',
    sizeLg: '1.125rem',
    sizeXl: '1.375rem',
    size2xl: '1.75rem',
    size3xl: '2.5rem',
    weightRegular: '400',
    weightMedium: '500',
    weightSemibold: '600',
    weightBold: '700',
    lineHeightTight: '1.2',
    lineHeightNormal: '1.5',
    lineHeightRelaxed: '1.7',
    letterSpacingTight: '-0.02em',
    letterSpacingNormal: '0em',
    letterSpacingWide: '0.04em',
  },
  shadows: {
    xs: '0 1px 2px rgba(0, 0, 0, 0.40)',
    sm: '0 2px 6px rgba(0, 0, 0, 0.45)',
    md: '0 8px 24px rgba(0, 0, 0, 0.50)',
    lg: '0 16px 40px rgba(0, 0, 0, 0.55)',
    xl: '0 28px 64px rgba(0, 0, 0, 0.60)',
  },
  blur: {
    sm: '6px',
    md: '12px',
    lg: '20px',
    xl: '40px',
  },
  motion: {
    durationInstant: '80ms',
    durationFast: '140ms',
    durationBase: '220ms',
    durationSlow: '340ms',
    durationSlower: '520ms',
    easeStandard: 'cubic-bezier(0.2, 0, 0, 1)',
    easeEmphasized: 'cubic-bezier(0.3, 0, 0, 1)',
    easeDecelerate: 'cubic-bezier(0, 0, 0, 1)',
    easeAccelerate: 'cubic-bezier(0.3, 0, 1, 1)',
  },
};

export const DEFAULT_THEME_ID = 'prism-default';

/** Built-in presets. Worker 3 may append more. */
export const THEME_PRESETS: ThemePreset[] = [
  {
    id: DEFAULT_THEME_ID,
    name: 'Prism Default',
    description: 'The signature Prism dark theme.',
    colorMode: 'dark',
    layout: 'sidebar',
    density: 'comfortable',
    radiusScale: 'soft',
    tokens: {},
  },
  {
    id: 'prism-midnight',
    name: 'Midnight',
    description: 'Deeper blacks, tighter radius, compact spacing.',
    colorMode: 'dark',
    layout: 'hybrid',
    density: 'compact',
    radiusScale: 'sharp',
    tokens: {
      colors: {
        bg: '#05050a',
        bgElevated: '#0b0b12',
        surface: '#0f0f17',
        surfaceHover: '#16161f',
        accent: '#7c5cff',
        accentHover: '#8f74ff',
        accentActive: '#6847f0',
        accentContrast: '#0b0618',
        focusRing: '#1ed760',
      },
    },
  },
  {
    id: 'prism-daylight',
    name: 'Daylight',
    description: 'A bright, high-contrast light theme.',
    colorMode: 'light',
    layout: 'topnav',
    density: 'comfortable',
    radiusScale: 'round',
    tokens: {
      colors: {
        bg: '#f6f6f8',
        bgElevated: '#ffffff',
        surface: '#ffffff',
        surfaceHover: '#f0f0f4',
        surfaceActive: '#e6e6ee',
        surfaceSunken: '#ececf1',
        overlay: 'rgba(20, 20, 30, 0.32)',
        accent: '#0f9d58',
        accentHover: '#0c8449',
        accentActive: '#0a6f3d',
        accentContrast: '#ffffff',
        text: '#14141c',
        textMuted: '#55556a',
        textSubtle: '#8a8a9d',
        border: '#e1e1ea',
        borderStrong: '#c8c8d6',
        focusRing: '#6847f0',
      },
      shadows: {
        md: '0 8px 24px rgba(20, 20, 40, 0.12)',
        lg: '0 16px 40px rgba(20, 20, 40, 0.16)',
      },
    },
  },
  {
    id: 'nord-frost',
    name: 'Nord Frost',
    description: 'The cool, arctic Nord palette with frosty cyan accents.',
    colorMode: 'dark',
    layout: 'sidebar',
    density: 'comfortable',
    radiusScale: 'soft',
    tokens: {
      colors: {
        bg: '#2e3440',
        bgElevated: '#3b4252',
        surface: '#3b4252',
        surfaceHover: '#434c5e',
        surfaceActive: '#4c566a',
        surfaceSunken: '#272c36',
        overlay: 'rgba(20, 24, 32, 0.72)',
        accent: '#88c0d0',
        accentHover: '#9fd0dd',
        accentActive: '#6fa9bd',
        accentContrast: '#1c2530',
        text: '#eceff4',
        textMuted: '#c2ccdd',
        textSubtle: '#7b8699',
        border: '#434c5e',
        borderStrong: '#4c566a',
        focusRing: '#a3be8c',
      },
      shadows: {
        md: '0 8px 24px rgba(10, 12, 18, 0.5)',
        lg: '0 16px 40px rgba(10, 12, 18, 0.55)',
      },
    },
  },
  {
    id: 'solarized',
    name: 'Solarized',
    description: 'Precision-tuned Solarized Dark with balanced, low-fatigue contrast.',
    colorMode: 'dark',
    layout: 'hybrid',
    density: 'cozy',
    radiusScale: 'soft',
    tokens: {
      colors: {
        bg: '#002b36',
        bgElevated: '#073642',
        surface: '#073642',
        surfaceHover: '#0b4250',
        surfaceActive: '#104e5e',
        surfaceSunken: '#00212b',
        overlay: 'rgba(0, 20, 26, 0.72)',
        accent: '#268bd2',
        accentHover: '#3aa0e8',
        accentActive: '#1c76b8',
        accentContrast: '#001b24',
        text: '#93a1a1',
        textMuted: '#839496',
        textSubtle: '#657b83',
        border: '#0b3d4a',
        borderStrong: '#17505f',
        focusRing: '#2aa198',
      },
    },
  },
  {
    id: 'dracula',
    name: 'Dracula',
    description: 'The legendary purple-and-pink Dracula theme, playful and vivid.',
    colorMode: 'dark',
    layout: 'hybrid',
    density: 'cozy',
    radiusScale: 'round',
    tokens: {
      colors: {
        bg: '#282a36',
        bgElevated: '#21222c',
        surface: '#343746',
        surfaceHover: '#3d4152',
        surfaceActive: '#464b60',
        surfaceSunken: '#1e1f29',
        overlay: 'rgba(15, 16, 22, 0.76)',
        accent: '#bd93f9',
        accentHover: '#cba9ff',
        accentActive: '#a679e0',
        accentContrast: '#1a1226',
        text: '#f8f8f2',
        textMuted: '#b9bccb',
        textSubtle: '#7c8093',
        border: '#44475a',
        borderStrong: '#5a5f78',
        focusRing: '#ff79c6',
      },
    },
  },
  {
    id: 'tokyo-night',
    name: 'Tokyo Night',
    description: 'Neon-drenched indigo streets with electric blue highlights.',
    colorMode: 'dark',
    layout: 'sidebar',
    density: 'comfortable',
    radiusScale: 'soft',
    tokens: {
      colors: {
        bg: '#1a1b26',
        bgElevated: '#16161e',
        surface: '#24283b',
        surfaceHover: '#2f3452',
        surfaceActive: '#3b4261',
        surfaceSunken: '#13131a',
        overlay: 'rgba(13, 14, 20, 0.76)',
        accent: '#7aa2f7',
        accentHover: '#9bb8ff',
        accentActive: '#5c86e0',
        accentContrast: '#0d1b3a',
        text: '#c0caf5',
        textMuted: '#9aa5ce',
        textSubtle: '#636b8b',
        border: '#2f334d',
        borderStrong: '#414868',
        focusRing: '#bb9af7',
      },
    },
  },
  {
    id: 'rose-pine',
    name: 'Rosé Pine',
    description: 'Muted rose and iris tones on a soft midnight base — all natural.',
    colorMode: 'dark',
    layout: 'hybrid',
    density: 'comfortable',
    radiusScale: 'round',
    tokens: {
      colors: {
        bg: '#191724',
        bgElevated: '#1f1d2e',
        surface: '#26233a',
        surfaceHover: '#2e2b44',
        surfaceActive: '#393552',
        surfaceSunken: '#14121c',
        overlay: 'rgba(10, 9, 15, 0.74)',
        accent: '#eb6f92',
        accentHover: '#ff8aa6',
        accentActive: '#d15c7f',
        accentContrast: '#23101a',
        text: '#e0def4',
        textMuted: '#908caa',
        textSubtle: '#6e6a86',
        border: '#403d52',
        borderStrong: '#524f67',
        focusRing: '#c4a7e7',
      },
    },
  },
  {
    id: 'rose-pine-dawn',
    name: 'Rosé Pine Dawn',
    description: 'The light Rosé Pine variant: warm paper with rose and iris accents.',
    colorMode: 'light',
    layout: 'topnav',
    density: 'comfortable',
    radiusScale: 'round',
    tokens: {
      colors: {
        bg: '#faf4ed',
        bgElevated: '#fffaf3',
        surface: '#fffaf3',
        surfaceHover: '#f4ede4',
        surfaceActive: '#ece0d1',
        surfaceSunken: '#f2e9e1',
        overlay: 'rgba(87, 82, 121, 0.26)',
        accent: '#d7827e',
        accentHover: '#e59a96',
        accentActive: '#c06a66',
        accentContrast: '#fff5f5',
        text: '#575279',
        textMuted: '#797593',
        textSubtle: '#9893a5',
        border: '#e4dcd3',
        borderStrong: '#cecacd',
        focusRing: '#907aa9',
      },
      shadows: {
        md: '0 8px 24px rgba(87, 82, 121, 0.14)',
        lg: '0 16px 40px rgba(87, 82, 121, 0.18)',
      },
    },
  },
  {
    id: 'cyberpunk',
    name: 'Cyberpunk Neon',
    description: 'High-voltage hot-pink neon over near-black with a glowing accent wash.',
    colorMode: 'dark',
    layout: 'compact',
    density: 'compact',
    radiusScale: 'sharp',
    effects: { background: 'gradient', gradientIntensity: 78, glow: true },
    motionSpeed: 1.1,
    tokens: {
      colors: {
        bg: '#0b0212',
        bgElevated: '#12041c',
        surface: '#180826',
        surfaceHover: '#24103a',
        surfaceActive: '#30164c',
        surfaceSunken: '#07010c',
        overlay: 'rgba(8, 0, 16, 0.8)',
        accent: '#ff2e88',
        accentHover: '#ff5aa3',
        accentActive: '#d61f70',
        accentContrast: '#1a0310',
        text: '#ede7ff',
        textMuted: '#b9a7d9',
        textSubtle: '#7d6a9e',
        border: '#3a1f5c',
        borderStrong: '#5b2e8c',
        success: '#39ff14',
        warning: '#ffd60a',
        danger: '#ff3864',
        focusRing: '#00f0ff',
      },
    },
  },
  {
    id: 'forest',
    name: 'Forest',
    description: 'Deep woodland greens with a mossy mesh backdrop.',
    colorMode: 'dark',
    layout: 'sidebar',
    density: 'comfortable',
    radiusScale: 'soft',
    effects: { background: 'mesh', gradientIntensity: 45 },
    tokens: {
      colors: {
        bg: '#0f1712',
        bgElevated: '#14201a',
        surface: '#182a20',
        surfaceHover: '#1f3629',
        surfaceActive: '#274433',
        surfaceSunken: '#0a100d',
        overlay: 'rgba(6, 12, 9, 0.74)',
        accent: '#5dd39e',
        accentHover: '#7eeab6',
        accentActive: '#45b480',
        accentContrast: '#06231a',
        text: '#e8f5ec',
        textMuted: '#a3c4b1',
        textSubtle: '#6c8a79',
        border: '#26402f',
        borderStrong: '#355943',
        focusRing: '#f2c94c',
      },
    },
  },
  {
    id: 'amethyst',
    name: 'Amethyst',
    description: 'Luminous violet layered on a jewel-toned plum base.',
    colorMode: 'dark',
    layout: 'sidebar',
    density: 'cozy',
    radiusScale: 'round',
    tokens: {
      colors: {
        bg: '#150f24',
        bgElevated: '#1d1633',
        surface: '#241b40',
        surfaceHover: '#2e2352',
        surfaceActive: '#392b66',
        surfaceSunken: '#0f0a1b',
        overlay: 'rgba(11, 7, 20, 0.76)',
        accent: '#a78bfa',
        accentHover: '#bda4ff',
        accentActive: '#8b6ff0',
        accentContrast: '#140a2a',
        text: '#f0ebff',
        textMuted: '#b7a9e0',
        textSubtle: '#7d6fb0',
        border: '#322552',
        borderStrong: '#463570',
        focusRing: '#f0abfc',
      },
    },
  },
  {
    id: 'sunset',
    name: 'Sunset',
    description: 'Warm amber glow across a dusky horizon, soft and cinematic.',
    colorMode: 'dark',
    layout: 'compact',
    density: 'cozy',
    radiusScale: 'round',
    effects: { background: 'gradient', gradientIntensity: 66, glow: true },
    tokens: {
      colors: {
        bg: '#1a0f12',
        bgElevated: '#241318',
        surface: '#2c1a1f',
        surfaceHover: '#3a2229',
        surfaceActive: '#482a33',
        surfaceSunken: '#120a0c',
        overlay: 'rgba(18, 8, 10, 0.76)',
        accent: '#fb923c',
        accentHover: '#fdae66',
        accentActive: '#e07a26',
        accentContrast: '#2a1206',
        text: '#fdefe9',
        textMuted: '#d3a99a',
        textSubtle: '#9c7568',
        border: '#3b2129',
        borderStrong: '#553039',
        focusRing: '#f472b6',
      },
    },
  },
  {
    id: 'cobalt',
    name: 'Cobalt',
    description: 'Crisp engineering blue on a deep navy canvas.',
    colorMode: 'dark',
    layout: 'sidebar',
    density: 'comfortable',
    radiusScale: 'soft',
    tokens: {
      colors: {
        bg: '#0b1020',
        bgElevated: '#0f1628',
        surface: '#141d33',
        surfaceHover: '#1b2740',
        surfaceActive: '#223050',
        surfaceSunken: '#070b16',
        overlay: 'rgba(5, 8, 16, 0.76)',
        accent: '#3b82f6',
        accentHover: '#60a5fa',
        accentActive: '#2563eb',
        accentContrast: '#04122e',
        text: '#e6edf8',
        textMuted: '#9fb0cc',
        textSubtle: '#6b7c99',
        border: '#22304d',
        borderStrong: '#33456b',
        focusRing: '#22d3ee',
      },
    },
  },
  {
    id: 'glass',
    name: 'Glass',
    description: 'Frosted translucent panes over a colored mesh — everything blurs.',
    colorMode: 'dark',
    layout: 'sidebar',
    density: 'comfortable',
    radiusScale: 'round',
    effects: { background: 'mesh', gradientIntensity: 62, backdropBlur: true },
    tokens: {
      colors: {
        bg: '#0d1017',
        bgElevated: '#12161f',
        surface: 'rgba(255, 255, 255, 0.06)',
        surfaceHover: 'rgba(255, 255, 255, 0.1)',
        surfaceActive: 'rgba(255, 255, 255, 0.14)',
        surfaceSunken: 'rgba(0, 0, 0, 0.35)',
        overlay: 'rgba(6, 8, 12, 0.6)',
        accent: '#7dd3fc',
        accentHover: '#a5e3ff',
        accentActive: '#4fb8e6',
        accentContrast: '#08222e',
        text: '#eef4ff',
        textMuted: '#aebcd4',
        textSubtle: '#7d8aa3',
        border: 'rgba(255, 255, 255, 0.12)',
        borderStrong: 'rgba(255, 255, 255, 0.2)',
        focusRing: '#c4b5fd',
      },
      shadows: {
        sm: '0 2px 8px rgba(0, 0, 0, 0.35)',
        md: '0 10px 30px rgba(0, 0, 0, 0.4)',
        lg: '0 18px 50px rgba(0, 0, 0, 0.5)',
      },
    },
  },
  {
    id: 'paper',
    name: 'Paper',
    description: 'Warm, low-contrast off-white reading surface with amber accents.',
    colorMode: 'light',
    layout: 'topnav',
    density: 'comfortable',
    radiusScale: 'soft',
    tokens: {
      colors: {
        bg: '#f7f4ee',
        bgElevated: '#ffffff',
        surface: '#ffffff',
        surfaceHover: '#f1ede4',
        surfaceActive: '#e8e2d6',
        surfaceSunken: '#efeae0',
        overlay: 'rgba(40, 36, 28, 0.28)',
        accent: '#c2410c',
        accentHover: '#ea580c',
        accentActive: '#9a3412',
        accentContrast: '#fff7ed',
        text: '#2b2620',
        textMuted: '#6b6357',
        textSubtle: '#9a917f',
        border: '#e4ddd0',
        borderStrong: '#cfc6b5',
        focusRing: '#2563eb',
      },
      shadows: {
        sm: '0 2px 6px rgba(60, 50, 35, 0.08)',
        md: '0 8px 24px rgba(60, 50, 35, 0.12)',
        lg: '0 16px 40px rgba(60, 50, 35, 0.16)',
      },
    },
  },
  {
    id: 'mono',
    name: 'Mono',
    description: 'Pure grayscale, maximum neutrality — a blank canvas for tinkering.',
    colorMode: 'dark',
    layout: 'compact',
    density: 'comfortable',
    radiusScale: 'sharp',
    tokens: {
      colors: {
        bg: '#0b0b0b',
        bgElevated: '#141414',
        surface: '#1b1b1b',
        surfaceHover: '#232323',
        surfaceActive: '#2b2b2b',
        surfaceSunken: '#050505',
        overlay: 'rgba(0, 0, 0, 0.76)',
        accent: '#f5f5f5',
        accentHover: '#ffffff',
        accentActive: '#d4d4d4',
        accentContrast: '#0b0b0b',
        text: '#fafafa',
        textMuted: '#b3b3b3',
        textSubtle: '#737373',
        border: '#2a2a2a',
        borderStrong: '#3d3d3d',
        focusRing: '#a3a3a3',
      },
    },
  },
];

/* -------------------------------------------------------------------------- */
/* Helpers                                                                     */
/* -------------------------------------------------------------------------- */

const isPlainObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

/** Recursively merge a deep-partial token patch into tokens (immutably). */
export function mergeThemeTokens(
  base: ThemeTokens,
  patch: ThemeTokensPatch,
): ThemeTokens {
  const out: Record<string, unknown> = { ...base };
  for (const [key, value] of Object.entries(patch)) {
    if (value === undefined) continue;
    const current = out[key];
    out[key] =
      isPlainObject(current) && isPlainObject(value)
        ? { ...current, ...mergeTokensObject(current, value) }
        : value;
  }
  return out as unknown as ThemeTokens;
}

function mergeTokensObject(
  base: Record<string, unknown>,
  patch: Record<string, unknown>,
): Record<string, unknown> {
  const out: Record<string, unknown> = { ...base };
  for (const [key, value] of Object.entries(patch)) {
    if (value === undefined) continue;
    const current = out[key];
    out[key] =
      isPlainObject(current) && isPlainObject(value)
        ? { ...current, ...mergeTokensObject(current, value) }
        : value;
  }
  return out;
}

/** Resolve a preset into a complete `Theme`. */
export function createTheme(preset: ThemePreset): Theme {
  return {
    id: preset.id,
    name: preset.name,
    description: preset.description,
    colorMode: preset.colorMode,
    layout: preset.layout ?? 'sidebar',
    density: preset.density ?? 'comfortable',
    radiusScale: preset.radiusScale ?? 'soft',
    tokens: mergeThemeTokens(DEFAULT_TOKENS, preset.tokens),
  };
}

export const DEFAULT_THEME: Theme = createTheme(THEME_PRESETS[0]);

const kebab = (input: string): string =>
  input.replace(/[A-Z]/g, (m) => `-${m.toLowerCase()}`);

const clamp = (value: number, min: number, max: number): number =>
  Math.min(max, Math.max(min, value));

const FONT_SIZE_KEYS = [
  'sizeXs',
  'sizeSm',
  'sizeMd',
  'sizeLg',
  'sizeXl',
  'size2xl',
  'size3xl',
] as const;

const MOTION_DURATION_KEYS = [
  'durationInstant',
  'durationFast',
  'durationBase',
  'durationSlow',
  'durationSlower',
] as const;

/**
 * Flatten a `Theme`'s tokens into a map of `--prism-*` CSS custom properties.
 *
 * An optional `projection` layers the customizer's non-token knobs (ambient
 * effects, typography scale, motion speed) on top. Calling it with a single
 * argument keeps the original, frozen behavior.
 */
export function themeToCssVariables(
  theme: Theme,
  projection?: ThemeProjection,
): Record<string, string> {
  const { tokens } = theme;
  const vars: Record<string, string> = {};

  for (const [key, value] of Object.entries(tokens.colors)) {
    vars[`--prism-color-${kebab(key)}`] = value;
  }
  for (const [key, value] of Object.entries(tokens.radii)) {
    vars[`--prism-radius-${key}`] = value;
  }
  for (const [key, value] of Object.entries(tokens.spacing)) {
    vars[`--prism-space-${key}`] = value;
  }
  for (const [key, value] of Object.entries(tokens.typography)) {
    let name = kebab(key);
    if (name.startsWith('size-') || name.startsWith('weight-')) {
      name = `font-${name}`;
    }
    vars[`--prism-${name}`] = value;
  }
  for (const [key, value] of Object.entries(tokens.shadows)) {
    vars[`--prism-shadow-${key}`] = value;
  }
  for (const [key, value] of Object.entries(tokens.blur)) {
    vars[`--prism-blur-${key}`] = value;
  }
  for (const [key, value] of Object.entries(tokens.motion)) {
    vars[`--prism-motion-${kebab(key)}`] = value;
  }

  vars['--prism-layout'] = theme.layout;
  vars['--prism-density'] = theme.density;
  vars['--prism-radius-scale'] = theme.radiusScale;
  vars['--prism-color-mode'] = theme.colorMode;

  /* ----- Ambient effects (W3 projection) -------------------------------- */
  const effects: ThemeEffects = { ...DEFAULT_EFFECTS, ...projection?.effects };

  // Keep the true, opaque background available for the canvas even when the
  // `bg` token is made translucent to reveal the ambient backdrop.
  vars['--prism-color-bg-solid'] = tokens.colors.bg;
  vars['--prism-effect-background'] = effects.background;
  vars['--prism-effect-gradient'] = String(clamp(effects.gradientIntensity, 0, 100));
  vars['--prism-effect-noise-opacity'] = String(
    clamp(effects.noiseOpacity, 0, 100) / 100,
  );
  vars['--prism-effect-glow'] = effects.glow ? '1' : '0';
  vars['--prism-effect-backdrop-blur'] = effects.backdropBlur
    ? 'blur(var(--prism-blur-md))'
    : 'none';

  if (effects.background === 'gradient' || effects.background === 'mesh') {
    // Let the body backdrop show through the shell surfaces.
    vars['--prism-color-bg'] = `color-mix(in srgb, ${tokens.colors.bg} 58%, transparent)`;
  }

  /* ----- Typography scale ---------------------------------------------- */
  const typeScale = projection?.typeScale ?? 1;
  if (typeScale !== 1) {
    for (const key of FONT_SIZE_KEYS) {
      const name = kebab(key).replace(/^size-?/, '');
      vars[`--prism-font-size-${name}`] = `calc(${tokens.typography[key]} * ${typeScale})`;
    }
  }

  /* ----- Motion speed / fades ------------------------------------------ */
  const transitions = projection?.transitions ?? true;
  const factor = transitions ? (projection?.motionSpeed ?? 1) : 0;
  if (factor !== 1) {
    for (const key of MOTION_DURATION_KEYS) {
      vars[`--prism-motion-${kebab(key)}`] = `calc(${tokens.motion[key]} * ${factor})`;
    }
  }

  return vars;
}
