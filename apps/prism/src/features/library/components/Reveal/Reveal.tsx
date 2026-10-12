import { motion, useReducedMotion } from 'framer-motion';
import type { ReactNode } from 'react';

export interface RevealProps {
  children: ReactNode;
  /** Stagger delay in seconds. */
  delay?: number;
  className?: string;
}

/**
 * Fades + lifts its children into view once. Respects `prefers-reduced-motion`
 * by rendering a plain wrapper with no animation.
 */
export function Reveal({ children, delay = 0, className }: RevealProps) {
  const reduceMotion = useReducedMotion();

  if (reduceMotion) {
    return <div className={className}>{children}</div>;
  }

  return (
    <motion.div
      className={className}
      initial={{ opacity: 0, y: 14 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-40px' }}
      transition={{ duration: 0.34, ease: [0.2, 0, 0, 1], delay }}
    >
      {children}
    </motion.div>
  );
}
