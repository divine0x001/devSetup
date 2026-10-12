import { useEffect, useRef, useState, type CSSProperties, type ReactNode } from 'react';

import styles from './Marquee.module.css';

export interface MarqueeProps {
  children: ReactNode;
  /** Scroll speed in pixels per second. */
  speed?: number;
  className?: string;
}

/**
 * Single-line text that scrolls horizontally on hover — but only when the
 * content actually overflows its container (measured with a ResizeObserver).
 */
export function Marquee({ children, speed = 40, className }: MarqueeProps) {
  const containerRef = useRef<HTMLSpanElement>(null);
  const itemRef = useRef<HTMLSpanElement>(null);
  const [overflowing, setOverflowing] = useState(false);
  const [duration, setDuration] = useState(12);

  useEffect(() => {
    const measure = () => {
      const container = containerRef.current;
      const item = itemRef.current;
      if (!container || !item) return;
      const width = item.scrollWidth;
      setOverflowing(width > container.clientWidth + 1);
      setDuration(Math.max(4, width / speed));
    };

    measure();
    const observer = new ResizeObserver(measure);
    if (containerRef.current) observer.observe(containerRef.current);
    if (itemRef.current) observer.observe(itemRef.current);
    return () => observer.disconnect();
  }, [children, speed]);

  const style = { '--marquee-duration': `${duration}s` } as CSSProperties;
  const classes = [styles.root, overflowing ? styles.overflowing : '', className ?? '']
    .filter(Boolean)
    .join(' ');

  return (
    <span ref={containerRef} className={classes} style={style}>
      <span className={styles.track}>
        <span ref={itemRef} className={styles.item}>
          {children}
        </span>
        <span className={styles.item} aria-hidden>
          {children}
        </span>
      </span>
    </span>
  );
}
