import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react';

import { Spinner } from '../Spinner/Spinner';
import styles from './Button.module.css';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
export type ButtonSize = 'sm' | 'md' | 'lg';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  iconLeft?: ReactNode;
  iconRight?: ReactNode;
  fullWidth?: boolean;
}

/**
 * Primary action control. All visuals come from `--prism-*` tokens so it adapts
 * to every theme, color mode, density and radius scale.
 */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  {
    variant = 'primary',
    size = 'md',
    loading = false,
    iconLeft,
    iconRight,
    fullWidth = false,
    disabled,
    className,
    children,
    type = 'button',
    ...rest
  },
  ref,
) {
  const classes = [
    styles.root,
    styles[variant],
    styles[size],
    fullWidth ? styles.fullWidth : '',
    className ?? '',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <button
      ref={ref}
      type={type}
      className={classes}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
      {...rest}
    >
      {loading ? (
        <span className={styles.icon} aria-hidden>
          <Spinner size={size === 'lg' ? 'md' : 'sm'} />
        </span>
      ) : iconLeft ? (
        <span className={styles.icon} aria-hidden>
          {iconLeft}
        </span>
      ) : null}

      {children != null ? <span className={styles.label}>{children}</span> : null}

      {!loading && iconRight ? (
        <span className={styles.icon} aria-hidden>
          {iconRight}
        </span>
      ) : null}
    </button>
  );
});
