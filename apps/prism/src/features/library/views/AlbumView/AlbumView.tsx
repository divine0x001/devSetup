import { Heart, Link2, Play, Shuffle } from 'lucide-react';
import { useState } from 'react';

import { Button, IconButton } from '@/components/ui';
import { useAlbum, usePlaybackControls } from '@/hooks';
import { SpotifyApiError } from '@/lib/spotify/client';

import { formatArtists, formatTotalDuration, formatYear } from '../../lib/format';
import { cn, fireAndForget } from '../../lib/utils';
import { useArtworkGradient } from '../../lib/artwork';
import { AuthGate } from '../../components/AuthGate/AuthGate';
import { ErrorState } from '../../components/ErrorState/ErrorState';
import { HeroHeader } from '../../components/HeroHeader/HeroHeader';
import { Menu, type MenuItem } from '../../components/Menu/Menu';
import { TrackList } from '../../components/TrackList/TrackList';
import { TrackListSkeleton } from '../../components/TrackListSkeleton/TrackListSkeleton';
import styles from './AlbumView.module.css';

export interface AlbumViewProps {
  id: string;
  className?: string;
}

const ALBUM_TYPE_LABEL: Record<string, string> = {
  album: 'Album',
  single: 'Single',
  compilation: 'Compilation',
};

/** Album detail: artwork hero, actions and the track list. */
export function AlbumView({ id, className }: AlbumViewProps) {
  return (
    <AuthGate>
      <AlbumContent id={id} className={className} />
    </AuthGate>
  );
}

function AlbumContent({ id, className }: AlbumViewProps) {
  const albumQuery = useAlbum(id);
  const controls = usePlaybackControls();
  const [saved, setSaved] = useState(false);

  const album = albumQuery.data;
  const tracks = album?.tracks?.items ?? [];
  const palette = useArtworkGradient(album?.images?.[0]?.url);

  const status = albumQuery.error instanceof SpotifyApiError ? albumQuery.error.status : undefined;

  if (albumQuery.isError) {
    return (
      <div className={cn(styles.root, className)}>
        <ErrorState
          status={status}
          onRetry={() => {
            void albumQuery.refetch();
          }}
        />
      </div>
    );
  }

  if (albumQuery.isLoading || !album) {
    return (
      <div className={cn(styles.root, className)}>
        <div className={styles.heroSkeleton} aria-hidden />
        <TrackListSkeleton rows={8} />
      </div>
    );
  }

  const totalMs = tracks.reduce((sum, track) => sum + track.duration_ms, 0);

  const menuItems: MenuItem[] = [
    {
      id: 'copy',
      label: "Copier le lien de l'album",
      icon: <Link2 />,
      onSelect: () => {
        const url = album.external_urls?.spotify;
        if (url) fireAndForget(navigator.clipboard.writeText(url));
      },
      disabled: !album.external_urls?.spotify,
    },
  ];

  return (
    <div className={cn(styles.root, className)}>
      <HeroHeader
        eyebrow={ALBUM_TYPE_LABEL[album.album_type] ?? 'Album'}
        title={album.name}
        imageUrl={album.images?.[0]?.url ?? null}
        palette={palette}
        meta={
          <>
            <span className={styles.artist}>{formatArtists(album.artists)}</span>
            <span aria-hidden>·</span>
            <span>{formatYear(album.release_date)}</span>
            <span aria-hidden>·</span>
            <span>
              {album.total_tracks} titre{album.total_tracks > 1 ? 's' : ''}
            </span>
            {totalMs > 0 ? (
              <>
                <span aria-hidden>·</span>
                <span>{formatTotalDuration(totalMs)}</span>
              </>
            ) : null}
          </>
        }
        actions={
          <>
            <Button
              size="lg"
              iconLeft={<Play fill="currentColor" />}
              disabled={tracks.length === 0}
              onClick={() => fireAndForget(controls.play({ context_uri: album.uri }))}
            >
              Lecture
            </Button>
            <IconButton
              label="Lecture aléatoire"
              size="lg"
              disabled={tracks.length === 0}
              onClick={() =>
                fireAndForget(
                  controls.setShuffle(true).then(() => controls.play({ context_uri: album.uri })),
                )
              }
            >
              <Shuffle />
            </IconButton>
            <IconButton
              label={saved ? "Retirer des albums enregistrés" : "Enregistrer l'album"}
              size="lg"
              className={saved ? styles.saved : ''}
              onClick={() => setSaved((value) => !value)}
            >
              <Heart fill={saved ? 'currentColor' : 'none'} />
            </IconButton>
            <Menu label="Options de l'album" items={menuItems} />
          </>
        }
      />

      {albumQuery.isLoading ? (
        <TrackListSkeleton rows={8} />
      ) : (
        <TrackList
          tracks={tracks}
          contextUri={album.uri}
          showAlbum={false}
          emptyState={
            <ErrorState
              title="Album vide"
              description="Cet album ne contient aucun titre pour le moment."
            />
          }
        />
      )}
    </div>
  );
}
