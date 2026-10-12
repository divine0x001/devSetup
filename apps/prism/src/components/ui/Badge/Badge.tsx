import type { HTMLAttributes } from 'react';

import styles from './Badge.module.css';

export type BadgeVariant = 'neutral' | 'accent' | 'success' | 'warning' | 'danger';

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant;
}

/** Small status / metadata pill. */
export function Badge({ variant = 'neutral', className, children, ...rest }: BadgeProps) {
  const classes = [styles.root, styles[variant], className ?? ''].filter(Boolean).join(' ');

  return (
    <span className={classes} {...rest}>
      {children}
    </span>
  );
}
