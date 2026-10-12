/**
 * Prism — UI store (FROZEN shape).
 *
 * Ephemeral UI state (panels, overlays, transient toasts). Only the sidebar
 * collapsed flag is persisted — the rest is intentionally session-scoped.
 */

import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';

export interface Toast {
  id: string;
  message: string;
  variant: 'info' | 'success' | 'warning' | 'danger';
}

export interface UiStoreState {
  sidebarCollapsed: boolean;
  commandPaletteOpen: boolean;
  nowPlayingOpen: boolean;
  queueOpen: boolean;
  toasts: Toast[];
  setSidebarCollapsed: (collapsed: boolean) => void;
  toggleSidebar: () => void;
  setCommandPaletteOpen: (open: boolean) => void;
  setNowPlayingOpen: (open: boolean) => void;
  setQueueOpen: (open: boolean) => void;
  pushToast: (toast: Omit<Toast, 'id'>) => void;
  dismissToast: (id: string) => void;
}

export const useUiStore = create<UiStoreState>()(
  persist(
    (set) => ({
      sidebarCollapsed: false,
      commandPaletteOpen: false,
      nowPlayingOpen: false,
      queueOpen: false,
      toasts: [],
      setSidebarCollapsed: (sidebarCollapsed) => set({ sidebarCollapsed }),
      toggleSidebar: () =>
        set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
      setCommandPaletteOpen: (commandPaletteOpen) => set({ commandPaletteOpen }),
      setNowPlayingOpen: (nowPlayingOpen) => set({ nowPlayingOpen }),
      setQueueOpen: (queueOpen) => set({ queueOpen }),
      pushToast: (toast) =>
        set((state) => ({
          toasts: [...state.toasts, { ...toast, id: crypto.randomUUID() }],
        })),
      dismissToast: (id) =>
        set((state) => ({ toasts: state.toasts.filter((toast) => toast.id !== id) })),
    }),
    {
      name: 'prism.ui',
      storage: createJSONStorage(() => localStorage),
      version: 1,
      partialize: (state) => ({ sidebarCollapsed: state.sidebarCollapsed }),
    },
  ),
);