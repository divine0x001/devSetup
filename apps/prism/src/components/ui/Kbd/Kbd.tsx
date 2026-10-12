import type { HTMLAttributes, ReactNode } from 'react';

import styles from './Kbd.module.css';

export interface KbdProps extends HTMLAttributes<HTMLElement> {
  children: ReactNode;
}

/** Styled keyboard-key hint. */
export function Kbd({ className, children, ...rest }: KbdProps) {
  return (
    <kbd className={[styles.root, className ?? ''].filter(Boolean).join(' ')} {...rest}>
      {children}
    </kbd>
  );
}
