import { ArrowLeft, ArrowRight, LogIn, Search } from 'lucide-react';
import { useEffect, useState, type FormEvent, type ReactNode } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { buildPath } from '@/app/routes/routes';
import { useAuth, useUi } from '@/hooks';
import { Avatar, Button, IconButton, Kbd } from '@/components/ui';

import styles from './Topbar.module.css';

export interface TopbarProps {
  title?: string;
  /** Right-aligned actions slot. */
  actions?: ReactNode;
  /** Leading slot (e.g. the mobile menu button). */
  leading?: ReactNode;
  /** Primary nav slot (used by the topnav / hybrid layouts). */
  nav?: ReactNode;
  className?: string;
}

/**
 * Sticky application header: history controls, contextual title, search field
 * (routes to `/search?q=`), right-aligned actions and the user avatar.
 */
export function Topbar({ title, actions, leading, nav, className }: TopbarProps) {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { profile, isAuthenticated, login } = useAuth();
  const { setCommandPaletteOpen } = useUi();
  const [query, setQuery] = useState(searchParams.get('q') ?? '');

  useEffect(() => {
    setQuery(searchParams.get('q') ?? '');
  }, [searchParams]);

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    navigate(buildPath.search(query.trim() || undefined));
  };

  return (
    <header className={[styles.root, className ?? ''].filter(Boolean).join(' ')}>
      <div className={styles.left}>
        {leading ? <span className={styles.leading}>{leading}</span> : null}

        <div className={styles.navButtons}>
          <IconButton label="Retour" size="sm" onClick={() => navigate(-1)}>
            <ArrowLeft />
          </IconButton>
          <IconButton label="Avancer" size="sm" onClick={() => navigate(1)}>
            <ArrowRight />
          </IconButton>
        </div>

        {title ? <h1 className={styles.title}>{title}</h1> : null}
        {nav ? <div className={styles.navSlot}>{nav}</div> : null}
      </div>

      <form className={styles.search} role="search" onSubmit={onSubmit}>
        <Search className={styles.searchIcon} aria-hidden />
        <input
          type="search"
          className={styles.searchInput}
          placeholder="Rechercher titres, artistes, albums…"
          value={query}
          onChange={(event) => setQuery(event.currentTarget.value)}
          aria-label="Rechercher"
        />
        <button
          type="button"
          className={styles.searchHint}
          onClick={() => setCommandPaletteOpen(true)}
          aria-label="Ouvrir la palette de commandes"
        >
          <Kbd>⌘K</Kbd>
        </button>
      </form>

      <div className={styles.right}>
        {actions ? <div className={styles.actions}>{actions}</div> : null}

        {isAuthenticated ? (
          <Avatar
            src={profile?.images?.[0]?.url ?? null}
            alt={profile?.display_name ?? 'Profil'}
            size={36}
          />
        ) : (
          <Button size="sm" variant="secondary" iconLeft={<LogIn />} onClick={() => login()}>
            Se connecter
          </Button>
        )}
      </div>
    </header>
  );
}
