/**
 * Prism — Public hooks (FROZEN signatures).
 *
 * Worker 2 owns `src/hooks/**`. Other workers import from '@/hooks'.
 * These are thin, typed selectors over the stores plus a few utilities, so the
 * real data-fetching hooks can be layered on top without breaking consumers.
 */

import { useCallback, useMemo } from 'react';

import { beginLogin, signOut as spotifySignOut } from '@/lib/spotify/auth';
import { spotifyApi } from '@/lib/spotify/client';
import {
  createDemoProfile,
  disableDemoMode,
  isDemoMode,
} from '@/lib/spotify/demo';
import { useAuthStore } from '@/store/authStore';
import { useLibraryStore } from '@/store/libraryStore';
import { usePlayerStore } from '@/store/playerStore';
import { useThemeStore } from '@/store/themeStore';
import { useUiStore } from '@/store/uiStore';

import type { ColorMode, Density, LayoutVariant, RadiusScale, Theme } from '@/theme/types';

/* -------------------------------------------------------------------------- */
/* Theme                                                                       */
/* -------------------------------------------------------------------------- */

export interface UseThemeResult {
  theme: Theme;
  setLayout: (layout: LayoutVariant) => void;
  setDensity: (density: Density) => void;
  setRadius: (radiusScale: RadiusScale) => void;
  setColorMode: (colorMode: ColorMode) => void;
  applyPreset: (presetId: string) => void;
  reset: () => void;
}

export function useTheme(): UseThemeResult {
  const theme = useThemeStore((state) => state.theme);
  const setLayout = useThemeStore((state) => state.setLayout);
  const setDensity = useThemeStore((state) => state.setDensity);
  const setRadius = useThemeStore((state) => state.setRadius);
  const setColorMode = useThemeStore((state) => state.setColorMode);
  const applyPreset = useThemeStore((state) => state.applyPreset);
  const reset = useThemeStore((state) => state.reset);

  return useMemo(
    () => ({ theme, setLayout, setDensity, setRadius, setColorMode, applyPreset, reset }),
    [theme, setLayout, setDensity, setRadius, setColorMode, applyPreset, reset],
  );
}

/* -------------------------------------------------------------------------- */
/* Auth                                                                        */
/* -------------------------------------------------------------------------- */

export interface UseAuthResult {
  status: ReturnType<typeof useAuthStore.getState>['status'];
  isAuthenticated: boolean;
  profile: ReturnType<typeof useAuthStore.getState>['profile'];
  error: string | null;
  login: (returnPath?: string) => Promise<void>;
  logout: () => void;
}

export function useAuth(): UseAuthResult {
  const status = useAuthStore((state) => state.status);
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  const profile = useAuthStore((state) => state.profile);
  const error = useAuthStore((state) => state.error);
  const reset = useAuthStore((state) => state.reset);

  const login = useCallback(async (returnPath = '/') => {
    await beginLogin(returnPath);
    if (isDemoMode()) {
      // No redirect happened; hydrate the auth store so queries fire.
      const auth = useAuthStore.getState();
      const profile = await spotifyApi.getMe().catch(() => null);
      if (profile) auth.setProfile(profile);
    }
  }, []);
  const logout = useCallback(() => {
    spotifySignOut();
    if (isDemoMode()) disableDemoMode();
    reset();
  }, [reset]);

  const demo = isDemoMode();

  return useMemo(() => {
    if (demo) {
      return {
        status: 'authenticated',
        isAuthenticated: true,
        profile: createDemoProfile(),
        error: null,
        login,
        logout,
      };
    }
    return { status, isAuthenticated, profile, error, login, logout };
  }, [demo, status, isAuthenticated, profile, error, login, logout]);
}

/* -------------------------------------------------------------------------- */
/* Player                                                                      */
/* -------------------------------------------------------------------------- */

export interface UsePlayerResult {
  playback: ReturnType<typeof usePlayerStore.getState>['playback'];
  device: ReturnType<typeof usePlayerStore.getState>['device'];
  isReady: boolean;
  isPlaying: boolean;
  progressMs: number;
  volume: number;
  setProgressMs: (ms: number) => void;
  setVolume: (volume: number) => void;
}

export function usePlayer(): UsePlayerResult {
  const playback = usePlayerStore((state) => state.playback);
  const device = usePlayerStore((state) => state.device);
  const isReady = usePlayerStore((state) => state.isReady);
  const isPlaying = usePlayerStore((state) => state.isPlaying);
  const progressMs = usePlayerStore((state) => state.progressMs);
  const volume = usePlayerStore((state) => state.volume);
  const setProgressMs = usePlayerStore((state) => state.setProgressMs);
  const setVolume = usePlayerStore((state) => state.setVolume);

  return useMemo(
    () => ({
      playback,
      device,
      isReady,
      isPlaying,
      progressMs,
      volume,
      setProgressMs,
      setVolume,
    }),
    [playback, device, isReady, isPlaying, progressMs, volume, setProgressMs, setVolume],
  );
}

/* -------------------------------------------------------------------------- */
/* Library                                                                     */
/* -------------------------------------------------------------------------- */

export interface UseLibraryResult {
  playlists: ReturnType<typeof useLibraryStore.getState>['playlists'];
  isTrackSaved: (id: string) => boolean;
  toggleSavedTrack: (id: string) => void;
}

export function useLibrary(): UseLibraryResult {
  const playlists = useLibraryStore((state) => state.playlists);
  const isTrackSaved = useLibraryStore((state) => state.isTrackSaved);
  const toggleSavedTrack = useLibraryStore((state) => state.toggleSavedTrack);

  return useMemo(
    () => ({ playlists, isTrackSaved, toggleSavedTrack }),
    [playlists, isTrackSaved, toggleSavedTrack],
  );
}

/* -------------------------------------------------------------------------- */
/* UI                                                                          */
/* -------------------------------------------------------------------------- */

export interface UseUiResult {
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
  commandPaletteOpen: boolean;
  setCommandPaletteOpen: (open: boolean) => void;
  nowPlayingOpen: boolean;
  setNowPlayingOpen: (open: boolean) => void;
}

export function useUi(): UseUiResult {
  const sidebarCollapsed = useUiStore((state) => state.sidebarCollapsed);
  const toggleSidebar = useUiStore((state) => state.toggleSidebar);
  const commandPaletteOpen = useUiStore((state) => state.commandPaletteOpen);
  const setCommandPaletteOpen = useUiStore((state) => state.setCommandPaletteOpen);
  const nowPlayingOpen = useUiStore((state) => state.nowPlayingOpen);
  const setNowPlayingOpen = useUiStore((state) => state.setNowPlayingOpen);

  return useMemo(
    () => ({
      sidebarCollapsed,
      toggleSidebar,
      commandPaletteOpen,
      setCommandPaletteOpen,
      nowPlayingOpen,
      setNowPlayingOpen,
    }),
    [
      sidebarCollapsed,
      toggleSidebar,
      commandPaletteOpen,
      setCommandPaletteOpen,
      nowPlayingOpen,
      setNowPlayingOpen,
    ],
  );
}

/* -------------------------------------------------------------------------- */
/* Re-exports of new hooks                                                     */
/* -------------------------------------------------------------------------- */

export { useDebouncedValue, useMediaQuery } from './useDebouncedValue';
export {
  useSearch,
  useSavedTracks,
  useUserPlaylists,
  usePlaylist,
  usePlaylistTracks,
  useAlbum,
  useArtist,
  useArtistTopTracks,
  useNowPlayingPolling,
} from './useSpotifyQueries';
export { usePlaybackControls, type PlaybackControls, type PlayBody } from './usePlaybackControls';
export { useSpotifyPlayerController } from './useSpotifyPlayerController';
export { useLibraryBootstrap } from './useLibraryBootstrap';