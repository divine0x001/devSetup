/**
 * Prism — Library bootstrap hook.
 *
 * Once authenticated, loads the signed-in user's saved tracks, saved albums
 * and playlists in parallel and mirrors them into the library store. Failures
 * are non-fatal: the store stays empty and the views render their empty states.
 */

import { useEffect } from 'react';

import { spotifyApi } from '@/lib/spotify/client';
import { useAuthStore } from '@/store/authStore';
import { useLibraryStore } from '@/store/libraryStore';

export function useLibraryBootstrap(): void {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);

  useEffect(() => {
    if (!isAuthenticated) return;

    let cancelled = false;

    (async () => {
      try {
        const [savedTracks, savedAlbums, playlists] = await Promise.all([
          spotifyApi.getSavedTracks(50),
          spotifyApi.getSavedAlbums(50),
          spotifyApi.getUserPlaylists(50),
        ]);
        if (cancelled) return;

        const library = useLibraryStore.getState();
        library.setSavedTrackIds(savedTracks.items.map((item) => item.track.id));
        library.setSavedAlbumIds(savedAlbums.items.map((item) => item.album.id));
        library.setPlaylists(playlists.items);
      } catch {
        // Non-fatal: leave the library empty rather than crashing the shell.
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [isAuthenticated]);
}