import { Music4, Sparkles } from 'lucide-react';

import { Button } from '@/components/ui';
import { useAuth } from '@/hooks';
import { isSpotifyConfigured } from '@/lib/spotify/config';

import { cn } from '../../lib/utils';
import styles from './LoginView.module.css';

export interface LoginViewProps {
  title?: string;
  description?: string;
  className?: string;
}

/**
 * Full-page sign-in prompt. Degrades gracefully when the app has no Spotify
 * Client ID configured (shows setup instructions instead of a dead button).
 */
export function LoginView({ title, description, className }: LoginViewProps) {
  const { login } = useAuth();
  const configured = isSpotifyConfigured();

  return (
    <div className={cn(styles.root, className)}>
      <div className={styles.card}>
        <span className={styles.logo} aria-hidden>
          <Music4 />
        </span>
        <h1 className={styles.title}>{title ?? 'Bienvenue sur Prism'}</h1>
        <p className={styles.description}>
          {description ??
            'Connectez votre compte Spotify pour explorer votre bibliothèque, vos playlists et lancer la lecture.'}
        </p>

        {configured ? (
          <Button
            size="lg"
            iconLeft={<Sparkles />}
            onClick={() => {
              void login();
            }}
          >
            Se connecter avec Spotify
          </Button>
        ) : (
          <div className={styles.notice} role="alert">
            <strong>Prism n’est pas encore configuré.</strong>
            <span>
              Définissez <code>VITE_SPOTIFY_CLIENT_ID</code> dans <code>.env.local</code> puis
              redémarrez le serveur de développement.
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
