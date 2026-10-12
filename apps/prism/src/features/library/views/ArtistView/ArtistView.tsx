import { Heart, Link2, Play, Shuffle } from 'lucide-react';
import { useState } from 'react';

import { Button, IconButton } from '@/components/ui';
import { useArtist, useArtistTopTracks, usePlaybackControls } from '@/hooks';
import { SpotifyApiError } from '@/lib/spotify/client';

import { cn, fireAndForget } from '../../lib/utils';
import { useArtworkGradient } from '../../lib/artwork';
import { AuthGate } from '../../components/AuthGate/AuthGate';
import { EmptyState } from '../../components/EmptyState/EmptyState';
import { ErrorState } from '../../components/ErrorState/ErrorState';
import { HeroHeader } from '../../components/HeroHeader/HeroHeader';
import { Menu, type MenuItem } from '../../components/Menu/Menu';
import { SectionHeader } from '../../components/SectionHeader/SectionHeader';
import { TrackList } from '../../components/TrackList/TrackList';
import { TrackListSkeleton } from '../../components/TrackListSkeleton/TrackListSkeleton';
import styles from './ArtistView.module.css';

export interface ArtistViewProps {
  id: string;
  className?: string;
}

/** Artist detail: round hero and the artist's most popular tracks. */
export function ArtistView({ id, className }: ArtistViewProps) {
  return (
    <AuthGate>
      <ArtistContent id={id} className={className} />
    </AuthGate>
  );
}

function ArtistContent({ id, className }: ArtistViewProps) {
  const artistQuery = useArtist(id);
  const topTracksQuery = useArtistTopTracks(id);
  const controls = usePlaybackControls();
  const [following, setFollowing] = useState(false);

  const artist = artistQuery.data;
  const topTracks = topTracksQuery.data?.tracks ?? [];
  const palette = useArtworkGradient(artist?.images?.[0]?.url);

  const status = artistQuery.error instanceof SpotifyApiError ? artistQuery.error.status : undefined;

  if (artistQuery.isError) {
    return (
      <div className={cn(styles.root, className)}>
        <ErrorState
          status={status}
          onRetry={() => {
            void artistQuery.refetch();
          }}
        />
      </div>
    );
  }

  if (artistQuery.isLoading || !artist) {
    return (
      <div className={cn(styles.root, className)}>
        <div className={styles.heroSkeleton} aria-hidden />
        <TrackListSkeleton rows={6} />
      </div>
    );
  }

  const genres = artist.genres ?? [];

  const menuItems: MenuItem[] = [
    {
      id: 'copy',
      label: "Copier le lien de l'artiste",
      icon: <Link2 />,
      onSelect: () => {
        const url = artist.external_urls?.spotify;
        if (url) fireAndForget(navigator.clipboard.writeText(url));
      },
      disabled: !artist.external_urls?.spotify,
    },
  ];

  return (
    <div className={cn(styles.root, className)}>
      <HeroHeader
        eyebrow="Artiste"
        title={artist.name}
        imageUrl={artist.images?.[0]?.url ?? null}
        round
        palette={palette}
        meta={
          genres.length > 0 ? (
            <span className={styles.genres}>{genres.slice(0, 3).join(' · ')}</span>
          ) : undefined
        }
        actions={
          <>
            <Button
              size="lg"
              iconLeft={<Play fill="currentColor" />}
              disabled={topTracks.length === 0}
              onClick={() =>
                fireAndForget(controls.play({ uris: topTracks.map((track) => track.uri) }))
              }
            >
              Lecture
            </Button>
            <IconButton
              label="Lecture aléatoire"
              size="lg"
              disabled={topTracks.length === 0}
              onClick={() =>
                fireAndForget(
                  controls
                    .setShuffle(true)
                    .then(() =>
                      controls.play({ uris: topTracks.map((track) => track.uri) }),
                    ),
                )
              }
            >
              <Shuffle />
            </IconButton>
            <IconButton
              label={following ? "Ne plus suivre l'artiste" : "Suivre l'artiste"}
              size="lg"
              className={following ? styles.following : ''}
              onClick={() => setFollowing((value) => !value)}
            >
              <Heart fill={following ? 'currentColor' : 'none'} />
            </IconButton>
            <Menu label="Options de l'artiste" items={menuItems} />
          </>
        }
      />

      <section aria-label="Titres populaires">
        <SectionHeader title="Titres populaires" />
        {topTracksQuery.isLoading ? (
          <TrackListSkeleton rows={6} />
        ) : topTracks.length > 0 ? (
          <TrackList tracks={topTracks} showAlbum />
        ) : (
          <EmptyState
            title="Aucun titre populaire"
            description="Aucun titre populaire n'est disponible pour cet artiste."
          />
        )}
      </section>
    </div>
  );
}
