import { Menu } from 'lucide-react';
import { useEffect, useState, type ReactNode } from 'react';
import { useLocation } from 'react-router-dom';

import { useMediaQuery, useTheme, useUi } from '@/hooks';
import { IconButton } from '@/components/ui';

import { MainView } from '../MainView/MainView';
import { NowPlayingBar } from '../NowPlayingBar/NowPlayingBar';
import { PrimaryNav, Sidebar } from '../Sidebar/Sidebar';
import { Topbar } from '../Topbar/Topbar';
import styles from './AppShell.module.css';

export interface AppShellProps {
  children: ReactNode;
  /** Right-aligned topbar actions slot. */
  actions?: ReactNode;
  /** Contextual page title rendered in the topbar. */
  title?: string;
  className?: string;
}

/**
 * Full-page application frame.
 *
 * Grid layout (desktop):
 *
 *   ┌──────────┬──────────────────┐
 *   │          │ Topbar           │
 *   │ Sidebar  ├──────────────────┤
 *   │          │ MainView         │
 *   ├──────────┴──────────────────┤
 *   │ NowPlayingBar               │
 *   └─────────────────────────────┘
 *
 * Adapts to the active `--prism-layout`:
 *  - `sidebar`: nav in the sidebar,
 *  - `topnav`: no sidebar, nav moves into the topbar,
 *  - `hybrid`: sidebar + nav in the topbar,
 *  - `compact`: sidebar forced to its icon rail.
 *
 * On small screens the sidebar becomes a modal drawer.
 */
export function AppShell({ children, actions, title, className }: AppShellProps) {
  const { theme } = useTheme();
  const { sidebarCollapsed, toggleSidebar } = useUi();
  const isMobile = useMediaQuery('(max-width: 56rem)');
  const location = useLocation();
  const [drawerOpen, setDrawerOpen] = useState(false);

  useEffect(() => {
    setDrawerOpen(false);
  }, [location.pathname]);

  const layout = theme.layout;
  const collapsed = isMobile ? false : layout === 'compact' ? true : sidebarCollapsed;
  const showTopbarNav = layout === 'topnav' || layout === 'hybrid';

  return (
    <div
      className={[styles.root, className ?? ''].filter(Boolean).join(' ')}
      data-layout={layout}
      data-mobile={isMobile || undefined}
    >
      {!isMobile ? (
        <div className={styles.sidebarArea}>
          <Sidebar collapsed={collapsed} onToggle={layout === 'compact' ? undefined : toggleSidebar} />
        </div>
      ) : null}

      <div className={styles.topbarArea}>
        <Topbar
          title={title}
          actions={actions}
          leading={
            isMobile ? (
              <IconButton label="Ouvrir la navigation" onClick={() => setDrawerOpen(true)}>
                <Menu />
              </IconButton>
            ) : undefined
          }
          nav={showTopbarNav ? <PrimaryNav /> : undefined}
        />
      </div>

      <div className={styles.mainArea}>
        <MainView>{children}</MainView>
      </div>

      <div className={styles.playerArea}>
        <NowPlayingBar />
      </div>

      {isMobile && drawerOpen ? (
        <div
          className={styles.drawerScrim}
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) setDrawerOpen(false);
          }}
        >
          <aside className={styles.drawer} aria-label="Navigation">
            <Sidebar collapsed={false} onToggle={() => setDrawerOpen(false)} />
          </aside>
        </div>
      ) : null}
    </div>
  );
}
