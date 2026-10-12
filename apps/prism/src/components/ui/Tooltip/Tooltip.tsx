import { useEffect, useId, useRef, useState, type ReactNode } from 'react';

import styles from './Tooltip.module.css';

export type TooltipSide = 'top' | 'right' | 'bottom' | 'left';

export interface TooltipProps {
  content: ReactNode;
  children: ReactNode;
  side?: TooltipSide;
  /** Delay before showing, in ms. */
  delayMs?: number;
  className?: string;
}

/**
 * Lightweight, CSS-positioned tooltip. Shows on hover and keyboard focus, hides
 * on Escape. Purely presentational — the child keeps its own semantics.
 */
export function Tooltip({
  content,
  children,
  side = 'top',
  delayMs = 200,
  className,
}: TooltipProps) {
  const [visible, setVisible] = useState(false);
  const timerRef = useRef<number | undefined>(undefined);
  const id = useId();

  useEffect(() => () => window.clearTimeout(timerRef.current), []);

  const show = () => {
    window.clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(() => setVisible(true), delayMs);
  };

  const hide = () => {
    window.clearTimeout(timerRef.current);
    setVisible(false);
  };

  return (
    <span
      className={[styles.root, className ?? ''].filter(Boolean).join(' ')}
      onMouseEnter={show}
      onMouseLeave={hide}
      onFocus={show}
      onBlur={hide}
      onKeyDown={(event) => {
        if (event.key === 'Escape') hide();
      }}
    >
      {children}
      {visible ? (
        <span role="tooltip" id={id} className={[styles.tooltip, styles[side]].join(' ')}>
          {content}
        </span>
      ) : null}
    </span>
  );
}
