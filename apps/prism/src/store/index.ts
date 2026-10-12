/**
 * Prism — store barrel (FROZEN public surface).
 *
 * Worker 2 owns every file in `src/store/**`. Other workers must import
 * individual stores from their own module, e.g.:
 *   import { usePlayerStore } from '@/store/playerStore';
 */

export { useAuthStore } from './authStore';
export type { AuthStoreState, AuthStatus } from './authStore';

export { usePlayerStore } from './playerStore';
export type { PlayerStoreState } from './playerStore';

export { useLibraryStore } from './libraryStore';
export type { LibraryStoreState } from './libraryStore';

export { useUiStore } from './uiStore';
export type { UiStoreState, Toast } from './uiStore';

export { useThemeStore } from './themeStore';
export type { ThemeStoreState } from './themeStore';
