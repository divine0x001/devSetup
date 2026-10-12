import type { CSSProperties } from 'react';

import styles from './Avatar.module.css';

export interface AvatarProps {
  src?: string | null;
  alt: string;
  /** Pixel diameter. Defaults to 40. */
  size?: number;
  className?: string;
}

/**
 * Circular image with a graceful initials fallback when no source is available.
 */
export function Avatar({ src, alt, size = 40, className }: AvatarProps) {
  const classes = [styles.root, className ?? ''].filter(Boolean).join(' ');
  const style: CSSProperties = { width: size, height: size, fontSize: Math.round(size * 0.4) };

  if (src) {
    return (
      <img
        className={classes}
        style={style}
        src={src}
        alt={alt}
        loading="lazy"
        draggable={false}
      />
    );
  }

  const initial = alt.trim().charAt(0).toUpperCase() || '?';

  return (
    <span className={classes} style={style} role="img" aria-label={alt}>
      <span className={styles.initial} aria-hidden>
        {initial}
      </span>
    </span>
  );
}
