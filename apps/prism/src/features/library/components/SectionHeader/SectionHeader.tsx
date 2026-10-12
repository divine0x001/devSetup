import { ChevronRight } from 'lucide-react';
import type { ReactNode } from 'react';

import { cn } from '../../lib/utils';
import styles from './SectionHeader.module.css';

export interface SectionHeaderProps {
  title: string;
  subtitle?: string;
  /** Right-aligned slot (e.g. a filter or "see all" button). */
  action?: ReactNode;
  /** Convenience "see all" link rendered before `action`. */
  onSeeAll?: () => void;
  seeAllLabel?: string;
  className?: string;
}

/** Title row used above shelves and grids. */
export function SectionHeader({
  title,
  subtitle,
  action,
  onSeeAll,
  seeAllLabel = 'Tout afficher',
  className,
}: SectionHeaderProps) {
  return (
    <header className={cn(styles.root, className)}>
      <div className={styles.heading}>
        <h2 className={styles.title}>{title}</h2>
        {subtitle ? <p className={styles.subtitle}>{subtitle}</p> : null}
      </div>
      <div className={styles.actions}>
        {onSeeAll ? (
          <button type="button" className={styles.seeAll} onClick={onSeeAll}>
            {seeAllLabel}
            <ChevronRight aria-hidden />
          </button>
        ) : null}
        {action}
      </div>
    </header>
  );
}
