import { X } from 'lucide-react';
import { useCallback, useRef } from 'react';
import { createPortal } from 'react-dom';

import { IconButton } from '@/components/ui';

import styles from './Customizer.module.css';
import { CustomizerBody } from './CustomizerBody';
import { useDrawerPresence } from './useDrawerPresence';
import { useFocusTrap } from './useFocusTrap';

export interface CustomizerPanelProps {
  open: boolean;
  onClose: () => void;
}

/**
 * Right-hand slide-over drawer that hosts the full customizer.
 *  - animates both in and out while unmounting completely when closed,
 *  - traps focus, closes on Escape and scrim click,
 *  - locks body scroll while open.
 */
export function CustomizerPanel({ open, onClose }: CustomizerPanelProps) {
  const { mounted, entered } = useDrawerPresence(open);
  const panelRef = useRef<HTMLElement>(null);
  const handleEscape = useCallback(() => onClose(), [onClose]);

  useFocusTrap(panelRef, mounted, handleEscape);

  if (!mounted) return null;

  return createPortal(
    <div
      className={styles.overlay}
      data-entered={entered || undefined}
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <aside
        ref={panelRef}
        className={styles.panel}
        data-entered={entered || undefined}
        role="dialog"
        aria-modal="true"
        aria-label="Panneau de personnalisation"
        tabIndex={-1}
      >
        <header className={styles.panelHeader}>
          <div className={styles.panelHeading}>
            <h2 className={styles.panelTitle}>Personnalisation</h2>
            <p className={styles.panelSubtitle}>
              Aperçu en direct — chaque réglage est instantané.
            </p>
          </div>
          <IconButton label="Fermer le panneau" onClick={onClose}>
            <X />
          </IconButton>
        </header>

        <div className={styles.panelBody}>
          <CustomizerBody variant="drawer" />
        </div>
      </aside>
    </div>,
    document.body,
  );
}
