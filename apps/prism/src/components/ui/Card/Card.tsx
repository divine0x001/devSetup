import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';

import styles from './Card.module.css';

export type CardElevation = 'flat' | 'raised' | 'floating';
export type CardPadding = 'none' | 'sm' | 'md' | 'lg';

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  elevation?: CardElevation;
  padding?: CardPadding;
  /** Adds hover elevation + pointer affordance (for clickable cards). */
  interactive?: boolean;
  children: ReactNode;
}

/** Generic surface container used across every feature view. */
export const Card = forwardRef<HTMLDivElement, CardProps>(function Card(
  { elevation = 'flat', padding = 'md', interactive = false, className, children, ...rest },
  ref,
) {
  const classes = [
    styles.root,
    styles[elevation],
    styles[`pad-${padding}`],
    interactive ? styles.interactive : '',
    className ?? '',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div ref={ref} className={classes} {...rest}>
      {children}
    </div>
  );
});
