/**
 * Prism — Library store (FROZEN shape).
 *
 * Local cache of the user's saved items for O(1) "is saved?" lookups across the
 * UI. Worker 2 keeps it in sync with the Web API.
 */

import { create } from 'zustand';

import type { SimplifiedPlaylist } from '@/lib/spotify/types';

export interface LibraryStoreState {
  savedTrackIds: string[];
  savedAlbumIds: string[];
  followedArtistIds: string[];
  playlists: SimplifiedPlaylist[];
  isTrackSaved: (id: string) => boolean;
  isAlbumSaved: (id: string) => boolean;
  isArtistFollowed: (id: string) => boolean;
  setSavedTrackIds: (ids: string[]) => void;
  setSavedAlbumIds: (ids: string[]) => void;
  setFollowedArtistIds: (ids: string[]) => void;
  setPlaylists: (playlists: SimplifiedPlaylist[]) => void;
  toggleSavedTrack: (id: string) => void;
  reset: () => void;
}

export const useLibraryStore = create<LibraryStoreState>((set, get) => ({
  savedTrackIds: [],
  savedAlbumIds: [],
  followedArtistIds: [],
  playlists: [],
  isTrackSaved: (id) => get().savedTrackIds.includes(id),
  isAlbumSaved: (id) => get().savedAlbumIds.includes(id),
  isArtistFollowed: (id) => get().followedArtistIds.includes(id),
  setSavedTrackIds: (savedTrackIds) => set({ savedTrackIds }),
  setSavedAlbumIds: (savedAlbumIds) => set({ savedAlbumIds }),
  setFollowedArtistIds: (followedArtistIds) => set({ followedArtistIds }),
  setPlaylists: (playlists) => set({ playlists }),
  toggleSavedTrack: (id) =>
    set((state) => ({
      savedTrackIds: state.savedTrackIds.includes(id)
        ? state.savedTrackIds.filter((trackId) => trackId !== id)
        : [...state.savedTrackIds, id],
    })),
  reset: () =>
    set({
      savedTrackIds: [],
      savedAlbumIds: [],
      followedArtistIds: [],
      playlists: [],
    }),
}));
