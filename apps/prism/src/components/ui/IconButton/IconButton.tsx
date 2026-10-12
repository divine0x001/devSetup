import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react';

import { Spinner } from '../Spinner/Spinner';
import type { ButtonSize, ButtonVariant } from '../Button/Button';
import styles from './IconButton.module.css';

export interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** Required accessible name (also used as the native tooltip). */
  label: string;
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  children: ReactNode;
}

/**
 * Square, icon-only button. `label` is mandatory to guarantee an accessible
 * name on every instance.
 */
export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(function IconButton(
  {
    label,
    variant = 'ghost',
    size = 'md',
    loading = false,
    disabled,
    className,
    children,
    type = 'button',
    ...rest
  },
  ref,
) {
  const classes = [styles.root, styles[variant], styles[size], className ?? '']
    .filter(Boolean)
    .join(' ');

  return (
    <button
      ref={ref}
      type={type}
      className={classes}
      aria-label={label}
      title={label}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
      {...rest}
    >
      {loading ? <Spinner size={size === 'lg' ? 'md' : 'sm'} /> : children}
    </button>
  );
});
