import { useEffect } from 'react';

import { useNowPlayingPolling, useSpotifyPlayerController } from '@/hooks';
import { getStoredTokens } from '@/lib/spotify/auth';
import { spotifyApi } from '@/lib/spotify/client';
import { useAuthStore } from '@/store/authStore';

/**
 * Headless app bootstrap (Worker 1).
 *
 * Mounted once inside `AppProviders` (so it sits under the QueryClient). It:
 *  - starts the Web Playback SDK controller (no-op while unauthenticated),
 *  - starts the now-playing polling loop (disabled while unauthenticated),
 *  - restores a persisted session on cold start: if tokens exist in
 *    localStorage, mirror them into the auth store and fetch the profile.
 *
 * It never throws when Spotify is unconfigured: with no stored tokens it simply
 * does nothing, and the W5 `AuthGate`/`LoginView` renders the "not configured"
 * state.
 */
export function AppBootstrap() {
  useSpotifyPlayerController();
  useNowPlayingPolling();

  useEffect(() => {
    const tokens = getStoredTokens();
    if (!tokens) return;

    const auth = useAuthStore.getState();
    // The PKCE callback may have already populated the store this session.
    if (auth.isAuthenticated) return;

    auth.setTokens(tokens);
    auth.setStatus('authenticated');

    void spotifyApi
      .getMe()
      .then((profile) => {
        useAuthStore.getState().setProfile(profile);
      })
      .catch(() => {
        // Non-fatal: keep the session; the profile can be retried later.
      });
  }, []);

  return null;
}