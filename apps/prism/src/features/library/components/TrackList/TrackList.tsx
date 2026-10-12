import { Clock3 } from 'lucide-react';
import type { CSSProperties, ReactNode } from 'react';

import { usePlaybackControls } from '@/hooks';
import { usePlayerStore } from '@/store/playerStore';
import { useQueueStore } from '@/features/player/queueStore';

import { cn, fireAndForget } from '../../lib/utils';
import { EmptyState } from '../EmptyState/EmptyState';
import { TrackRow } from '../TrackRow/TrackRow';
import styles from './TrackList.module.css';

import type { SimplifiedTrack } from '@/lib/spotify/types';

export interface TrackListProps {
  tracks: SimplifiedTrack[];
  /** Album/playlist URI — enables context playback from any row. */
  contextUri?: string;
  showAlbum?: boolean;
  showHeader?: boolean;
  /** Overrides the default play behaviour (context playback). */
  onPlayTrack?: (index: number) => void;
  emptyState?: ReactNode;
  className?: string;
}

const GRID_WITH_ALBUM = '2rem 2.5rem minmax(0, 1.4fr) minmax(0, 1fr) 3rem 4.5rem 2.5rem';
const GRID_NO_ALBUM = '2rem 2.5rem minmax(0, 1fr) 3rem 4.5rem 2.5rem';

/** Column-aligned list of tracks with an optional header row. */
export function TrackList({
  tracks,
  contextUri,
  showAlbum = true,
  showHeader = true,
  onPlayTrack,
  emptyState,
  className,
}: TrackListProps) {
  const controls = usePlaybackControls();
  const setQueue = useQueueStore((state) => state.setQueue);
  const currentTrackId = usePlayerStore((state) => state.playback?.item?.id ?? null);
  const isPlaying = usePlayerStore((state) => state.isPlaying);

  const gridStyle = {
    '--track-grid': showAlbum ? GRID_WITH_ALBUM : GRID_NO_ALBUM,
    '--track-grid-compact': GRID_NO_ALBUM,
  } as CSSProperties;

  const handlePlay = (index: number) => {
    if (onPlayTrack) {
      onPlayTrack(index);
      return;
    }
    setQueue(tracks, index);
    if (contextUri) {
      fireAndForget(controls.play({ context_uri: contextUri, offset: { position: index } }));
    } else {
      fireAndForget(
        controls.play({ uris: tracks.map((track) => track.uri), offset: { position: index } }),
      );
    }
  };

  if (tracks.length === 0) {
    return (
      <div className={cn(styles.root, className)} style={gridStyle}>
        {emptyState ?? (
          <EmptyState title="Aucun titre" description="Cette liste ne contient aucun titre." />
        )}
      </div>
    );
  }

  return (
    <div className={cn(styles.root, className)} style={gridStyle}>
      {showHeader ? (
        <div className={styles.header}>
          <span className={styles.headerIndex}>#</span>
          <span aria-hidden />
          <span className={styles.headerTitle}>Titre</span>
          {showAlbum ? <span className={styles.headerAlbum}>Album</span> : null}
          <span aria-hidden />
          <span className={styles.headerDuration}>
            <Clock3 aria-hidden />
          </span>
          <span aria-hidden />
        </div>
      ) : null}

      <div className={styles.rows} role="list" aria-label="Titres">
        {tracks.map((track, index) => (
          <TrackRow
            key={`${track.id}-${index}`}
            track={track}
            index={index}
            showAlbum={showAlbum}
            isActive={track.id === currentTrackId}
            isPlaying={isPlaying}
            onPlay={() => handlePlay(index)}
          />
        ))}
      </div>
    </div>
  );
}
