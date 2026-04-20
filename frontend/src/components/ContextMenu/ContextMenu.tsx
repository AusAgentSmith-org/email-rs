import React, { useCallback, useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import styles from './ContextMenu.module.css';

export type ContextMenuItem =
  | { label: string; action: () => void; disabled?: boolean }
  | { separator: true };

interface ContextMenuState {
  x: number;
  y: number;
  items: ContextMenuItem[];
}

interface ContextMenuProps {
  state: ContextMenuState | null;
  onClose: () => void;
}

function ContextMenuPopup({ state, onClose }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);

  // Clamp position to viewport
  const [pos, setPos] = useState({ x: state?.x ?? 0, y: state?.y ?? 0 });

  useEffect(() => {
    if (!state || !ref.current) return;
    const menu = ref.current;
    const { innerWidth: vw, innerHeight: vh } = window;
    const rect = menu.getBoundingClientRect();
    const x = state.x + rect.width > vw ? Math.max(0, vw - rect.width - 4) : state.x;
    const y = state.y + rect.height > vh ? Math.max(0, vh - rect.height - 4) : state.y;
    setPos({ x, y });
  }, [state]);

  useEffect(() => {
    if (!state) return;
    const onMouseDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('mousedown', onMouseDown);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onMouseDown);
      document.removeEventListener('keydown', onKey);
    };
  }, [state, onClose]);

  if (!state) return null;

  return createPortal(
    <div
      ref={ref}
      className={styles.menu}
      style={{ left: pos.x, top: pos.y }}
      role="menu"
    >
      {state.items.map((item, i) => {
        if ('separator' in item) {
          return <div key={i} className={styles.separator} role="separator" />;
        }
        return (
          <button
            key={i}
            type="button"
            className={styles.item}
            role="menuitem"
            disabled={item.disabled}
            onClick={() => {
              item.action();
              onClose();
            }}
          >
            {item.label}
          </button>
        );
      })}
    </div>,
    document.body,
  );
}

export interface UseContextMenuReturn {
  contextMenu: React.ReactElement | null;
  openContextMenu: (e: React.MouseEvent, items: ContextMenuItem[]) => void;
}

export function useContextMenu(): UseContextMenuReturn {
  const [state, setState] = useState<ContextMenuState | null>(null);

  const onClose = useCallback(() => setState(null), []);

  const openContextMenu = useCallback((e: React.MouseEvent, items: ContextMenuItem[]) => {
    e.preventDefault();
    setState({ x: e.clientX, y: e.clientY, items });
  }, []);

  const contextMenu = (
    <ContextMenuPopup state={state} onClose={onClose} />
  );

  return { contextMenu, openContextMenu };
}
