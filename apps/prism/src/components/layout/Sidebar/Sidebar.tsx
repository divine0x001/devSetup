import type { LucideIcon } from 'lucide-react';
import {
  ChevronsLeft,
  ChevronsRight,
  Home,
  Library,
  Music2,
  Plus,
  Search,
  Settings,
} from 'lucide-react';
import { NavLink } from 'react-router-dom';

import { buildPath, ROUTES } from '@/app/routes/routes';
import { useLibrary, useUi } from '@/hooks';
import { Avatar, IconButton, Tooltip } from '@/components/ui';

import styles from './Sidebar.module.css';

export interface SidebarProps {
  /** Defaults to `useUi().sidebarCollapsed`. */
  collapsed?: boolean;
  onToggle?: () => void;
  className?: string;
}

interface NavItem {
  to: string;
  label: string;
  icon: LucideIcon;
  end?: boolean;
}

const NAV_ITEMS: NavItem[] = [
  { to: ROUTES.home, label: 'Accueil', icon: Home, end: true },
  { to: ROUTES.search, label: 'Rechercher', icon: Search },
  { to: ROUTES.library, label: 'Bibliothèque', icon: Library },
  { to: ROUTES.settings, label: 'Paramètres', icon: Settings },
];

/**
 * Primary application navigation. Full sidebar on desktop, animated rail when
 * collapsed, and the same component is reused inside the mobile drawer.
 */
export function Sidebar({ collapsed: collapsedProp, onToggle, className }: SidebarProps) {
  const { sidebarCollapsed, toggleSidebar } = useUi();
  const collapsed = collapsedProp ?? sidebarCollapsed;
  const handleToggle = onToggle ?? toggleSidebar;
  const { playlists } = useLibrary();

  return (
    <aside
      className={[styles.root, className ?? ''].filter(Boolean).join(' ')}
      data-collapsed={collapsed || undefined}
      aria-label="Navigation principale"
    >
      <div className={styles.brand}>
        <NavLink to={ROUTES.home} className={styles.brandLink} aria-label="Prism — Accueil">
          <span className={styles.brandMark} aria-hidden>
            <Music2 />
          </span>
          <span className={styles.brandName}>Prism</span>
        </NavLink>
      </div>

      <nav className={styles.nav} aria-label="Sections">
        <ul role="list" className={styles.navList}>
          {NAV_ITEMS.map(({ to, label, icon: Icon, end }) => {
            const link = (
              <NavLink
                to={to}
                end={end}
                className={({ isActive }) =>
                  [styles.navItem, isActive ? styles.navItemActive : ''].filter(Boolean).join(' ')
                }
              >
                <span className={styles.navIcon} aria-hidden>
                  <Icon />
                </span>
                <span className={styles.navLabel}>{label}</span>
              </NavLink>
            );

            return (
              <li key={to}>
                {collapsed ? (
                  <Tooltip content={label} side="right">
                    {link}
                  </Tooltip>
                ) : (
                  link
                )}
              </li>
            );
          })}
        </ul>
      </nav>

      <div className={styles.playlists}>
        <header className={styles.playlistsHeader}>
          <span className={styles.playlistsTitle}>Playlists</span>
          <IconButton label="Créer une playlist" size="sm">
            <Plus />
          </IconButton>
        </header>

        <ul role="list" className={styles.playlistList}>
          {playlists.length === 0 ? (
            <li className={styles.empty}>Aucune playlist pour l’instant.</li>
          ) : (
            playlists.map((playlist) => (
              <li key={playlist.id}>
                <NavLink
                  to={buildPath.playlist(playlist.id)}
                  className={({ isActive }) =>
                    [styles.playlistItem, isActive ? styles.playlistItemActive : '']
                      .filter(Boolean)
                      .join(' ')
                  }
                >
                  <Avatar
                    src={playlist.images?.[0]?.url ?? null}
                    alt={playlist.name}
                    size={32}
                    className={styles.playlistCover}
                  />
                  <span className={styles.playlistMeta}>
                    <span className={styles.playlistName}>{playlist.name}</span>
                    <span className={styles.playlistOwner}>
                      {playlist.owner?.display_name ?? 'Playlist'}
                    </span>
                  </span>
                </NavLink>
              </li>
            ))
          )}
        </ul>
      </div>

      <div className={styles.footer}>
        <IconButton
          label={collapsed ? 'Déplier la barre latérale' : 'Replier la barre latérale'}
          size="sm"
          onClick={handleToggle}
          aria-pressed={collapsed}
        >
          {collapsed ? <ChevronsRight /> : <ChevronsLeft />}
        </IconButton>
      </div>
    </aside>
  );
}

/** Horizontal variant of the primary nav, used by the `topnav`/`hybrid` layouts. */
export function PrimaryNav({ className }: { className?: string }) {
  return (
    <nav
      className={[styles.primaryNav, className ?? ''].filter(Boolean).join(' ')}
      aria-label="Navigation principale"
    >
      {NAV_ITEMS.map(({ to, label, icon: Icon, end }) => (
        <NavLink
          key={to}
          to={to}
          end={end}
          className={({ isActive }) =>
            [styles.primaryNavItem, isActive ? styles.primaryNavItemActive : '']
              .filter(Boolean)
              .join(' ')
          }
        >
          <span className={styles.navIcon} aria-hidden>
            <Icon />
          </span>
          <span>{label}</span>
        </NavLink>
      ))}
    </nav>
  );
}
