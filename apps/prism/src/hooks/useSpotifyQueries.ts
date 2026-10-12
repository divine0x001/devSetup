/**
 * Prism — React Query hooks for the Spotify Web API.
 *
 * Every query is gated on `isAuthenticated` so the app boots cleanly in
 * "not configured" mode without firing doomed requests.
 */

import { useQuery } from '@tanstack/react-query';

import { spotifyApi } from '@/lib/spotify/client';
import { useAuthStore } from '@/store/authStore';
import { usePlayerStore } from '@/store/playerStore';
import { useDebouncedValue } from './useDebouncedValue';

/** Reactive auth gate shared by every Spotify query. */
function useIsAuthenticated(): boolean {
  return useAuthStore((state) => state.isAuthenticated);
}

/* -------------------------------------------------------------------------- */
/* Search                                                                      */
/* -------------------------------------------------------------------------- */

/** Debounced search across the given entity types. */
export function useSearch(q: string, types = 'track,album,artist,playlist', limit = 20) {
  const isAuthenticated = useIsAuthenticated();
  const debouncedQ = useDebouncedValue(q, 300);
  const trimmed = debouncedQ.trim();

  return useQuery({
    queryKey: ['search', trimmed, types, limit],
    queryFn: () => spotifyApi.search(trimmed, types, limit),
    enabled: isAuthenticated && trimmed.length > 0,
  });
}

/* -------------------------------------------------------------------------- */
/* Library                                                                     */
/* -------------------------------------------------------------------------- */

export function useSavedTracks(limit = 50) {
  const isAuthenticated = useIsAuthenticated();
  return useQuery({
    queryKey: ['saved-tracks', limit],
    queryFn: () => spotifyApi.getSavedTracks(limit),
    enabled: isAuthenticated,
  });
}

export function useUserPlaylists(limit = 50) {
  const isAuthenticated = useIsAuthenticated();
  return useQuery({
    queryKey: ['user-playlists', limit],
    queryFn: () => spotifyApi.getUserPlaylists(limit),
    enabled: isAuthenticated,
  });
}

/* -------------------------------------------------------------------------- */
/* Catalog                                                                     */
/* -------------------------------------------------------------------------- */

export function usePlaylist(id: string | undefined) {
  const isAuthenticated = useIsAuthenticated();
  return useQuery({
    queryKey: ['playlist', id],
    queryFn: () => {
      if (!id) throw new Error('Prism: usePlaylist requires an id.');
      return spotifyApi.getPlaylist(id);
    },
    enabled: isAuthenticated && !!id,
  });
}

export function usePlaylistTracks(id: string | undefined, limit = 50) {
  const isAuthenticated = useIsAuthenticated();
  return useQuery({
    queryKey: ['playlist-tracks', id, limit],
    queryFn: () => {
      if (!id) throw new Error('Prism: usePlaylistTracks requires an id.');
      return spotifyApi.getPlaylistTracks(id, limit);
    },
    enabled: isAuthenticated && !!id,
  });
}

export function useAlbum(id: string | undefined) {
  const isAuthenticated = useIsAuthenticated();
  return useQuery({
    queryKey: ['album', id],
    queryFn: () => {
      if (!id) throw new Error('Prism: useAlbum requires an id.');
      return spotifyApi.getAlbum(id);
    },
    enabled: isAuthenticated && !!id,
  });
}

export function useArtist(id: string | undefined) {
  const isAuthenticated = useIsAuthenticated();
  return useQuery({
    queryKey: ['artist', id],
    queryFn: () => {
      if (!id) throw new Error('Prism: useArtist requires an id.');
      return spotifyApi.getArtist(id);
    },
    enabled: isAuthenticated && !!id,
  });
}

export function useArtistTopTracks(id: string | undefined, market = 'from_token') {
  const isAuthenticated = useIsAuthenticated();
  return useQuery({
    queryKey: ['artist-top-tracks', id, market],
    queryFn: () => {
      if (!id) throw new Error('Prism: useArtistTopTracks requires an id.');
      return spotifyApi.getArtistTopTracks(id, market);
    },
    enabled: isAuthenticated && !!id,
  });
}

/* -------------------------------------------------------------------------- */
/* Now-playing polling                                                         */
/* -------------------------------------------------------------------------- */

/**
 * Polls the Web API playback state and mirrors it into the player store.
 * Adaptive cadence: 3s while playing, 10s while paused, 15s when idle.
 * Polling is fully disabled while not authenticated.
 */
export function useNowPlayingPolling(): void {
  const isAuthenticated = useIsAuthenticated();
  const setPlayback = usePlayerStore((state) => state.setPlayback);
  const setDevice = usePlayerStore((state) => state.setDevice);

  useQuery({
    queryKey: ['now-playing'],
    queryFn: async () => {
      // 204 (nothing playing) resolves to undefined — normalize to null.
      const state = await spotifyApi.getPlaybackState().catch(() => null);
      setPlayback(state ?? null);
      if (state) setDevice(state.device);
      return state;
    },
    enabled: isAuthenticated,
    refetchInterval: (query) => {
      const state = query.state.data;
      if (!state) return 15_000;
      return state.is_playing ? 3_000 : 10_000;
    },
  });
}