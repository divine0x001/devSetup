import { Outlet, Route, Routes } from 'react-router-dom';

import { AppShell } from '@/components/layout';
import { CustomizerTrigger } from '@/features/customizer';

import { ROUTES } from './routes';
import {
  AlbumPage,
  ArtistPage,
  AuthCallbackPage,
  HomePage,
  LibraryPage,
  NotFoundPage,
  PlaylistPage,
  SearchPage,
  SettingsPage,
} from './pages';

/**
 * Route table (Worker 1).
 *
 * Every product route is wrapped in the W4 shell via a layout route
 * (`AppShell` + `<Outlet />`). The auth callback stays OUTSIDE the shell so it
 * renders full-screen during the PKCE redirect.
 */
export function AppRoutes() {
  return (
    <Routes>
      {/* Full-screen, outside the shell */}
      <Route path={ROUTES.authCallback} element={<AuthCallbackPage />} />

      {/* Product routes inside the shell */}
      <Route
        element={
          <AppShell actions={<CustomizerTrigger />}>
            <Outlet />
          </AppShell>
        }
      >
        <Route path={ROUTES.home} element={<HomePage />} />
        <Route path={ROUTES.search} element={<SearchPage />} />
        <Route path={ROUTES.library} element={<LibraryPage />} />
        <Route path={ROUTES.playlist} element={<PlaylistPage />} />
        <Route path={ROUTES.album} element={<AlbumPage />} />
        <Route path={ROUTES.artist} element={<ArtistPage />} />
        <Route path={ROUTES.settings} element={<SettingsPage />} />
        <Route path={ROUTES.notFound} element={<NotFoundPage />} />
      </Route>
    </Routes>
  );
}