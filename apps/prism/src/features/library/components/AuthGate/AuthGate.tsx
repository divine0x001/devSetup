import type { ReactNode } from 'react';

import { useAuth } from '@/hooks';

import { LoginView } from '../LoginView/LoginView';

export interface AuthGateProps {
  children: ReactNode;
  /** Rendered while the session is being restored. */
  fallback?: ReactNode;
}

/**
 * Renders `children` only for authenticated users; otherwise shows the sign-in
 * view. While the auth status is still resolving it renders `fallback`.
 */
export function AuthGate({ children, fallback = null }: AuthGateProps) {
  const { isAuthenticated, status } = useAuth();

  if (isAuthenticated) return <>{children}</>;
  if (status === 'authenticating') return <>{fallback}</>;
  return <LoginView />;
}
