import { cn } from '../../lib/utils';
import styles from './Equalizer.module.css';

export interface EqualizerProps {
  /** When false the bars freeze (used for the active-but-paused row). */
  playing?: boolean;
  className?: string;
}

/** Three animated bars marking the currently playing track. */
export function Equalizer({ playing = true, className }: EqualizerProps) {
  return (
    <span
      className={cn(styles.root, playing ? styles.playing : '', className)}
      aria-hidden
    >
      <span className={styles.bar} />
      <span className={styles.bar} />
      <span className={styles.bar} />
    </span>
  );
}
