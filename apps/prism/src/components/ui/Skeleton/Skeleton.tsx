import type { CSSProperties } from 'react';

import styles from './Skeleton.module.css';

export interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  /** Explicit radius token value, e.g. `var(--prism-radius-lg)`. */
  radius?: string;
  circle?: boolean;
  className?: string;
}

/** Shimmering placeholder for loading content. */
export function Skeleton({
  width = '100%',
  height = '1em',
  radius,
  circle = false,
  className,
}: SkeletonProps) {
  const style: CSSProperties = {
    width,
    height,
    borderRadius: circle ? 'var(--prism-radius-full)' : radius,
  };

  return (
    <span
      className={[styles.root, className ?? ''].filter(Boolean).join(' ')}
      style={style}
      aria-hidden
    />
  );
}
