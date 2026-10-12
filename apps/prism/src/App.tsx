import { AppProviders } from '@/app/providers/AppProviders';
import { AppRoutes } from '@/app/routes';
import { CommandPalette, GlobalShortcuts } from '@/components/layout';
import { NowPlayingView } from '@/features/player';

/**
 * Application root.
 *
 * Providers (Worker 1) own: React Query, Theme, Router + the headless
 * bootstrap (SDK controller, now-playing polling, session restore).
 *
 * Global overlays mounted once, inside the Router:
 *  - `NowPlayingView`  (W5) — full-screen player, driven by uiStore
 *  - `CommandPalette`  (W4) — ⌘K palette
 *  - `GlobalShortcuts` (W4) — keyboard shortcuts
 */
export default function App() {
  return (
    <AppProviders>
      <AppRoutes />
      <NowPlayingView />
      <CommandPalette />
      <GlobalShortcuts />
    </AppProviders>
  );
}