import { Skeleton } from '@/components/ui';

import { cn } from '../../lib/utils';
import styles from './GridSkeleton.module.css';

export interface GridSkeletonProps {
  cards?: number;
  className?: string;
}

/** Placeholder for a responsive grid of `MediaCard`s. */
export function GridSkeleton({ cards = 8, className }: GridSkeletonProps) {
  return (
    <div className={cn(styles.root, className)} aria-hidden>
      {Array.from({ length: cards }).map((_, index) => (
        <div key={index} className={styles.card}>
          <Skeleton width="100%" height="0" className={styles.artwork} />
          <Skeleton width="70%" height="0.85rem" />
          <Skeleton width="45%" height="0.7rem" />
        </div>
      ))}
    </div>
  );
}
