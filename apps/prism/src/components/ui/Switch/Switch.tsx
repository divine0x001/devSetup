import type { ChangeEvent } from 'react';

import styles from './Switch.module.css';

export interface SwitchProps {
  checked: boolean;
  onChange(checked: boolean): void;
  label?: string;
  disabled?: boolean;
  id?: string;
  /** Render the label text next to the track (default hides it visually). */
  showLabel?: boolean;
  className?: string;
}

/** Accessible on/off toggle built on a native checkbox with `role="switch"`. */
export function Switch({
  checked,
  onChange,
  label,
  disabled = false,
  id,
  showLabel = false,
  className,
}: SwitchProps) {
  const classes = [styles.root, disabled ? styles.disabled : '', className ?? '']
    .filter(Boolean)
    .join(' ');

  return (
    <label className={classes}>
      <input
        id={id}
        type="checkbox"
        role="switch"
        className={styles.input}
        checked={checked}
        disabled={disabled}
        aria-label={showLabel ? undefined : label}
        onChange={(event: ChangeEvent<HTMLInputElement>) => onChange(event.currentTarget.checked)}
      />
      <span className={styles.track} aria-hidden>
        <span className={styles.thumb} />
      </span>
      {showLabel && label ? <span className={styles.label}>{label}</span> : null}
    </label>
  );
}
