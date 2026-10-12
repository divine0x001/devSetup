import type { ReactNode } from 'react';

import { cn } from '../../lib/utils';
import { SectionHeader } from '../SectionHeader/SectionHeader';
import styles from './Shelf.module.css';

export interface ShelfProps {
  title: string;
  subtitle?: string;
  onSeeAll?: () => void;
  children: ReactNode;
  className?: string;
}

/** Horizontally scrollable row of cards under a section header. */
export function Shelf({ title, subtitle, onSeeAll, children, className }: ShelfProps) {
  return (
    <section className={cn(styles.root, className)}>
      <SectionHeader title={title} subtitle={subtitle} onSeeAll={onSeeAll} />
      <div className={styles.track}>{children}</div>
    </section>
  );
}
