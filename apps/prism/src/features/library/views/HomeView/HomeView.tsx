import { Play, Sparkles } from 'lucide-react';
import { Link } from 'react-router-dom';

import { buildPath } from '@/app/routes/routes';
import { useAuth, usePlaybackControls, usePlayer, useSavedTracks, useUserPlaylists } from '@/hooks';

import { formatArtists, getGreeting } from '../../lib/format';
import { fireAndForget } from '../../lib/utils';
import { useSavedAlbums } from '../../lib/queries';
import { AuthGate } from '../../components/AuthGate/AuthGate';
import { EmptyState } from '../../components/EmptyState/EmptyState';
import { MediaCard } from '../../components/MediaCard/MediaCard';
import { Reveal } from '../../components/Reveal/Reveal';
import { Shelf } from '../../components/Shelf/Shelf';
import { ShelfSkeleton } from '../../components/ShelfSkeleton/ShelfSkeleton';
import styles from './HomeView.module.css';

import type { SimplifiedPlaylist, SimplifiedTrack } from '@/lib/spotify/types';

const MIX_PATTERN = /mix|daily|weekly|discover|découverte|radar/i;

function firstImage(images: Array<{ url: string }> | undefined): string | null {
  return images?.[0]?.url ?? null;
}

/** Landing page: greeting, resume hero, quick picks and shelves. */
export function HomeView() {
  return (
    <AuthGate>
      <HomeContent />
    </AuthGate>
  );
}

function HomeContent() {
  const { profile } = useAuth();
  const { playback } = usePlayer();
  const controls = usePlaybackControls();

  const playlistsQuery = useUserPlaylists(50);
  const savedTracksQuery = useSavedTracks(50);
  const savedAlbumsQuery = useSavedAlbums(20);

  const playlists = playlistsQuery.data?.items ?? [];
  const savedTracks = savedTracksQuery.data?.items ?? [];
  const savedAlbums = savedAlbumsQuery.data?.items ?? [];

  const mixes = playlists.filter((playlist) => MIX_PATTERN.test(playlist.name));
  const quickPicks = playlists.slice(0, 6);
  const currentTrack = playback?.item ?? null;

  const playPlaylist = (playlist: SimplifiedPlaylist) =>
    fireAndForget(controls.play({ context_uri: playlist.uri }));

  const playTracks = (tracks: SimplifiedTrack[]) => {
    if (tracks.length === 0) return;
    fireAndForget(controls.play({ uris: tracks.map((track) => track.uri) }));
  };

  const greeting = getGreeting();
  const firstName = profile?.display_name?.split(' ')[0] ?? null;

  return (
    <div className={styles.root}>
      <Reveal>
        <h1 className={styles.greeting}>
          {greeting}
          {firstName ? `, ${firstName}` : ''}
        </h1>
      </Reveal>

      {currentTrack ? (
        <Reveal delay={0.05}>
          <section className={styles.resume} aria-label="Reprendre l’écoute">
            <div className={styles.resumeArtwork}>
              {firstImage(currentTrack.album.images) ? (
                <img src={firstImage(currentTrack.album.images) as string} alt="" />
              ) : (
                <span className={styles.resumePlaceholder} aria-hidden />
              )}
            </div>
            <div className={styles.resumeMeta}>
              <span className={styles.resumeEyebrow}>Reprendre l’écoute</span>
              <span className={styles.resumeTitle}>{currentTrack.name}</span>
              <span className={styles.resumeArtists}>{formatArtists(currentTrack.artists)}</span>
            </div>
            <button
              type="button"
              className={styles.resumePlay}
              onClick={() => fireAndForget(controls.play())}
              aria-label="Reprendre la lecture"
            >
              <Play fill="currentColor" aria-hidden />
            </button>
          </section>
        </Reveal>
      ) : null}

      {playlistsQuery.isLoading ? (
        <ShelfSkeleton />
      ) : quickPicks.length > 0 ? (
        <Reveal delay={0.1}>
          <section className={styles.quick} aria-label="Accès rapide">
            {quickPicks.map((playlist) => (
              <Link
                key={playlist.id}
                to={buildPath.playlist(playlist.id)}
                className={styles.quickCard}
              >
                <span className={styles.quickArtwork}>
                  {firstImage(playlist.images) ? (
                    <img src={firstImage(playlist.images) as string} alt="" loading="lazy" />
                  ) : (
                    <span className={styles.quickPlaceholder} aria-hidden />
                  )}
                </span>
                <span className={styles.quickName}>{playlist.name}</span>
              </Link>
            ))}
          </section>
        </Reveal>
      ) : null}

      {mixes.length > 0 ? (
        <Reveal delay={0.15}>
          <Shelf title="Vos mixes" subtitle="Faits pour vous">
            {mixes.map((playlist) => (
              <MediaCard
                key={playlist.id}
                title={playlist.name}
                subtitle={playlist.owner.display_name ?? 'Spotify'}
                imageUrl={firstImage(playlist.images)}
                to={buildPath.playlist(playlist.id)}
                onPlay={() => playPlaylist(playlist)}
              />
            ))}
          </Shelf>
        </Reveal>
      ) : null}

      {playlistsQuery.isLoading ? (
        <ShelfSkeleton />
      ) : playlists.length > 0 ? (
        <Reveal delay={0.2}>
          <Shelf title="Vos playlists">
            {playlists.map((playlist) => (
              <MediaCard
                key={playlist.id}
                title={playlist.name}
                subtitle={playlist.owner.display_name ?? 'Spotify'}
                imageUrl={firstImage(playlist.images)}
                to={buildPath.playlist(playlist.id)}
                onPlay={() => playPlaylist(playlist)}
              />
            ))}
          </Shelf>
        </Reveal>
      ) : null}

      {savedTracksQuery.isLoading ? (
        <ShelfSkeleton />
      ) : savedTracks.length > 0 ? (
        <Reveal delay={0.25}>
          <Shelf title="Titres likés">
            {savedTracks.map(({ track }) => (
              <MediaCard
                key={track.id}
                title={track.name}
                subtitle={formatArtists(track.artists)}
                imageUrl={firstImage(track.album.images)}
                onPlay={() => playTracks([track])}
              />
            ))}
          </Shelf>
        </Reveal>
      ) : null}

      {savedAlbumsQuery.isLoading ? (
        <ShelfSkeleton />
      ) : savedAlbums.length > 0 ? (
        <Reveal delay={0.3}>
          <Shelf title="Albums enregistrés">
            {savedAlbums.map(({ album }) => (
              <MediaCard
                key={album.id}
                title={album.name}
                subtitle={formatArtists(album.artists)}
                imageUrl={firstImage(album.images)}
                to={buildPath.album(album.id)}
                onPlay={() => fireAndForget(controls.play({ context_uri: album.uri }))}
              />
            ))}
          </Shelf>
        </Reveal>
      ) : null}

      {!playlistsQuery.isLoading &&
      !savedTracksQuery.isLoading &&
      !savedAlbumsQuery.isLoading &&
      playlists.length === 0 &&
      savedTracks.length === 0 &&
      savedAlbums.length === 0 ? (
        <EmptyState
          icon={<Sparkles />}
          title="Votre bibliothèque est vide"
          description="Enregistrez des titres, des albums ou créez une playlist pour les retrouver ici."
        />
      ) : null}
    </div>
  );
}
