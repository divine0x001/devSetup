import { Search, X } from 'lucide-react';
import { useEffect, useState, type CSSProperties } from 'react';
import { Link, useSearchParams } from 'react-router-dom';

import { buildPath } from '@/app/routes/routes';
import { useSearch } from '@/hooks';

import { formatArtists } from '../../lib/format';
import { cn } from '../../lib/utils';
import { AuthGate } from '../../components/AuthGate/AuthGate';
import { EmptyState } from '../../components/EmptyState/EmptyState';
import { GridSkeleton } from '../../components/GridSkeleton/GridSkeleton';
import { MediaCard } from '../../components/MediaCard/MediaCard';
import { TrackList } from '../../components/TrackList/TrackList';
import { TrackListSkeleton } from '../../components/TrackListSkeleton/TrackListSkeleton';
import styles from './SearchView.module.css';

import type {
  SimplifiedAlbum,
  SimplifiedArtist,
  SimplifiedPlaylist,
  SimplifiedTrack,
} from '@/lib/spotify/types';

const CATEGORIES = [
  'Pop',
  'Hip-Hop',
  'Rock',
  'Électro',
  'Jazz',
  'Classique',
  'Rap français',
  'Chill',
  'Focus',
  'Workout',
  'Années 80',
  'Indie',
];

function firstImage(images: Array<{ url: string }> | undefined): string | null {
  return images?.[0]?.url ?? null;
}

/** Search across tracks, albums, artists and playlists. */
export function SearchView() {
  return (
    <AuthGate>
      <SearchContent />
    </AuthGate>
  );
}

function SearchContent() {
  const [params, setParams] = useSearchParams();
  const q = params.get('q') ?? '';
  const [input, setInput] = useState(q);

  useEffect(() => {
    setInput(q);
  }, [q]);

  const searchQuery = useSearch(q, 'track,album,artist,playlist', 20);
  const results = searchQuery.data;

  const tracks = results?.tracks?.items ?? [];
  const albums = results?.albums?.items ?? [];
  const artists = results?.artists?.items ?? [];
  const playlists = results?.playlists?.items ?? [];

  const hasResults =
    tracks.length > 0 || albums.length > 0 || artists.length > 0 || playlists.length > 0;

  const updateQuery = (value: string) => {
    setInput(value);
    setParams(value.trim() ? { q: value } : {}, { replace: true });
  };

  const topResult = buildTopResult({ tracks, albums, artists, playlists });

  return (
    <div className={styles.root}>
      <div className={styles.searchBar}>
        <Search className={styles.searchIcon} aria-hidden />
        <input
          type="search"
          className={styles.input}
          value={input}
          placeholder="Que souhaitez-vous écouter ?"
          aria-label="Rechercher"
          onChange={(event) => updateQuery(event.currentTarget.value)}
        />
        {input ? (
          <button
            type="button"
            className={styles.clear}
            aria-label="Effacer la recherche"
            onClick={() => updateQuery('')}
          >
            <X aria-hidden />
          </button>
        ) : null}
      </div>

      {!q.trim() ? (
        <section className={styles.categories} aria-label="Parcourir">
          <h2 className={styles.sectionTitle}>Parcourir tout</h2>
          <div className={styles.categoryGrid}>
            {CATEGORIES.map((category, index) => (
              <button
                key={category}
                type="button"
                className={styles.category}
                style={{ '--category-hue': `${(index * 37) % 360}` } as CSSProperties}
                onClick={() => updateQuery(category)}
              >
                {category}
              </button>
            ))}
          </div>
        </section>
      ) : searchQuery.isLoading ? (
        <div className={styles.loading}>
          <TrackListSkeleton rows={6} />
          <GridSkeleton cards={6} />
        </div>
      ) : !hasResults ? (
        <EmptyState
          icon={<Search />}
          title={`Aucun résultat pour « ${q} »`}
          description="Vérifiez l’orthographe ou essayez d’autres mots-clés."
        />
      ) : (
        <div className={styles.results}>
          {topResult ? (
            <section className={styles.topSection} aria-label="Résultat principal">
              <h2 className={styles.sectionTitle}>Résultat principal</h2>
              <Link to={topResult.to} className={styles.topResult}>
                <span className={cn(styles.topArtwork, topResult.round ? styles.round : '')}>
                  {topResult.image ? (
                    <img src={topResult.image} alt="" loading="lazy" />
                  ) : (
                    <span className={styles.topPlaceholder} aria-hidden />
                  )}
                </span>
                <span className={styles.topMeta}>
                  <span className={styles.topTitle}>{topResult.title}</span>
                  <span className={styles.topSubtitle}>{topResult.subtitle}</span>
                </span>
              </Link>
            </section>
          ) : null}

          {tracks.length > 0 ? (
            <section aria-label="Titres">
              <h2 className={styles.sectionTitle}>Titres</h2>
              <TrackList tracks={tracks} showAlbum />
            </section>
          ) : null}

          {artists.length > 0 ? (
            <section aria-label="Artistes">
              <h2 className={styles.sectionTitle}>Artistes</h2>
              <div className={styles.grid}>
                {artists.map((artist) => (
                  <MediaCard
                    key={artist.id}
                    title={artist.name}
                    subtitle="Artiste"
                    imageUrl={firstImage(artist.images)}
                    round
                    to={buildPath.artist(artist.id)}
                  />
                ))}
              </div>
            </section>
          ) : null}

          {albums.length > 0 ? (
            <section aria-label="Albums">
              <h2 className={styles.sectionTitle}>Albums</h2>
              <div className={styles.grid}>
                {albums.map((album) => (
                  <MediaCard
                    key={album.id}
                    title={album.name}
                    subtitle={formatArtists(album.artists)}
                    imageUrl={firstImage(album.images)}
                    to={buildPath.album(album.id)}
                  />
                ))}
              </div>
            </section>
          ) : null}

          {playlists.length > 0 ? (
            <section aria-label="Playlists">
              <h2 className={styles.sectionTitle}>Playlists</h2>
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
            </section>
          ) : null}
        </div>
      )}
    </div>
  );
}

interface TopResult {
  to: string;
  title: string;
  subtitle: string;
  image: string | null;
  round: boolean;
}

function buildTopResult({
  tracks,
  albums,
  artists,
  playlists,
}: {
  tracks: SimplifiedTrack[];
  albums: SimplifiedAlbum[];
  artists: SimplifiedArtist[];
  playlists: SimplifiedPlaylist[];
}): TopResult | null {
  if (artists[0]) {
    return {
      to: buildPath.artist(artists[0].id),
      title: artists[0].name,
      subtitle: 'Artiste',
      image: firstImage(artists[0].images),
      round: true,
    };
  }
  if (albums[0]) {
    return {
      to: buildPath.album(albums[0].id),
      title: albums[0].name,
      subtitle: `Album · ${formatArtists(albums[0].artists)}`,
      image: firstImage(albums[0].images),
      round: false,
    };
  }
  if (tracks[0]) {
    return {
      to: buildPath.album(tracks[0].album.id),
      title: tracks[0].name,
      subtitle: `Titre · ${formatArtists(tracks[0].artists)}`,
      image: firstImage(tracks[0].album.images),
      round: false,
    };
  }
  if (playlists[0]) {
    return {
      to: buildPath.playlist(playlists[0].id),
      title: playlists[0].name,
      subtitle: `Playlist · ${playlists[0].owner.display_name ?? 'Spotify'}`,
      image: firstImage(playlists[0].images),
      round: false,
    };
  }
  return null;
}
