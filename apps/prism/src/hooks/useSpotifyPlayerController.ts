/**
 * Prism — Web Playback SDK controller.
 *
 * Loads the SDK, creates a player instance, connects it to Spotify Connect and
 * keeps the player store in sync with SDK events. The controller is idempotent
 * and only runs while authenticated.
 */

import { useEffect, useRef } from 'react';

import { getValidAccessToken } from '@/lib/spotify/auth';
import { isDemoMode } from '@/lib/spotify/demo';
import {
  createSpotifyPlayer,
  loadSpotifySdk,
  type SpotifyPlayerInstance,
  type SpotifySdkPlaybackState,
} from '@/lib/spotify/player';
import { useAuthStore } from '@/store/authStore';
import { usePlayerStore } from '@/store/playerStore';

function mapSdkState(state: SpotifySdkPlaybackState | null): void {
  const { setPlayback, setPlaying, setProgressMs } = usePlayerStore.getState();
  if (!state) {
    setPlayback(null);
    setPlaying(false);
    setProgressMs(0);
    return;
  }

  // The SDK's track_window.current_track is the closest shape we have; we do
  // not attempt to fully map it to the Web API SimplifiedTrack here (the API
  // polling hook remains the source of truth for rich metadata). We only keep
  // local progress/playing flags in sync while the SDK is active.
  setPlaying(!state.paused);
  setProgressMs(state.position);
}

export function useSpotifyPlayerController(): void {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  const setSdkDeviceId = usePlayerStore((state) => state.setSdkDeviceId);
  const setReady = usePlayerStore((state) => state.setReady);
  const playerRef = useRef<SpotifyPlayerInstance | null>(null);

  useEffect(() => {
    // Demo mode: no Web Playback SDK on the network — fall back to the mock
    // playback state driven by useNowPlayingPolling.
    if (isDemoMode()) {
      return;
    }

    if (!isAuthenticated) {
      // Disconnect and reset SDK state on sign-out.
      playerRef.current?.disconnect();
      playerRef.current = null;
      setSdkDeviceId(null);
      setReady(false);
      return;
    }

    let mounted = true;

    (async () => {
      try {
        await loadSpotifySdk();
        if (!mounted) return;

        const player = createSpotifyPlayer({
          name: 'Prism Web Player',
          getOAuthToken: async (callback) => {
            const token = await getValidAccessToken();
            if (token) callback(token);
          },
          volume: 0.7,
        });

        playerRef.current = player;

        player.addListener('ready', ({ device_id }) => {
          if (!mounted) return;
          setSdkDeviceId(device_id);
          setReady(true);
        });

        player.addListener('not_ready', () => {
          if (!mounted) return;
          setReady(false);
        });

        player.addListener('player_state_changed', (state) => {
          if (!mounted) return;
          mapSdkState(state);
        });

        player.addListener('account_error', () => {
          if (!mounted) return;
          setReady(false);
        });

        player.addListener('playback_error', () => {
          if (!mounted) return;
          setReady(false);
        });

        player.addListener('authentication_error', () => {
          if (!mounted) return;
          setReady(false);
        });

        player.addListener('initialization_error', () => {
          if (!mounted) return;
          setReady(false);
        });

        await player.connect();
      } catch {
        // SDK load/connect failures are non-fatal (Connect fallback remains).
        if (mounted) {
          setReady(false);
          setSdkDeviceId(null);
        }
      }
    })();

    return () => {
      mounted = false;
      playerRef.current?.disconnect();
      playerRef.current = null;
    };
  }, [isAuthenticated, setReady, setSdkDeviceId]);
}