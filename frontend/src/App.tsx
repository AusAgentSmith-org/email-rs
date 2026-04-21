import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Sidebar } from './components/Sidebar/Sidebar';
import { MessageList } from './components/MessageList/MessageList';
import { ReadingPane } from './components/ReadingPane/ReadingPane';
import { AccountSetup } from './components/AccountSetup/AccountSetup';
import { ComposeModal } from './components/Compose/ComposeModal';
import { SettingsModal } from './components/Settings/SettingsModal';
import { CommandPalette } from './components/CommandPalette/CommandPalette';
import { useAppStore } from './store';
import { applyDensity } from './utils/density';
import type { Account } from './types';

const SIDEBAR_W_KEY = 'email_sidebar_w';
const MSGLIST_W_KEY  = 'email_msglist_w';
const DEFAULT_SIDEBAR_W = 200;
const DEFAULT_MSGLIST_W = 280;
const MIN_W = 140;
const MAX_SIDEBAR_W = 480;
const MAX_MSGLIST_W = 600;

function storedWidth(key: string, def: number): number {
  try {
    const v = localStorage.getItem(key);
    return v ? Math.max(MIN_W, Number(v)) : def;
  } catch { return def; }
}

function usePanelResize(key: string, def: number, min: number, max: number) {
  const [width, setWidth] = useState(() => storedWidth(key, def));
  const widthRef = useRef(width);
  useEffect(() => { widthRef.current = width; }, [width]);

  const onMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startW = widthRef.current;

    const onMove = (ev: MouseEvent) => {
      setWidth(Math.min(max, Math.max(min, startW + (ev.clientX - startX))));
    };
    const onUp = (ev: MouseEvent) => {
      const final = Math.min(max, Math.max(min, startW + (ev.clientX - startX)));
      setWidth(final);
      try { localStorage.setItem(key, String(Math.round(final))); } catch { /* */ }
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };

    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  }, [key, min, max]);

  return { width, onMouseDown };
}

const handleStyle: React.CSSProperties = {
  width: 4,
  flexShrink: 0,
  cursor: 'col-resize',
  background: 'transparent',
  position: 'relative',
  zIndex: 10,
  transition: 'background 0.15s',
};

export function App() {
  const { theme, densityLevel, accounts, setAccounts, compose, settingsOpen, openPalette, closePalette, paletteOpen } = useAppStore();
  const [accountsLoaded, setAccountsLoaded] = useState(false);

  const sidebar = usePanelResize(SIDEBAR_W_KEY,  DEFAULT_SIDEBAR_W, MIN_W, MAX_SIDEBAR_W);
  const msglist = usePanelResize(MSGLIST_W_KEY,  DEFAULT_MSGLIST_W, MIN_W, MAX_MSGLIST_W);

  useEffect(() => { document.documentElement.setAttribute('data-theme', theme); }, [theme]);
  useEffect(() => { applyDensity(densityLevel); }, [densityLevel]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        paletteOpen ? closePalette() : openPalette();
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [paletteOpen, openPalette, closePalette]);

  const fetchAccounts = () => {
    fetch('/api/v1/accounts')
      .then((res) => res.json() as Promise<Account[]>)
      .then((data) => { setAccounts(data); setAccountsLoaded(true); })
      .catch(() => { setAccounts([]); setAccountsLoaded(true); });
  };

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    if (params.get('oauth') === 'success') window.history.replaceState(null, '', window.location.pathname);
    fetchAccounts();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (!accountsLoaded) return null;
  if (accounts.length === 0) return <AccountSetup onAccountAdded={fetchAccounts} />;

  return (
    <div style={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
        <div style={{ width: sidebar.width, flexShrink: 0, overflow: 'hidden', height: '100%' }}>
          <Sidebar onAccountAdded={fetchAccounts} />
        </div>

        <div
          style={handleStyle}
          onMouseDown={sidebar.onMouseDown}
          onMouseEnter={(e) => { (e.currentTarget as HTMLDivElement).style.background = 'var(--divider)'; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLDivElement).style.background = 'transparent'; }}
        />

        <div style={{ width: msglist.width, flexShrink: 0, overflow: 'hidden', height: '100%' }}>
          <MessageList />
        </div>

        <div
          style={handleStyle}
          onMouseDown={msglist.onMouseDown}
          onMouseEnter={(e) => { (e.currentTarget as HTMLDivElement).style.background = 'var(--divider)'; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLDivElement).style.background = 'transparent'; }}
        />

        <div style={{ flex: 1, overflow: 'hidden', height: '100%', minWidth: 0 }}>
          {compose ? <ComposeModal /> : <ReadingPane />}
        </div>

      {settingsOpen && <SettingsModal onAccountDeleted={fetchAccounts} />}
      <CommandPalette />
    </div>
  );
}
