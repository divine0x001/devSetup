/**
 * Prism — Library feature: extra React Query hooks.
 *
 * W2's `@/hooks` covers most endpoints; saved albums are the one gap the
 * library views need, so we add a local hook here (same auth-gating pattern).
 */

import { useQuery } from '@tanstack/react-query';

import { spotifyApi } from '@/lib/spotify/client';
import { useAuthStore } from '@/store/authStore';

/** The user's saved albums, newest first (as returned by Spotify). */
export function useSavedAlbums(limit = 20) {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  return useQuery({
    queryKey: ['saved-albums', limit],
    queryFn: () => spotifyApi.getSavedAlbums(limit),
    enabled: isAuthenticated,
  });
}
