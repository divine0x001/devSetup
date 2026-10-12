import { Check, X } from 'lucide-react';
import { useCallback, type CSSProperties } from 'react';

import { Slider, Switch } from '@/components/ui';
import { useThemeStore } from '@/store/themeStore';
import { useCustomizerStore } from '@/theme/customizerStore';
import {
  createTheme,
  type BackgroundStyle,
  type ColorMode,
  type ColorTokens,
  type Density,
  type LayoutVariant,
  type RadiusScale,
  type Theme,
  type ThemePreset,
} from '@/theme/types';

import {
  CollapsibleSection,
  ColorField,
  FieldRow,
  LayoutPreview,
  SegmentedControl,
  type SegmentedOption,
} from './controls';
import styles from './Customizer.module.css';

/* -------------------------------------------------------------------------- */
/* Presets                                                                    */
/* -------------------------------------------------------------------------- */

interface PresetCardProps {
  theme: Theme;
  name: string;
  description: string;
  active: boolean;
  onSelect: () => void;
  onRemove?: () => void;
}

function PresetCard({
  theme,
  name,
  description,
  active,
  onSelect,
  onRemove,
}: PresetCardProps) {
  const colors = theme.tokens.colors;
  const previewStyle = {
    '--prism-swatch-bg': colors.bg,
    '--prism-swatch-surface': colors.surface,
    '--prism-swatch-hover': colors.surfaceHover,
    '--prism-swatch-accent': colors.accent,
    '--prism-swatch-text': colors.text,
    '--prism-swatch-border': colors.border,
  } as CSSProperties;

  return (
    <div className={styles.presetCard} data-active={active || undefined} style={previewStyle}>
      <button
        type="button"
        className={styles.presetSelect}
        aria-pressed={active}
        onClick={onSelect}
      >
        <span className={styles.presetPreview} aria-hidden>
          <span className={styles.swatchTop} />
          <span className={styles.swatchSide} />
          <span className={styles.swatchMain}>
            <span className={styles.swatchLine} />
            <span className={styles.swatchLineShort} />
          </span>
          <span className={styles.swatchDot} />
        </span>
        <span className={styles.presetName}>
          {active ? <Check className={styles.presetCheck} aria-hidden /> : null}
          {name}
        </span>
        <span className={styles.presetDescription}>{description}</span>
      </button>
      {onRemove ? (
        <button
          type="button"
          className={styles.presetRemove}
          onClick={onRemove}
          aria-label={`Supprimer le preset ${name}`}
        >
          <X aria-hidden />
        </button>
      ) : null}
    </div>
  );
}

/** Preset gallery: built-in `THEME_PRESETS` + the user's saved themes. */
export function PresetsSection() {
  const presets = useThemeStore((state) => state.presets);
  const activeId = useThemeStore((state) => state.theme.id);
  const customPresets = useCustomizerStore((state) => state.customPresets);
  const removeCustomPreset = useCustomizerStore((state) => state.removeCustomPreset);

  const applyBuiltIn = useCallback((preset: ThemePreset) => {
    useThemeStore.getState().applyPreset(preset.id);
    const custom = useCustomizerStore.getState();
    custom.resetEffects();
    if (preset.effects) custom.patchEffects(preset.effects);
    custom.setTypeScale(preset.typeScale ?? 1);
    custom.setMotionSpeed(preset.motionSpeed ?? 1);
  }, []);

  const applyCustom = useCallback((theme: Theme) => {
    useThemeStore.getState().setTheme(theme);
  }, []);

  return (
    <CollapsibleSection
      title="Presets"
      description="Des ambiances prêtes à l’emploi — appliquez-les et affinez ensuite."
    >
      <ul className={styles.presetGrid} role="list">
        {presets.map((preset) => (
          <li key={preset.id}>
            <PresetCard
              theme={createTheme(preset)}
              name={preset.name}
              description={preset.description}
              active={preset.id === activeId}
              onSelect={() => applyBuiltIn(preset)}
            />
          </li>
        ))}
        {customPresets.map((theme) => (
          <li key={theme.id}>
            <PresetCard
              theme={theme}
              name={theme.name}
              description={theme.description}
              active={theme.id === activeId}
              onSelect={() => applyCustom(theme)}
              onRemove={() => removeCustomPreset(theme.id)}
            />
          </li>
        ))}
      </ul>
    </CollapsibleSection>
  );
}

/* -------------------------------------------------------------------------- */
/* Colors                                                                     */
/* -------------------------------------------------------------------------- */

const COLOR_FIELDS = [
  { key: 'accent', label: 'Accent' },
  { key: 'bg', label: 'Fond' },
  { key: 'bgElevated', label: 'Fond élevé' },
  { key: 'surface', label: 'Surface' },
  { key: 'surfaceSunken', label: 'Surface creusée' },
  { key: 'text', label: 'Texte' },
  { key: 'textMuted', label: 'Texte atténué' },
  { key: 'border', label: 'Bordure' },
] as const satisfies readonly { key: keyof ColorTokens; label: string }[];

/** Live color pickers wired straight to `themeStore.patchTokens`. */
export function ColorsSection() {
  const colors = useThemeStore((state) => state.theme.tokens.colors);
  const patchTokens = useThemeStore((state) => state.patchTokens);

  const setColor = useCallback(
    (key: keyof ColorTokens, value: string) => {
      const patch: Partial<ColorTokens> = { [key]: value };
      patchTokens({ colors: patch });
    },
    [patchTokens],
  );

  return (
    <CollapsibleSection
      title="Couleurs"
      description="Pipettes en direct : chaque réglage se répercute instantanément."
    >
      <div className={styles.colorGrid}>
        {COLOR_FIELDS.map((field) => (
          <ColorField
            key={field.key}
            label={field.label}
            value={colors[field.key]}
            onChange={(value) => setColor(field.key, value)}
          />
        ))}
      </div>
    </CollapsibleSection>
  );
}

/* -------------------------------------------------------------------------- */
/* Structure (layout, density, radius, color mode)                            */
/* -------------------------------------------------------------------------- */

const LAYOUT_OPTIONS: readonly { value: LayoutVariant; label: string }[] = [
  { value: 'sidebar', label: 'Latérale' },
  { value: 'topnav', label: 'Haut' },
  { value: 'hybrid', label: 'Hybride' },
  { value: 'compact', label: 'Compacte' },
];

const DENSITY_OPTIONS: readonly SegmentedOption<Density>[] = [
  { value: 'comfortable', label: 'Confort' },
  { value: 'cozy', label: 'Cozy' },
  { value: 'compact', label: 'Compact' },
];

const RADIUS_OPTIONS: readonly SegmentedOption<RadiusScale>[] = [
  { value: 'sharp', label: 'Vif' },
  { value: 'soft', label: 'Doux' },
  { value: 'round', label: 'Rond' },
  { value: 'pill', label: 'Pilule' },
];

const MODE_OPTIONS: readonly SegmentedOption<ColorMode>[] = [
  { value: 'light', label: 'Clair' },
  { value: 'dark', label: 'Sombre' },
  { value: 'system', label: 'Système' },
];

/** Shell structure: layout, density, radius scale and light/dark mode. */
export function StructureSection() {
  const layout = useThemeStore((state) => state.theme.layout);
  const density = useThemeStore((state) => state.theme.density);
  const radiusScale = useThemeStore((state) => state.theme.radiusScale);
  const colorMode = useThemeStore((state) => state.theme.colorMode);
  const setLayout = useThemeStore((state) => state.setLayout);
  const setDensity = useThemeStore((state) => state.setDensity);
  const setRadius = useThemeStore((state) => state.setRadius);
  const setColorMode = useThemeStore((state) => state.setColorMode);

  return (
    <CollapsibleSection
      title="Structure"
      description="Disposition, densité, arrondis et mode clair/sombre du shell."
    >
      <div className={styles.optionGrid}>
        {LAYOUT_OPTIONS.map((option) => {
          const active = option.value === layout;
          return (
            <button
              key={option.value}
              type="button"
              className={styles.layoutOption}
              data-active={active || undefined}
              aria-pressed={active}
              onClick={() => setLayout(option.value)}
            >
              <LayoutPreview layout={option.value} />
              <span className={styles.layoutOptionLabel}>{option.label}</span>
            </button>
          );
        })}
      </div>

      <FieldRow label="Densité">
        <SegmentedControl
          value={density}
          options={DENSITY_OPTIONS}
          onChange={setDensity}
          ariaLabel="Densité de l’interface"
        />
      </FieldRow>

      <FieldRow label="Arrondis">
        <SegmentedControl
          value={radiusScale}
          options={RADIUS_OPTIONS}
          onChange={setRadius}
          ariaLabel="Échelle des arrondis"
          columns={4}
        />
      </FieldRow>

      <FieldRow label="Mode">
        <SegmentedControl
          value={colorMode}
          options={MODE_OPTIONS}
          onChange={setColorMode}
          ariaLabel="Mode de couleur"
        />
      </FieldRow>
    </CollapsibleSection>
  );
}

/* -------------------------------------------------------------------------- */
/* Typography & motion                                                        */
/* -------------------------------------------------------------------------- */

const FONT_OPTIONS: readonly { value: string; label: string }[] = [
  {
    value:
      "'Inter', system-ui, -apple-system, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif",
    label: 'Inter',
  },
  {
    value:
      "system-ui, -apple-system, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif",
    label: 'Système',
  },
  {
    value:
      "'Iowan Old Style', 'Palatino Linotype', Palatino, Georgia, 'Times New Roman', serif",
    label: 'Serif',
  },
  {
    value:
      "'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
    label: 'Mono',
  },
  {
    value: "ui-rounded, 'SF Pro Rounded', 'Nunito', system-ui, sans-serif",
    label: 'Arrondie',
  },
];

/** Font family, global type scale, motion speed and fades. */
export function TypographyMotionSection() {
  const fontSans = useThemeStore((state) => state.theme.tokens.typography.fontSans);
  const patchTokens = useThemeStore((state) => state.patchTokens);
  const typeScale = useCustomizerStore((state) => state.typeScale);
  const setTypeScale = useCustomizerStore((state) => state.setTypeScale);
  const motionSpeed = useCustomizerStore((state) => state.motionSpeed);
  const setMotionSpeed = useCustomizerStore((state) => state.setMotionSpeed);
  const transitions = useCustomizerStore((state) => state.transitions);
  const setTransitions = useCustomizerStore((state) => state.setTransitions);

  return (
    <CollapsibleSection title="Typographie & mouvement" defaultOpen={false}>
      <FieldRow label="Police">
        <select
          className={styles.select}
          value={fontSans}
          onChange={(event) =>
            patchTokens({ typography: { fontSans: event.currentTarget.value } })
          }
          aria-label="Police de l’interface"
        >
          {FONT_OPTIONS.map((option) => (
            <option key={option.label} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </FieldRow>

      <FieldRow label="Échelle de taille" hint={`${Math.round(typeScale * 100)} %`}>
        <Slider
          value={typeScale}
          min={0.85}
          max={1.25}
          step={0.01}
          onChange={setTypeScale}
          ariaLabel="Échelle de la typographie"
        />
      </FieldRow>

      <FieldRow label="Vitesse du mouvement" hint={`${motionSpeed.toFixed(2)}×`}>
        <Slider
          value={motionSpeed}
          min={0.5}
          max={1.6}
          step={0.05}
          onChange={setMotionSpeed}
          ariaLabel="Vitesse des animations"
        />
      </FieldRow>

      <FieldRow label="Fondus & transitions">
        <Switch
          checked={transitions}
          onChange={setTransitions}
          label="Activer les fondus et transitions"
        />
      </FieldRow>
    </CollapsibleSection>
  );
}

/* -------------------------------------------------------------------------- */
/* Background & effects                                                       */
/* -------------------------------------------------------------------------- */

const BACKGROUND_OPTIONS: readonly SegmentedOption<BackgroundStyle>[] = [
  { value: 'solid', label: 'Uni' },
  { value: 'gradient', label: 'Dégradé' },
  { value: 'mesh', label: 'Mesh' },
  { value: 'noise', label: 'Grain' },
];

/** Ambient backdrop, accent bloom and frosted-glass backdrop. */
export function BackgroundSection() {
  const effects = useCustomizerStore((state) => state.effects);
  const setEffect = useCustomizerStore((state) => state.setEffect);

  return (
    <CollapsibleSection title="Fond & effets" defaultOpen={false}>
      <FieldRow label="Ambiance">
        <SegmentedControl
          value={effects.background}
          options={BACKGROUND_OPTIONS}
          onChange={(value) => setEffect('background', value)}
          ariaLabel="Style de fond"
          columns={4}
        />
      </FieldRow>

      <FieldRow label="Intensité du halo" hint={`${effects.gradientIntensity} %`}>
        <Slider
          value={effects.gradientIntensity}
          min={0}
          max={100}
          onChange={(value) => setEffect('gradientIntensity', value)}
          ariaLabel="Intensité du fond"
        />
      </FieldRow>

      <FieldRow label="Opacité du grain" hint={`${effects.noiseOpacity} %`}>
        <Slider
          value={effects.noiseOpacity}
          min={0}
          max={60}
          onChange={(value) => setEffect('noiseOpacity', value)}
          ariaLabel="Opacité du grain"
        />
      </FieldRow>

      <FieldRow label="Lueur d’accent">
        <Switch
          checked={effects.glow}
          onChange={(value) => setEffect('glow', value)}
          label="Activer la lueur d’accent"
        />
      </FieldRow>

      <FieldRow label="Flou du backdrop">
        <Switch
          checked={effects.backdropBlur}
          onChange={(value) => setEffect('backdropBlur', value)}
          label="Activer le flou du backdrop"
        />
      </FieldRow>
    </CollapsibleSection>
  );
}
