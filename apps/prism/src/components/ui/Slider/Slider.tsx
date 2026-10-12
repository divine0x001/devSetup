import type { CSSProperties } from 'react';

import styles from './Slider.module.css';

export interface SliderProps {
  value: number;
  min?: number;
  max?: number;
  step?: number;
  onChange(value: number): void;
  ariaLabel: string;
  className?: string;
}

/**
 * Accessible range slider built on a native `<input type="range">` (so keyboard
 * support is free) with a fully token-driven track and thumb. The filled
 * portion is painted via the `--slider-progress` custom property.
 */
export function Slider({
  value,
  min = 0,
  max = 100,
  step = 1,
  onChange,
  ariaLabel,
  className,
}: SliderProps) {
  const range = max - min || 1;
  const clamped = Math.min(max, Math.max(min, value));
  const percent = ((clamped - min) / range) * 100;

  const style = { '--slider-progress': `${percent}%` } as CSSProperties;

  return (
    <span className={[styles.root, className ?? ''].filter(Boolean).join(' ')} style={style}>
      <input
        type="range"
        className={styles.input}
        value={clamped}
        min={min}
        max={max}
        step={step}
        onChange={(event) => onChange(Number(event.currentTarget.value))}
        aria-label={ariaLabel}
      />
    </span>
  );
}
