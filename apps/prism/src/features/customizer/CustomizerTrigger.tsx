import { Palette } from 'lucide-react';
import { useState } from 'react';

import { IconButton, type ButtonSize, type ButtonVariant } from '@/components/ui';

import { CustomizerPanel } from './CustomizerPanel';

export interface CustomizerTriggerProps {
  className?: string;
  size?: ButtonSize;
  variant?: ButtonVariant;
  /** Accessible name / tooltip for the button. */
  label?: string;
}

/**
 * Self-contained palette button that opens the customizer drawer.
 * Drop it anywhere (e.g. the topbar actions slot) — it owns its open state.
 */
export function CustomizerTrigger({
  className,
  size = 'md',
  variant = 'secondary',
  label = 'Personnaliser le thème',
}: CustomizerTriggerProps) {
  const [open, setOpen] = useState(false);

  return (
    <>
      <IconButton
        label={label}
        className={className}
        size={size}
        variant={variant}
        aria-haspopup="dialog"
        aria-expanded={open}
        onClick={() => setOpen(true)}
      >
        <Palette />
      </IconButton>
      <CustomizerPanel open={open} onClose={() => setOpen(false)} />
    </>
  );
}
