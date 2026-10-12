import { Disc3, Heart, Link2, ListPlus, ListStart, Play, User } from 'lucide-react';
import { Link, useNavigate } from 'react-router-dom';

import { buildPath } from '@/app/routes/routes';
import { IconButton } from '@/components/ui';
import { useLibrary } from '@/hooks';
import { spotifyApi } from '@/lib/spotify/client';
import { useQueueStore } from '@/features/player/queueStore';

import { formatDuration } from '../../lib/format';
import { cn, fireAndForget } from '../../lib/utils';
import { Equalizer } from '../Equalizer/Equalizer';
import { Menu, type MenuItem } from '../Menu/Menu';
import styles from './TrackRow.module.css';

import type { SimplifiedTrack } from '@/lib/spotify/types';

export interface TrackRowProps {
  track: SimplifiedTrack;
  /** Zero-based position within the list. */
  index: number;
  /** Play handler; when omitted the row plays just this track. */
  onPlay?: () => void;
  showAlbum?: boolean;
  showArtists?: boolean;
  isActive?: boolean;
  isPlaying?: boolean;
  className?: string;
}

/** A single track line: index/play, artwork, title, album, like, duration, menu. */
export function TrackRow({
  track,
  index,
  onPlay,
  showAlbum = true,
  showArtists = true,
  isActive = false,
  isPlaying = false,
  className,
}: TrackRowProps) {
  const navigate = useNavigate();
  const { isTrackSaved, toggleSavedTrack } = useLibrary();
  const addToQueue = useQueueStore((state) => state.addToQueue);
  const addNext = useQueueStore((state) => state.addNext);

  const saved = isTrackSaved(track.id);
  const primaryArtist = track.artists[0];

  const handleToggleSave = () => {
    toggleSavedTrack(track.id);
    fireAndForget(
      saved ? spotifyApi.removeSavedTracks([track.id]) : spotifyApi.saveTracks([track.id]),
    );
  };

  const handleCopyLink = () => {
    const url = track.external_urls?.spotify;
    if (!url) return;
    fireAndForget(navigator.clipboard.writeText(url));
  };

  const menuItems: MenuItem[] = [
    {
      id: 'queue',
      label: "Ajouter à la file d'attente",
      icon: <ListPlus />,
      onSelect: () => addToQueue(track),
    },
    {
      id: 'next',
      label: 'Lire ensuite',
      icon: <ListStart />,
      onSelect: () => addNext(track),
    },
    {
      id: 'album',
      label: "Aller à l'album",
      icon: <Disc3 />,
      onSelect: () => navigate(buildPath.album(track.album.id)),
    },
    ...(primaryArtist
      ? [
          {
            id: 'artist',
            label: "Aller à l'artiste",
            icon: <User />,
            onSelect: () => navigate(buildPath.artist(primaryArtist.id)),
          } satisfies MenuItem,
        ]
      : []),
    {
      id: 'copy',
      label: 'Copier le lien',
      icon: <Link2 />,
      onSelect: handleCopyLink,
      disabled: !track.external_urls?.spotify,
    },
  ];

  return (
    <div
      className={cn(styles.row, isActive ? styles.active : '', className)}
      role="listitem"
      aria-current={isActive || undefined}
    >
      <div className={styles.indexCell}>
        {isActive ? (
          <Equalizer playing={isPlaying} className={styles.equalizer} />
        ) : (
          <span className={styles.index}>{index + 1}</span>
        )}
        <button
          type="button"
          className={styles.playButton}
          onClick={onPlay}
          aria-label={`Lire ${track.name}`}
        >
          <Play fill="currentColor" aria-hidden />
        </button>
      </div>

      <div className={styles.artworkCell}>
        {track.album.images?.[0]?.url ? (
          <img
            className={styles.artwork}
            src={track.album.images[0].url}
            alt=""
            loading="lazy"
            draggable={false}
          />
        ) : (
          <span className={styles.artworkPlaceholder} aria-hidden />
        )}
      </div>

      <div className={styles.titleCell}>
        <span className={cn(styles.name, isActive ? styles.nameActive : '')}>{track.name}</span>
        {showArtists && track.artists.length > 0 ? (
          <span className={styles.artists}>
            {track.artists.map((artist, artistIndex) => (
              <span key={artist.id}>
                {artistIndex > 0 ? ', ' : null}
                <Link to={buildPath.artist(artist.id)} className={styles.artistLink}>
                  {artist.name}
                </Link>
              </span>
            ))}
          </span>
        ) : null}
      </div>

      {showAlbum ? (
        <div className={styles.albumCell}>
          <Link to={buildPath.album(track.album.id)} className={styles.albumLink}>
            {track.album.name}
          </Link>
        </div>
      ) : null}

      <div className={styles.likeCell}>
        <IconButton
          label={saved ? 'Retirer des titres likés' : 'Ajouter aux titres likés'}
          size="sm"
          className={saved ? styles.likeActive : ''}
          onClick={handleToggleSave}
        >
          <Heart fill={saved ? 'currentColor' : 'none'} />
        </IconButton>
      </div>

      <div className={styles.durationCell}>{formatDuration(track.duration_ms)}</div>

      <div className={styles.menuCell}>
        <Menu label={`Options pour ${track.name}`} items={menuItems} />
      </div>
    </div>
  );
}
