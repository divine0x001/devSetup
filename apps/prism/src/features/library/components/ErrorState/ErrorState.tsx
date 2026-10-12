import { AlertTriangle, RefreshCw, SearchX } from 'lucide-react';

import { Button } from '@/components/ui';

import { cn } from '../../lib/utils';
import styles from './ErrorState.module.css';

export interface ErrorStateProps {
  title?: string;
  description?: string;
  /** HTTP status when known — a 404 renders a dedicated "not found" copy. */
  status?: number;
  onRetry?: () => void;
  className?: string;
}

/** Error panel with an optional retry action; special-cases 404s. */
export function ErrorState({
  title,
  description,
  status,
  onRetry,
  className,
}: ErrorStateProps) {
  const notFound = status === 404;
  const Icon = notFound ? SearchX : AlertTriangle;

  const resolvedTitle = title ?? (notFound ? 'Introuvable' : 'Une erreur est survenue');
  const resolvedDescription =
    description ??
    (notFound
      ? "Ce contenu n'existe pas ou n'est plus disponible."
      : 'Impossible de charger ce contenu pour le moment.');

  return (
    <div className={cn(styles.root, className)} role="alert">
      <span className={styles.icon} aria-hidden>
        <Icon />
      </span>
      <h2 className={styles.title}>{resolvedTitle}</h2>
      <p className={styles.description}>{resolvedDescription}</p>
      {onRetry && !notFound ? (
        <Button
          variant="secondary"
          size="sm"
          iconLeft={<RefreshCw />}
          onClick={onRetry}
        >
          Réessayer
        </Button>
      ) : null}
    </div>
  );
}
