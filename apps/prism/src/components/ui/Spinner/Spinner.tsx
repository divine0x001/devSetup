import styles from './Spinner.module.css';

export type SpinnerSize = 'sm' | 'md' | 'lg';

export interface SpinnerProps {
  size?: SpinnerSize;
  /** Accessible label announced to screen readers. */
  label?: string;
  className?: string;
}

/**
 * Indeterminate loading indicator. Sizes derive from the spacing ramp so it
 * scales with the active density / radius theme.
 */
export function Spinner({ size = 'md', label, className }: SpinnerProps) {
  const classes = [styles.root, styles[size], className ?? ''].filter(Boolean).join(' ');

  return (
    <span className={classes} role="status" aria-live="polite">
      <span className={styles.ring} aria-hidden />
      <span className="prism-visually-hidden">{label ?? 'Chargement'}</span>
    </span>
  );
}
