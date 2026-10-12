import { Disc3, Heart, ListMusic, Mic2 } from 'lucide-react';
import { useState, type ReactNode } from 'react';

import { buildPath } from '@/app/routes/routes';
import { useSavedTracks, useUserPlaylists } from '@/hooks';

import { formatArtists } from '../../lib/format';
import { cn } from '../../lib/utils';
import { useSavedAlbums } from '../../lib/queries';
import { AuthGate } from '../../components/AuthGate/AuthGate';
import { EmptyState } from '../../components/EmptyState/EmptyState';
import { GridSkeleton } from '../../components/GridSkeleton/GridSkeleton';
import { MediaCard } from '../../components/MediaCard/MediaCard';
import { TrackList } from '../../components/TrackList/TrackList';
import { TrackListSkeleton } from '../../components/TrackListSkeleton/TrackListSkeleton';
import styles from './LibraryView.module.css';

import type { SimplifiedTrack } from '@/lib/spotify/types';

type LibraryTab = 'playlists' | 'tracks' | 'albums' | 'artists';

export type { LibraryTab };

export interface LibraryViewProps {
  defaultTab?: LibraryTab;
  className?: string;
}

const TABS: Array<{ id: LibraryTab; label: string; icon: ReactNode }> = [
  { id: 'playlists', label: 'Playlists', icon: <ListMusic /> },
  { id: 'tracks', label: 'Titres likés', icon: <Heart /> },
  { id: 'albums', label: 'Albums', icon: <Disc3 /> },
  { id: 'artists', label: 'Artistes', icon: <Mic2 /> },
];

function firstImage(images: Array<{ url: string }> | undefined): string | null {
  return images?.[0]?.url ?? null;
}

/** The user's library, split into playlists / liked tracks / albums / artists. */
export function LibraryView({ defaultTab = 'playlists', className }: LibraryViewProps) {
  return (
    <AuthGate>
      <LibraryContent defaultTab={defaultTab} className={className} />
    </AuthGate>
  );
}

function LibraryContent({ defaultTab, className }: { defaultTab: LibraryTab; className?: string }) {
  const [tab, setTab] = useState<LibraryTab>(defaultTab);

  const playlistsQuery = useUserPlaylists(50);
  const savedTracksQuery = useSavedTracks(50);
  const savedAlbumsQuery = useSavedAlbums(50);

  const playlists = playlistsQuery.data?.items ?? [];
  const savedTracks = savedTracksQuery.data?.items ?? [];
  const savedAlbums = savedAlbumsQuery.data?.items ?? [];

  const artists = deriveArtists(savedTracks.map((item) => item.track));

  return (
    <div className={cn(styles.root, className)}>
      <h1 className={styles.title}>Votre bibliothèque</h1>

      <div className={styles.tabs} role="tablist" aria-label="Sections de la bibliothèque">
        {TABS.map((item) => (
          <button
            key={item.id}
            type="button"
            role="tab"
            id={`library-tab-${item.id}`}
            aria-selected={tab === item.id}
            aria-controls={`library-panel-${item.id}`}
            className={cn(styles.tab, tab === item.id ? styles.tabActive : '')}
            onClick={() => setTab(item.id)}
          >
            <span className={styles.tabIcon} aria-hidden>
              {item.icon}
            </span>
            {item.label}
          </button>
        ))}
      </div>

      <div
        className={styles.panel}
        role="tabpanel"
        id={`library-panel-${tab}`}
        aria-labelledby={`library-tab-${tab}`}
      >
        {tab === 'playlists' ? (
          playlistsQuery.isLoading ? (
            <GridSkeleton />
          ) : playlists.length > 0 ? (
            <div className={styles.grid}>
              {playlists.map((playlist) => (
                <MediaCard
                  key={playlist.id}
                  title={playlist.name}
                  subtitle={playlist.owner.display_name ?? 'Spotify'}
                  imageUrl={firstImage(playlist.images)}
                  to={buildPath.playlist(playlist.id)}
                />
              ))}
            </div>
          ) : (
            <EmptyState
              icon={<ListMusic />}
              title="Aucune playlist"
              description="Les playlists que vous créez ou suivez apparaîtront ici."
            />
          )
        ) : null}

        {tab === 'tracks' ? (
          savedTracksQuery.isLoading ? (
            <TrackListSkeleton />
          ) : (
            <TrackList
              tracks={savedTracks.map((item) => item.track)}
              showAlbum
              emptyState={
                <EmptyState
                  icon={<Heart />}
                  title="Aucun titre liké"
                  description="Les titres que vous aimez apparaîtront ici."
                />
              }
            />
          )
        ) : null}

        {tab === 'albums' ? (
          savedAlbumsQuery.isLoading ? (
            <GridSkeleton />
          ) : savedAlbums.length > 0 ? (
            <div className={styles.grid}>
              {savedAlbums.map(({ album }) => (
                <MediaCard
                  key={album.id}
                  title={album.name}
                  subtitle={formatArtists(album.artists)}
                  imageUrl={firstImage(album.images)}
                  to={buildPath.album(album.id)}
                />
              ))}
            </div>
          ) : (
            <EmptyState
              icon={<Disc3 />}
              title="Aucun album enregistré"
              description="Les albums que vous enregistrez apparaîtront ici."
            />
          )
        ) : null}

        {tab === 'artists' ? (
          savedTracksQuery.isLoading ? (
            <GridSkeleton />
          ) : artists.length > 0 ? (
            <div className={styles.grid}>
              {artists.map((artist) => (
                <MediaCard
                  key={artist.id}
                  title={artist.name}
                  subtitle="Artiste"
                  imageUrl={artist.image}
                  round
                  to={buildPath.artist(artist.id)}
                />
              ))}
            </div>
          ) : (
            <EmptyState
              icon={<Mic2 />}
              title="Aucun artiste"
              description="Les artistes de vos titres likés apparaîtront ici."
            />
          )
        ) : null}
      </div>
    </div>
  );
}

interface DerivedArtist {
  id: string;
  name: string;
  image: string | null;
}

/** Unique artists across the given tracks, using the first track's cover. */
function deriveArtists(tracks: SimplifiedTrack[]): DerivedArtist[] {
  const map = new Map<string, DerivedArtist>();
  for (const track of tracks) {
    for (const artist of track.artists) {
      if (!map.has(artist.id)) {
        map.set(artist.id, {
          id: artist.id,
          name: artist.name,
          image: firstImage(track.album.images),
        });
      }
    }
  }
  return [...map.values()];
}
