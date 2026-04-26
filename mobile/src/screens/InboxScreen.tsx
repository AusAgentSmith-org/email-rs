import { useEffect, useState, useCallback } from 'react';
import styles from './InboxScreen.module.css';
import { MailRow } from '../components/MailRow';
import { FolderDrawer } from '../components/FolderDrawer';
import { useAppStore } from '../store';
import { api } from '../api/client';
import type { Account, Folder, Message } from '../types';

function getInbox(folders: Folder[]): Folder | null {
  return (
    folders.find((f) => f.specialUse?.toLowerCase() === 'inbox') ??
    folders.find((f) => f.name.toLowerCase() === 'inbox') ??
    null
  );
}

export function InboxScreen() {
  const { accounts, setAccounts, folders, setFolders, selectedFolderId, setSelectedFolder, setSelectedMessage, theme } = useAppStore();
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [drawerOpen, setDrawerOpen] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const accs = await api.accounts() as Account[];
        setAccounts(accs);
        if (accs.length === 0) { setLoading(false); return; }
        const allFolders = (await Promise.all(accs.map((a) => api.folders(a.id) as Promise<Folder[]>))).flat();
        setFolders(allFolders);
        const inbox = getInbox(allFolders);
        if (inbox) setSelectedFolder(inbox.id);
        else setLoading(false);
      } catch { setLoading(false); }
    })();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const loadMessages = useCallback(async (folderId: string) => {
    setLoading(true);
    try {
      const msgs = await api.messages(folderId) as Message[];
      setMessages(msgs);
    } catch { /* ignore */ }
    setLoading(false);
  }, []);

  useEffect(() => {
    if (!selectedFolderId) return;
    loadMessages(selectedFolderId);
  }, [selectedFolderId, loadMessages]);

  const handleRefresh = async () => {
    if (!selectedFolderId || refreshing) return;
    setRefreshing(true);
    await loadMessages(selectedFolderId);
    setRefreshing(false);
  };

  const activeFolder = folders.find((f) => f.id === selectedFolderId);
  const unread = messages.filter((m) => !m.isRead).length;

  return (
    <div className={styles.screen}>
      {/* Header */}
      <header className={styles.header}>
        <button className={styles.folderBtn} onClick={() => setDrawerOpen(true)} aria-label="Browse folders">
          <svg viewBox="0 0 20 16" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" width="20" height="16">
            <path d="M1 3h18M1 8h18M1 13h18" />
          </svg>
        </button>

        <div className={styles.headerCenter}>
          <span className={styles.folderName}>{activeFolder?.name ?? 'Mail'}</span>
          {unread > 0 && <span className={styles.badge}>{unread} unread</span>}
        </div>

        <button className={styles.headerBtn} onClick={handleRefresh} aria-label="Refresh">
          <img
            src={theme === 'dark' ? '/icons/refresh-dark.png' : '/icons/refresh-light.png'}
            alt="" width="20" height="20"
          />
        </button>
      </header>

      {/* Message list */}
      <div className={`${styles.list} scroll`}>
        {loading && <div className={styles.empty}>Loading…</div>}
        {!loading && messages.length === 0 && <div className={styles.empty}>No messages</div>}
        {!loading && messages.map((msg) => (
          <MailRow key={msg.id} message={msg} onClick={() => setSelectedMessage(msg)} />
        ))}
        <div className={styles.listPad} />
      </div>

      {/* Folder drawer */}
      {drawerOpen && (
        <FolderDrawer
          folders={folders}
          selectedId={selectedFolderId}
          onSelect={setSelectedFolder}
          onClose={() => setDrawerOpen(false)}
        />
      )}
    </div>
  );
}
