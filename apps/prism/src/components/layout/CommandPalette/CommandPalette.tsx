import { AnimatePresence, motion } from 'framer-motion';
import {
  Disc3,
  Heart,
  Home,
  Library,
  ListMusic,
  MonitorSmartphone,
  Moon,
  Music,
  Pause,
  Play,
  Search,
  Settings,
  SkipBack,
  SkipForward,
  Sun,
  User,
} from 'lucide-react';
import {
  useEffect,
  useId,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
} from 'react';
import { createPortal } from 'react-dom';
import { useNavigate } from 'react-router-dom';

import { buildPath, ROUTES } from '@/app/routes/routes';
import { useLibrary, usePlaybackControls, usePlayer, useSearch, useUi } from '@/hooks';
import { useThemeStore } from '@/store/themeStore';
import { createTheme } from '@/theme/types';
import { Kbd, Spinner } from '@/components/ui';

import styles from './CommandPalette.module.css';

interface CommandItem {
  id: string;
  label: string;
  hint?: string;
  icon?: ReactNode;
  /** A color swatch (preset accent) shown instead of an icon. */
  swatch?: string;
  run: () => void;
}

interface CommandGroup {
  id: string;
  title: string;
  items: CommandItem[];
}

/**
 * Global command palette. The outer component only reads the open flag; the
 * actual palette (and every Spotify query it needs) is a child mounted **only
 * when open** — so hooks are never called conditionally and no request fires
 * while the palette is closed.
 */
export function CommandPalette() {
  const { commandPaletteOpen, setCommandPaletteOpen } = useUi();

  return (
    <AnimatePresence>
      {commandPaletteOpen ? (
        <CommandPalettePanel key="command-palette" onClose={() => setCommandPaletteOpen(false)} />
      ) : null}
    </AnimatePresence>
  );
}

function CommandPalettePanel({ onClose }: { onClose: () => void }) {
  const navigate = useNavigate();
  const controls = usePlaybackControls();
  const { isTrackSaved, toggleSavedTrack } = useLibrary();
  const { playback, isPlaying } = usePlayer();
  const presets = useThemeStore((state) => state.presets);
  const setColorMode = useThemeStore((state) => state.setColorMode);
  const applyPreset = useThemeStore((state) => state.applyPreset);

  const [query, setQuery] = useState('');
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const baseId = useId();
  const listId = `${baseId}-list`;
  const optionId = (index: number) => `${baseId}-opt-${index}`;

  // Only query Spotify for queries of 2+ characters (useSearch already debounces).
  const trimmedQuery = query.trim();
  const searchQuery = trimmedQuery.length >= 2 ? trimmedQuery : '';
  const search = useSearch(searchQuery, 'track,album,artist,playlist', 6);

  const go = (path: string) => {
    onClose();
    navigate(path);
  };
  const runThenClose = (fn: () => void) => {
    onClose();
    fn();
  };
  const fire = (promise: Promise<unknown>) => {
    promise.catch(() => undefined);
  };

  /* --- Build command groups ------------------------------------------------ */
  const q = trimmedQuery.toLowerCase();
  const matches = (text?: string) => !q || (text ? text.toLowerCase().includes(q) : false);

  const navItems: CommandItem[] = [
    { id: 'nav-home', label: 'Accueil', icon: <Home />, run: () => go(ROUTES.home) },
    { id: 'nav-search', label: 'Rechercher', icon: <Search />, run: () => go(ROUTES.search) },
    { id: 'nav-library', label: 'Bibliothèque', icon: <Library />, run: () => go(ROUTES.library) },
    { id: 'nav-settings', label: 'Paramètres', icon: <Settings />, run: () => go(ROUTES.settings) },
  ].filter((item) => matches(item.label));

  const track = playback?.item ?? null;
  const saved = track ? isTrackSaved(track.id) : false;

  const playbackItems: CommandItem[] = [
    {
      id: 'pb-toggle',
      label: isPlaying ? 'Mettre la lecture en pause' : 'Lancer la lecture',
      hint: 'Espace',
      icon: isPlaying ? <Pause /> : <Play />,
      run: () =>
        runThenClose(() => fire(isPlaying ? controls.pause() : controls.play())),
    },
    {
      id: 'pb-next',
      label: 'Piste suivante',
      icon: <SkipForward />,
      run: () => runThenClose(() => fire(controls.next())),
    },
    {
      id: 'pb-previous',
      label: 'Piste précédente',
      icon: <SkipBack />,
      run: () => runThenClose(() => fire(controls.previous())),
    },
  ];

  if (track) {
    playbackItems.push({
      id: 'pb-like',
      label: saved ? 'Retirer des titres likés' : 'Ajouter aux titres likés',
      hint: track.name,
      icon: <Heart fill={saved ? 'currentColor' : 'none'} />,
      run: () => runThenClose(() => toggleSavedTrack(track.id)),
    });
  }

  const modeItems: CommandItem[] = [
    {
      id: 'mode-light',
      label: 'Mode clair',
      icon: <Sun />,
      run: () => runThenClose(() => setColorMode('light')),
    },
    {
      id: 'mode-dark',
      label: 'Mode sombre',
      icon: <Moon />,
      run: () => runThenClose(() => setColorMode('dark')),
    },
    {
      id: 'mode-system',
      label: 'Mode système',
      icon: <MonitorSmartphone />,
      run: () => runThenClose(() => setColorMode('system')),
    },
  ];

  const presetItems: CommandItem[] = presets.map((preset) => {
    const resolved = createTheme(preset);
    return {
      id: `preset-${preset.id}`,
      label: `Thème : ${preset.name}`,
      hint: resolved.colorMode === 'light' ? 'Clair' : 'Foncé',
      swatch: resolved.tokens.colors.accent,
      run: () => runThenClose(() => applyPreset(preset.id)),
    };
  });

  const resultItems: CommandItem[] = [];
  if (searchQuery && search.data) {
    search.data.tracks?.items.forEach((item) => {
      resultItems.push({
        id: `tr-${item.id}`,
        label: item.name,
        hint: `${item.artists.map((artist) => artist.name).join(', ')} · Titre`,
        icon: <Music />,
        run: () => go(buildPath.album(item.album.id)),
      });
    });
    search.data.albums?.items.forEach((item) => {
      resultItems.push({
        id: `al-${item.id}`,
        label: item.name,
        hint: `${item.artists.map((artist) => artist.name).join(', ')} · Album`,
        icon: <Disc3 />,
        run: () => go(buildPath.album(item.id)),
      });
    });
    search.data.artists?.items.forEach((item) => {
      resultItems.push({
        id: `ar-${item.id}`,
        label: item.name,
        hint: 'Artiste',
        icon: <User />,
        run: () => go(buildPath.artist(item.id)),
      });
    });
    search.data.playlists?.items.forEach((item) => {
      resultItems.push({
        id: `pl-${item.id}`,
        label: item.name,
        hint: `${item.owner?.display_name ?? 'Playlist'} · Playlist`,
        icon: <ListMusic />,
        run: () => go(buildPath.playlist(item.id)),
      });
    });
  }

  const groups: CommandGroup[] = [
    { id: 'nav', title: 'Navigation', items: navItems },
    { id: 'playback', title: 'Lecture', items: playbackItems.filter((item) => matches(item.label) || matches(item.hint)) },
    { id: 'theme', title: 'Apparence', items: [...modeItems, ...presetItems].filter((item) => matches(item.label)) },
    { id: 'results', title: 'Résultats Spotify', items: resultItems },
  ].filter((group) => group.items.length > 0);

  const flat = groups.flatMap((group) => group.items);
  const indexMap = new Map(flat.map((item, index) => [item.id, index]));

  /* --- Effects ------------------------------------------------------------- */
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  useEffect(() => {
    setActiveIndex((index) => Math.min(index, Math.max(0, flat.length - 1)));
  }, [flat.length]);

  useEffect(() => {
    const node = listRef.current?.querySelector<HTMLElement>(`[data-index="${activeIndex}"]`);
    node?.scrollIntoView({ block: 'nearest' });
  }, [activeIndex]);

  useEffect(() => {
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    const onDocumentKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', onDocumentKey);
    return () => {
      document.body.style.overflow = previousOverflow;
      document.removeEventListener('keydown', onDocumentKey);
    };
  }, [onClose]);

  const onKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (event.key === 'Escape') {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.key === 'Tab') {
      event.preventDefault();
      return;
    }
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setActiveIndex((index) => (flat.length ? (index + 1) % flat.length : 0));
      return;
    }
    if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActiveIndex((index) => (flat.length ? (index - 1 + flat.length) % flat.length : 0));
      return;
    }
    if (event.key === 'Enter') {
      event.preventDefault();
      flat[activeIndex]?.run();
    }
  };

  return createPortal(
    <motion.div
      className={styles.scrim}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.15, ease: 'easeOut' }}
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <motion.div
        className={styles.panel}
        role="dialog"
        aria-modal="true"
        aria-label="Palette de commandes"
        initial={{ opacity: 0, y: -10, scale: 0.98 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: -10, scale: 0.98 }}
        transition={{ duration: 0.18, ease: [0.3, 0, 0, 1] }}
        onKeyDown={onKeyDown}
      >
        <div className={styles.searchRow}>
          <Search className={styles.searchIcon} aria-hidden />
          <input
            ref={inputRef}
            className={styles.input}
            type="text"
            role="combobox"
            aria-expanded="true"
            aria-controls={listId}
            aria-activedescendant={flat.length ? optionId(activeIndex) : undefined}
            aria-autocomplete="list"
            autoComplete="off"
            spellCheck={false}
            placeholder="Rechercher une commande ou du contenu…"
            value={query}
            onChange={(event) => setQuery(event.currentTarget.value)}
          />
          {search.isFetching ? <Spinner size="sm" /> : null}
        </div>

        <div ref={listRef} className={styles.list} role="listbox" id={listId} aria-label="Commandes">
          {groups.length === 0 ? (
            <div className={styles.empty}>Aucun résultat pour « {trimmedQuery} »</div>
          ) : (
            groups.map((group) => (
              <div key={group.id} className={styles.group} role="group" aria-label={group.title}>
                <div className={styles.groupTitle}>{group.title}</div>
                {group.items.map((item) => {
                  const index = indexMap.get(item.id) ?? 0;
                  const active = index === activeIndex;
                  return (
                    <div
                      key={item.id}
                      id={optionId(index)}
                      data-index={index}
                      role="option"
                      aria-selected={active}
                      className={[styles.item, active ? styles.itemActive : ''].filter(Boolean).join(' ')}
                      onMouseEnter={() => setActiveIndex(index)}
                      onMouseDown={(event) => event.preventDefault()}
                      onClick={() => item.run()}
                    >
                      {item.swatch ? (
                        <span
                          className={styles.swatch}
                          style={{ backgroundColor: item.swatch }}
                          aria-hidden
                        />
                      ) : (
                        <span className={styles.itemIcon} aria-hidden>
                          {item.icon}
                        </span>
                      )}
                      <span className={styles.itemLabel}>{item.label}</span>
                      {item.hint ? <span className={styles.itemHint}>{item.hint}</span> : null}
                    </div>
                  );
                })}
              </div>
            ))
          )}
        </div>

        <div className={styles.footer}>
          <span className={styles.footerHint}>
            <Kbd>↑</Kbd>
            <Kbd>↓</Kbd> naviguer
          </span>
          <span className={styles.footerHint}>
            <Kbd>↵</Kbd> ouvrir
          </span>
          <span className={styles.footerHint}>
            <Kbd>Échap</Kbd> fermer
          </span>
        </div>
      </motion.div>
    </motion.div>,
    document.body,
  );
}
