/**
 * Prism — Application shell barrel (Worker 4).
 *
 * Wire the shell around the router with a layout route (see CONTRACT §8):
 *
 *   <Route element={<AppShell><Outlet /></AppShell>}>
 *     ...child routes
 *   </Route>
 */

export { AppShell } from './AppShell/AppShell';
export type { AppShellProps } from './AppShell/AppShell';

export { CommandPalette } from './CommandPalette/CommandPalette';

export { GlobalShortcuts } from './GlobalShortcuts/GlobalShortcuts';

export { MainView } from './MainView/MainView';
export type { MainViewProps } from './MainView/MainView';

export { NowPlayingBar } from './NowPlayingBar/NowPlayingBar';
export type { NowPlayingBarProps } from './NowPlayingBar/NowPlayingBar';

export { PrimaryNav, Sidebar } from './Sidebar/Sidebar';
export type { SidebarProps } from './Sidebar/Sidebar';

export { Topbar } from './Topbar/Topbar';
export type { TopbarProps } from './Topbar/Topbar';
