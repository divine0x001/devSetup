import { ListMusic, Play, Trash2, X } from 'lucide-react';

import { IconButton } from '@/components/ui';
import { usePlaybackControls, usePlayer } from '@/hooks';
import { usePlayerStore } from '@/store/playerStore';
import { cn, fireAndForget } from '@/features/library/lib/utils';
import { formatArtists } from '@/features/library/lib/format';
import { EmptyState } from '@/features/library/components/EmptyState/EmptyState';
import { Equalizer } from '@/features/library/components/Equalizer/Equalizer';

import { useQueueStore } from '../queueStore';
import styles from './QueueView.module.css';

export interface QueueViewProps {
  className?: string;
  onClose?: () => void;
}

/** The local play queue: current track + up-next list. */
export function QueueView({ className, onClose }: QueueViewProps) {
  const { playback, isPlaying } = usePlayer();
  const controls = usePlaybackControls();
  const queue = useQueueStore((state) => state.queue);
  const currentIndex = useQueueStore((state) => state.currentIndex);
  const removeFromQueue = useQueueStore((state) => state.removeFromQueue);
  const setCurrentIndex = useQueueStore((state) => state.setCurrentIndex);
  const clearQueue = useQueueStore((state) => state.clearQueue);
  const currentTrackId = usePlayerStore((state) => state.playback?.item?.id ?? null);

  const currentTrack = playback?.item ?? null;
  const upNext = queue
    .map((track, index) => ({ track, index }))
    .filter(({ index }) => index > currentIndex);

  const playAt = (index: number) => {
    setCurrentIndex(index);
    fireAndForget(
      controls.play({ uris: queue.map((track) => track.uri), offset: { position: index } }),
    );
  };

  return (
    <div className={cn(styles.root, className)}>
      <header className={styles.header}>
        <h2 className={styles.title}>File d’attente</h2>
        <div className={styles.headerActions}>
          {queue.length > 0 ? (
            <button type="button" className={styles.clear} onClick={clearQueue}>
              Vider
            </button>
          ) : null}
          {onClose ? (
            <IconButton label="Fermer la file d’attente" size="sm" onClick={onClose}>
              <X />
            </IconButton>
          ) : null}
        </div>
      </header>

      {currentTrack ? (
        <section className={styles.section} aria-label="En lecture">
          <h3 className={styles.sectionTitle}>En lecture</h3>
          <div className={cn(styles.item, styles.current)}>
            <span className={styles.artwork}>
              {currentTrack.album.images?.[0]?.url ? (
                <img src={currentTrack.album.images[0].url} alt="" loading="lazy" />
              ) : (
                <span className={styles.placeholder} aria-hidden />
              )}
            </span>
            <span className={styles.meta}>
              <span className={styles.name}>{currentTrack.name}</span>
              <span className={styles.artists}>{formatArtists(currentTrack.artists)}</span>
            </span>
            <Equalizer playing={isPlaying} className={styles.equalizer} />
          </div>
        </section>
      ) : null}

      <section className={styles.section} aria-label="À suivre">
        <h3 className={styles.sectionTitle}>À suivre</h3>
        {upNext.length === 0 ? (
          <EmptyState
            icon={<ListMusic />}
            title="File d’attente vide"
            description="Ajoutez des titres depuis la bibliothèque pour les voir ici."
          />
        ) : (
          <ul className={styles.list}>
            {upNext.map(({ track, index }) => (
              <li key={`${track.id}-${index}`} className={styles.item}>
                <button
                  type="button"
                  className={styles.playTarget}
                  onClick={() => playAt(index)}
                  aria-label={`Lire ${track.name}`}
                >
                  <span className={styles.artwork}>
                    {track.album.images?.[0]?.url ? (
                      <img src={track.album.images[0].url} alt="" loading="lazy" />
                    ) : (
                      <span className={styles.placeholder} aria-hidden />
                    )}
                    <span className={styles.playOverlay} aria-hidden>
                      <Play fill="currentColor" />
                    </span>
                  </span>
                  <span className={styles.meta}>
                    <span
                      className={cn(styles.name, track.id === currentTrackId ? styles.nameActive : '')}
                    >
                      {track.name}
                    </span>
                    <span className={styles.artists}>{formatArtists(track.artists)}</span>
                  </span>
                </button>
                <IconButton
                  label={`Retirer ${track.name} de la file`}
                  size="sm"
                  onClick={() => removeFromQueue(index)}
                >
                  <Trash2 />
                </IconButton>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
