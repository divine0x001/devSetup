import { Skeleton } from '@/components/ui';

import { cn } from '../../lib/utils';
import styles from './ShelfSkeleton.module.css';

export interface ShelfSkeletonProps {
  cards?: number;
  className?: string;
}

/** Placeholder for a `Shelf` (header + horizontal cards). */
export function ShelfSkeleton({ cards = 6, className }: ShelfSkeletonProps) {
  return (
    <div className={cn(styles.root, className)} aria-hidden>
      <Skeleton width="10rem" height="1.4rem" />
      <div className={styles.track}>
        {Array.from({ length: cards }).map((_, index) => (
          <div key={index} className={styles.card}>
            <Skeleton width="100%" height="0" className={styles.artwork} />
            <Skeleton width="70%" height="0.85rem" />
            <Skeleton width="45%" height="0.7rem" />
          </div>
        ))}
      </div>
    </div>
  );
}
