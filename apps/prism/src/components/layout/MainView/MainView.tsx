import type { ReactNode } from 'react';
import { Outlet } from 'react-router-dom';

import styles from './MainView.module.css';

export interface MainViewProps {
  /** Routed content. Falls back to the router `<Outlet/>` when omitted. */
  children?: ReactNode;
  className?: string;
  /** Constrain content to `--prism-content-max-width`. Defaults to true. */
  constrainWidth?: boolean;
}

/**
 * Scroll container for routed pages. Sits under the topbar and above the
 * now-playing bar, and optionally centers its content on very wide screens.
 */
export function MainView({ children, className, constrainWidth = true }: MainViewProps) {
  return (
    <div className={[styles.root, className ?? ''].filter(Boolean).join(' ')}>
      <div className={[styles.inner, constrainWidth ? styles.constrained : ''].filter(Boolean).join(' ')}>
        {children ?? <Outlet />}
      </div>
    </div>
  );
}
