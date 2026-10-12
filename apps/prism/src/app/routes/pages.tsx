import { motion } from 'framer-motion';
import { Music2 } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';

import { Button } from '@/components/ui';
import { useAuth } from '@/hooks';
import { handleAuthCallback } from '@/lib/spotify/auth';
import { spotifyApi } from '@/lib/spotify/client';
import { useAuthStore } from '@/store/authStore';
import { CustomizerPage } from '@/features/customizer';
import {
  AlbumView,
  ArtistView,
  HomeView,
  LibraryView,
  PlaylistView,
  SearchView,
} from '@/features/library';

import { buildPath } from './routes';
import styles from './PlaceholderPage.module.css';

/* -------------------------------------------------------------------------- */
/* Shared placeholder (kept for the 404 page)                                  */
/* -------------------------------------------------------------------------- */

interface PlaceholderPageProps {
  title: string;
  description?: string;
}

export function PlaceholderPage({ title, description }: PlaceholderPageProps) {
  return (
    <motion.main
      className={styles.root}
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.22, ease: [0.2, 0, 0, 1] }}
    >
      <Music2 className={styles.icon} aria-hidden strokeWidth={1.5} />
      <h1 className={styles.title}>{title}</h1>
      {description ? <p className={styles.description}>{description}</p> : null}
    </motion.main>
  );
}

/* -------------------------------------------------------------------------- */
/* Product pages (W5 views)                                                    */
/* -------------------------------------------------------------------------- */

export function HomePage() {
  return <HomeView />;
}

export function SearchPage() {
  return <SearchView />;
}

export function LibraryPage() {
  return <LibraryView />;
}

export function PlaylistPage() {
  const { id } = useParams<{ id: string }>();
  if (!id) return <NotFoundPage />;
  return <PlaylistView id={id} />;
}

export function AlbumPage() {
  const { id } = useParams<{ id: string }>();
  if (!id) return <NotFoundPage />;
  return <AlbumView id={id} />;
}

export function ArtistPage() {
  const { id } = useParams<{ id: string }>();
  if (!id) return <NotFoundPage />;
  return <ArtistView id={id} />;
}

export function SettingsPage() {
  return <CustomizerPage />;
}

/* -------------------------------------------------------------------------- */
/* Auth callback (PKCE redirect target, rendered full-screen, outside shell)   */
/* -------------------------------------------------------------------------- */

type CallbackState =
  | { phase: 'processing' }
  | { phase: 'error'; message: string }
  | { phase: 'no-code' };

/**
 * Completes the Spotify Authorization Code + PKCE flow.
 *
 * `handleAuthCallback()` validates state, exchanges the code and persists the
 * tokens (localStorage). On success we mirror the session into the auth store,
 * fetch the profile, then redirect to the pre-login path. A ref guard makes the
 * flow idempotent under React StrictMode's double effect invocation.
 */
export function AuthCallbackPage() {
  const navigate = useNavigate();
  const { login } = useAuth();
  const startedRef = useRef(false);
  const [state, setState] = useState<CallbackState>({ phase: 'processing' });

  useEffect(() => {
    if (startedRef.current) return;
    startedRef.current = true;

    void (async () => {
      try {
        const result = await handleAuthCallback();
        if (!result) {
          setState({ phase: 'no-code' });
          return;
        }

        const { tokens, returnPath } = result;
        const auth = useAuthStore.getState();
        auth.setTokens(tokens);
        auth.setStatus('authenticated');

        try {
          const profile = await spotifyApi.getMe();
          useAuthStore.getState().setProfile(profile);
        } catch {
          // Non-fatal: the session is valid even if the profile fetch fails.
        }

        navigate(returnPath, { replace: true });
      } catch (error) {
        setState({
          phase: 'error',
          message:
            error instanceof Error
              ? error.message
              : 'Échec de la connexion Spotify.',
        });
      }
    })();
  }, [navigate]);

  if (state.phase === 'processing') {
    return (
      <PlaceholderPage
        title="Connexion en cours…"
        description="Finalisation de l’autorisation Spotify (PKCE)."
      />
    );
  }

  const message =
    state.phase === 'no-code'
      ? 'Aucun code d’autorisation reçu. Cette page est la cible du retour Spotify après connexion.'
      : state.message;

  return (
    <motion.main
      className={styles.root}
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.22, ease: [0.2, 0, 0, 1] }}
    >
      <Music2 className={styles.icon} aria-hidden strokeWidth={1.5} />
      <h1 className={styles.title}>Connexion impossible</h1>
      <p className={styles.description}>{message}</p>
      <div className={styles.actions}>
        <Button variant="secondary" onClick={() => navigate(buildPath.home())}>
          Retour à l’accueil
        </Button>
        <Button
          variant="primary"
          onClick={() => {
            void login();
          }}
        >
          Réessayer la connexion
        </Button>
      </div>
    </motion.main>
  );
}

/* -------------------------------------------------------------------------- */
/* 404                                                                         */
/* -------------------------------------------------------------------------- */

export function NotFoundPage() {
  return (
    <PlaceholderPage
      title="404"
      description="Cette page n’existe pas ou a été déplacée."
    />
  );
}