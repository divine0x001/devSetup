import { Skeleton } from '@/components/ui';

import { cn } from '../../lib/utils';
import styles from './TrackListSkeleton.module.css';

export interface TrackListSkeletonProps {
  rows?: number;
  className?: string;
}

/** Placeholder rows matching the `TrackList` layout. */
export function TrackListSkeleton({ rows = 8, className }: TrackListSkeletonProps) {
  return (
    <div className={cn(styles.root, className)} aria-hidden>
      {Array.from({ length: rows }).map((_, index) => (
        <div key={index} className={styles.row}>
          <Skeleton width="1.5rem" height="1rem" />
          <Skeleton width="2.5rem" height="2.5rem" radius="var(--prism-radius-sm)" />
          <div className={styles.title}>
            <Skeleton width="60%" height="0.85rem" />
            <Skeleton width="35%" height="0.7rem" />
          </div>
          <Skeleton width="70%" height="0.75rem" />
          <Skeleton width="2.5rem" height="0.75rem" />
        </div>
      ))}
    </div>
  );
}
