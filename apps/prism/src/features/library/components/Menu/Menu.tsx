import { MoreHorizontal } from 'lucide-react';
import {
  useEffect,
  useId,
  useRef,
  useState,
  type KeyboardEvent,
  type ReactNode,
} from 'react';

import { cn } from '../../lib/utils';
import styles from './Menu.module.css';

export interface MenuItem {
  id: string;
  label: string;
  icon?: ReactNode;
  onSelect: () => void;
  danger?: boolean;
  disabled?: boolean;
}

export interface MenuProps {
  /** Accessible name for the trigger button. */
  label: string;
  items: MenuItem[];
  /** Custom trigger content; defaults to a "more" icon. */
  trigger?: ReactNode;
  align?: 'start' | 'end';
  className?: string;
}

/**
 * Accessible dropdown menu: closes on Escape / outside click, supports arrow
 * navigation and Home/End, and restores focus to the trigger on close.
 */
export function Menu({ label, items, trigger, align = 'end', className }: MenuProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const menuId = useId();

  useEffect(() => {
    if (!open) return;

    const onPointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    };
    const onKeyDown = (event: globalThis.KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.stopPropagation();
        setOpen(false);
        triggerRef.current?.focus();
      }
    };

    document.addEventListener('mousedown', onPointerDown);
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('mousedown', onPointerDown);
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const first = menuRef.current?.querySelector<HTMLButtonElement>('[role="menuitem"]:not([disabled])');
    first?.focus();
  }, [open]);

  const onMenuKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp' && event.key !== 'Home' && event.key !== 'End') {
      return;
    }
    event.preventDefault();
    const nodes = Array.from(
      menuRef.current?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]:not([disabled])') ?? [],
    );
    if (nodes.length === 0) return;
    const current = nodes.indexOf(document.activeElement as HTMLButtonElement);
    let nextIndex = 0;
    if (event.key === 'ArrowDown') nextIndex = current < 0 ? 0 : (current + 1) % nodes.length;
    else if (event.key === 'ArrowUp') nextIndex = current <= 0 ? nodes.length - 1 : current - 1;
    else if (event.key === 'End') nextIndex = nodes.length - 1;
    nodes[nextIndex]?.focus();
  };

  return (
    <div ref={rootRef} className={cn(styles.root, className)}>
      <button
        ref={triggerRef}
        type="button"
        className={styles.trigger}
        aria-label={label}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-controls={open ? menuId : undefined}
        onClick={() => setOpen((value) => !value)}
      >
        {trigger ?? <MoreHorizontal aria-hidden />}
      </button>

      {open ? (
        <div
          ref={menuRef}
          id={menuId}
          role="menu"
          className={cn(styles.menu, align === 'start' ? styles.start : styles.end)}
          onKeyDown={onMenuKeyDown}
        >
          {items.map((item) => (
            <button
              key={item.id}
              type="button"
              role="menuitem"
              className={cn(styles.item, item.danger ? styles.danger : '')}
              disabled={item.disabled}
              onClick={() => {
                setOpen(false);
                item.onSelect();
              }}
            >
              {item.icon ? (
                <span className={styles.itemIcon} aria-hidden>
                  {item.icon}
                </span>
              ) : null}
              <span className={styles.itemLabel}>{item.label}</span>
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
