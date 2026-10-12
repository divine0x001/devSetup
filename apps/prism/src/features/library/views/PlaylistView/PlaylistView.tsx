import { Heart, Link2, Play, Shuffle } from 'lucide-react';
import { useState } from 'react';

import { Button, IconButton } from '@/components/ui';
import { usePlaybackControls, usePlaylist, usePlaylistTracks } from '@/hooks';
import { SpotifyApiError } from '@/lib/spotify/client';

import { formatCount, formatTotalDuration } from '../../lib/format';
import { cn, fireAndForget } from '../../lib/utils';
import { useArtworkGradient } from '../../lib/artwork';
import { AuthGate } from '../../components/AuthGate/AuthGate';
import { ErrorState } from '../../components/ErrorState/ErrorState';
import { HeroHeader } from '../../components/HeroHeader/HeroHeader';
import { Menu, type MenuItem } from '../../components/Menu/Menu';
import { TrackList } from '../../components/TrackList/TrackList';
import { TrackListSkeleton } from '../../components/TrackListSkeleton/TrackListSkeleton';
import styles from './PlaylistView.module.css';

export interface PlaylistViewProps {
  id: string;
  className?: string;
}

/** Playlist detail: artwork hero, actions and the full track list. */
export function PlaylistView({ id, className }: PlaylistViewProps) {
  return (
    <AuthGate>
      <PlaylistContent id={id} className={className} />
    </AuthGate>
  );
}

function PlaylistContent({ id, className }: PlaylistViewProps) {
  const playlistQuery = usePlaylist(id);
  const tracksQuery = usePlaylistTracks(id, 50);
  const controls = usePlaybackControls();
  const [following, setFollowing] = useState(false);

  const playlist = playlistQuery.data;
  const tracks = (tracksQuery.data?.items ?? []).map((item) => item.track);
  const palette = useArtworkGradient(playlist?.images?.[0]?.url);

  const status = playlistQuery.error instanceof SpotifyApiError ? playlistQuery.error.status : undefined;

  if (playlistQuery.isError) {
    return (
      <div className={cn(styles.root, className)}>
        <ErrorState
          status={status}
          onRetry={() => {
            void playlistQuery.refetch();
          }}
        />
      </div>
    );
  }

  if (playlistQuery.isLoading || !playlist) {
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
      label: 'Copier le lien de la playlist',
      icon: <Link2 />,
      onSelect: () => {
        const url = playlist.external_urls?.spotify;
        if (url) fireAndForget(navigator.clipboard.writeText(url));
      },
      disabled: !playlist.external_urls?.spotify,
    },
  ];

  return (
    <div className={cn(styles.root, className)}>
      <HeroHeader
        eyebrow="Playlist"
        title={playlist.name}
        subtitle={playlist.description ?? undefined}
        imageUrl={playlist.images?.[0]?.url ?? null}
        palette={palette}
        meta={
          <>
            <span className={styles.owner}>{playlist.owner.display_name ?? 'Spotify'}</span>
            <span aria-hidden>·</span>
            <span>{formatCount(playlist.tracks.total)} titres</span>
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
              onClick={() => fireAndForget(controls.play({ context_uri: playlist.uri }))}
            >
              Lecture
            </Button>
            <IconButton
              label="Lecture aléatoire"
              size="lg"
              disabled={tracks.length === 0}
              onClick={() =>
                fireAndForget(
                  controls.setShuffle(true).then(() => controls.play({ context_uri: playlist.uri })),
                )
              }
            >
              <Shuffle />
            </IconButton>
            <IconButton
              label={following ? 'Ne plus suivre' : 'Suivre la playlist'}
              size="lg"
              className={following ? styles.following : ''}
              onClick={() => setFollowing((value) => !value)}
            >
              <Heart fill={following ? 'currentColor' : 'none'} />
            </IconButton>
            <Menu label="Options de la playlist" items={menuItems} />
          </>
        }
      />

      {tracksQuery.isLoading ? (
        <TrackListSkeleton rows={8} />
      ) : (
        <TrackList
          tracks={tracks}
          contextUri={playlist.uri}
          showAlbum
          emptyState={
            <ErrorState
              title="Playlist vide"
              description="Cette playlist ne contient aucun titre pour le moment."
            />
          }
        />
      )}
    </div>
  );
}
