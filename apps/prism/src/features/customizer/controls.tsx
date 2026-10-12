import { ChevronDown } from 'lucide-react';
import { useId, useState, type CSSProperties, type ReactNode } from 'react';

import type { LayoutVariant } from '@/theme/types';

import styles from './Customizer.module.css';

/* -------------------------------------------------------------------------- */
/* Collapsible section                                                        */
/* -------------------------------------------------------------------------- */

export interface CollapsibleSectionProps {
  title: string;
  description?: string;
  defaultOpen?: boolean;
  children: ReactNode;
}

/** Accordion section with an accessible disclosure header. */
export function CollapsibleSection({
  title,
  description,
  defaultOpen = true,
  children,
}: CollapsibleSectionProps) {
  const [open, setOpen] = useState(defaultOpen);
  const panelId = useId();

  return (
    <section className={styles.section}>
      <h3 className={styles.sectionHeading}>
        <button
          type="button"
          className={styles.sectionToggle}
          aria-expanded={open}
          aria-controls={panelId}
          onClick={() => setOpen((value) => !value)}
        >
          <span className={styles.sectionTitle}>{title}</span>
          <ChevronDown
            className={styles.chevron}
            data-open={open || undefined}
            aria-hidden
          />
        </button>
      </h3>
      {description ? <p className={styles.sectionDescription}>{description}</p> : null}
      {open ? (
        <div id={panelId} className={styles.sectionPanel}>
          {children}
        </div>
      ) : null}
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/* Segmented control                                                          */
/* -------------------------------------------------------------------------- */

export interface SegmentedOption<T extends string> {
  value: T;
  label: ReactNode;
  icon?: ReactNode;
  disabled?: boolean;
}

export interface SegmentedControlProps<T extends string> {
  value: T;
  options: readonly SegmentedOption<T>[];
  onChange: (value: T) => void;
  ariaLabel: string;
  /** Force a grid with this many columns instead of a single row. */
  columns?: number;
}

/** Radio-group styled as a segmented control (keyboard + AT friendly). */
export function SegmentedControl<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
  columns,
}: SegmentedControlProps<T>) {
  const style: CSSProperties | undefined = columns
    ? { gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` }
    : undefined;

  return (
    <div
      className={styles.segmented}
      role="radiogroup"
      aria-label={ariaLabel}
      style={style}
    >
      {options.map((option) => {
        const active = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            role="radio"
            aria-checked={active}
            disabled={option.disabled}
            className={styles.segment}
            data-active={active || undefined}
            onClick={() => onChange(option.value)}
          >
            {option.icon ? (
              <span className={styles.segmentIcon} aria-hidden>
                {option.icon}
              </span>
            ) : null}
            <span className={styles.segmentLabel}>{option.label}</span>
          </button>
        );
      })}
    </div>
  );
}

/* -------------------------------------------------------------------------- */
/* Color field                                                                */
/* -------------------------------------------------------------------------- */

const HEX_SHORT = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/i;
const HEX_LONG = /^#([0-9a-f]{6})$/i;

/** Normalize any string into a `#rrggbb` value a native color input accepts. */
export function toHex(value: string): string {
  const short = HEX_SHORT.exec(value.trim());
  if (short) return `#${short[1]}${short[1]}${short[2]}${short[2]}${short[3]}${short[3]}`.toLowerCase();
  const long = HEX_LONG.exec(value.trim());
  if (long) return `#${long[1].toLowerCase()}`;
  return '#000000';
}

export interface ColorFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
}

/** Native color picker + editable value, fully keyboard accessible. */
export function ColorField({ label, value, onChange }: ColorFieldProps) {
  const inputId = useId();

  return (
    <div className={styles.colorField}>
      <input
        id={inputId}
        type="color"
        className={styles.colorInput}
        value={toHex(value)}
        onChange={(event) => onChange(event.currentTarget.value)}
        aria-label={`${label} — sélecteur de couleur`}
      />
      <label className={styles.colorText} htmlFor={inputId}>
        <span className={styles.colorLabel}>{label}</span>
        <input
          type="text"
          className={styles.colorHex}
          value={value}
          spellCheck={false}
          autoComplete="off"
          onChange={(event) => onChange(event.currentTarget.value)}
          aria-label={`${label} — valeur de la couleur`}
        />
      </label>
    </div>
  );
}

/* -------------------------------------------------------------------------- */
/* Layout preview                                                             */
/* -------------------------------------------------------------------------- */

export interface LayoutPreviewProps {
  layout: LayoutVariant;
}

/** Tiny schematic mock of a shell layout used in the layout picker. */
export function LayoutPreview({ layout }: LayoutPreviewProps) {
  return (
    <span className={styles.layoutPreview} data-layout={layout} aria-hidden>
      <span className={styles.lpTop} />
      <span className={styles.lpSide} />
      <span className={styles.lpMain} />
      <span className={styles.lpPlayer} />
    </span>
  );
}

/* -------------------------------------------------------------------------- */
/* Field row                                                                  */
/* -------------------------------------------------------------------------- */

export interface FieldRowProps {
  label: string;
  hint?: string;
  children: ReactNode;
}

/** Label + control row used throughout the customizer. */
export function FieldRow({ label, hint, children }: FieldRowProps) {
  return (
    <div className={styles.fieldRow}>
      <div className={styles.fieldLabel}>
        <span className={styles.fieldLabelText}>{label}</span>
        {hint ? <span className={styles.fieldHint}>{hint}</span> : null}
      </div>
      <div className={styles.fieldControl}>{children}</div>
    </div>
  );
}
