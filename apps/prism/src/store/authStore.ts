/**
 * Prism — Auth store (FROZEN shape).
 *
 * Holds the Spotify session. Tokens themselves are the source of truth in
 * `src/lib/spotify/auth.ts` (localStorage); this store mirrors the auth status
 * for the UI.
 */

import { create } from 'zustand';

import type { SpotifyTokens, UserProfile } from '@/lib/spotify/types';

export type AuthStatus = 'anonymous' | 'authenticating' | 'authenticated';

export interface AuthStoreState {
  status: AuthStatus;
  isAuthenticated: boolean;
  tokens: SpotifyTokens | null;
  profile: UserProfile | null;
  error: string | null;
  setStatus: (status: AuthStatus) => void;
  setTokens: (tokens: SpotifyTokens | null) => void;
  setProfile: (profile: UserProfile | null) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

const initialState = {
  status: 'anonymous' as AuthStatus,
  isAuthenticated: false,
  tokens: null,
  profile: null,
  error: null,
};

export const useAuthStore = create<AuthStoreState>((set) => ({
  ...initialState,
  setStatus: (status) => set({ status }),
  setTokens: (tokens) =>
    set({
      tokens,
      isAuthenticated: tokens !== null,
      status: tokens ? 'authenticated' : 'anonymous',
    }),
  setProfile: (profile) => set({ profile }),
  setError: (error) => set({ error }),
  reset: () => set({ ...initialState }),
}));
