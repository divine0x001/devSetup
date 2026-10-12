import { Play } from 'lucide-react';
import type { ReactNode } from 'react';
import { Link } from 'react-router-dom';

import { cn } from '../../lib/utils';
import styles from './MediaCard.module.css';

export interface MediaCardProps {
  title: string;
  subtitle?: string;
  imageUrl?: string | null;
  /** Internal route — renders the card as a link. */
  to?: string;
  /** Click handler when the card is not a link. */
  onClick?: () => void;
  /** Circular artwork (artists). */
  round?: boolean;
  /** Small overlay badge (e.g. "Album", "Playlist"). */
  badge?: ReactNode;
  /** When provided, a play button is revealed on hover/focus. */
  onPlay?: () => void;
  className?: string;
}

/**
 * Artwork card used across shelves and grids. The play affordance is a sibling
 * of the navigation target so it never triggers navigation.
 */
export function MediaCard({
  title,
  subtitle,
  imageUrl,
  to,
  onClick,
  round = false,
  badge,
  onPlay,
  className,
}: MediaCardProps) {
  const artwork = (
    <span className={cn(styles.artwork, round ? styles.round : '')}>
      {imageUrl ? (
        <img className={styles.image} src={imageUrl} alt="" loading="lazy" draggable={false} />
      ) : (
        <span className={styles.placeholder} aria-hidden />
      )}
      {badge ? <span className={styles.badge}>{badge}</span> : null}
    </span>
  );

  const body = (
    <>
      {artwork}
      <span className={styles.meta}>
        <span className={styles.title}>{title}</span>
        {subtitle ? <span className={styles.subtitle}>{subtitle}</span> : null}
      </span>
    </>
  );

  return (
    <div className={cn(styles.root, className)}>
      {to ? (
        <Link to={to} className={styles.target} aria-label={title}>
          {body}
        </Link>
      ) : (
        <button type="button" className={styles.target} onClick={onClick}>
          {body}
        </button>
      )}

      {onPlay ? (
        <button
          type="button"
          className={styles.play}
          onClick={(event) => {
            event.stopPropagation();
            onPlay();
          }}
          aria-label={`Lire ${title}`}
        >
          <Play fill="currentColor" aria-hidden />
        </button>
      ) : null}
    </div>
  );
}
