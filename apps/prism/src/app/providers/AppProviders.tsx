import type { ReactNode } from 'react';
import { BrowserRouter } from 'react-router-dom';

import { AppBootstrap } from './AppBootstrap';
import { QueryProvider } from './QueryProvider';
import { ThemeProvider } from './ThemeProvider';

export interface AppProvidersProps {
  children: ReactNode;
}

/**
 * Single composition root for every cross-cutting provider.
 * Order matters: Query -> Theme -> Router -> Bootstrap.
 */
export function AppProviders({ children }: AppProvidersProps) {
  return (
    <QueryProvider>
      <ThemeProvider>
        <BrowserRouter>
          <AppBootstrap />
          {children}
        </BrowserRouter>
      </ThemeProvider>
    </QueryProvider>
  );
}