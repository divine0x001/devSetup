import type { CSSProperties, ReactNode } from 'react';

import { cn } from '../../lib/utils';
import type { ArtworkPalette } from '../../lib/artwork';
import styles from './HeroHeader.module.css';

export interface HeroHeaderProps {
  /** Small label above the title (e.g. "Playlist", "Album"). */
  eyebrow?: string;
  title: string;
  /** Long description (playlist description, album credits…). */
  subtitle?: string;
  /** Inline metadata row (artist · year · track count). */
  meta?: ReactNode;
  imageUrl?: string | null;
  /** Circular artwork (artists). */
  round?: boolean;
  /** Ambient palette sampled from the artwork. */
  palette?: ArtworkPalette;
  /** Primary actions (play, like, menu…). */
  actions?: ReactNode;
  className?: string;
}

/** Large, artwork-tinted header used by playlist / album / artist views. */
export function HeroHeader({
  eyebrow,
  title,
  subtitle,
  meta,
  imageUrl,
  round = false,
  palette,
  actions,
  className,
}: HeroHeaderProps) {
  const style: CSSProperties | undefined = palette
    ? ({
        '--hero-from': palette.from,
        '--hero-to': palette.to,
        '--hero-rgb': palette.rgb,
      } as CSSProperties)
    : undefined;

  return (
    <header className={cn(styles.root, className)} style={style}>
      <div className={styles.artworkWrap}>
        <span className={cn(styles.artwork, round ? styles.round : '')}>
          {imageUrl ? (
            <img className={styles.image} src={imageUrl} alt="" draggable={false} />
          ) : (
            <span className={styles.placeholder} aria-hidden />
          )}
        </span>
      </div>

      <div className={styles.content}>
        {eyebrow ? <span className={styles.eyebrow}>{eyebrow}</span> : null}
        <h1 className={styles.title}>{title}</h1>
        {subtitle ? <p className={styles.subtitle}>{subtitle}</p> : null}
        {meta ? <div className={styles.meta}>{meta}</div> : null}
        {actions ? <div className={styles.actions}>{actions}</div> : null}
      </div>
    </header>
  );
}
