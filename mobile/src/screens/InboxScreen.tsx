import { useEffect, useState, useCallback } from 'react';
import styles from './InboxScreen.module.css';
import { MailRow } from '../components/MailRow';
import { useAppStore } from '../store';
import { api } from '../api/client';
import type { Account, Folder, Message } from '../types';

function getInbox(folders: Folder[]): Folder | null {
  return (
    folders.find((f) => f.specialUse === 'Inbox') ??
    folders.find((f) => f.name.toLowerCase() === 'inbox') ??
    null
  );
}

export function InboxScreen() {
  const { accounts, setAccounts, folders, setFolders, selectedFolderId, setSelectedFolder, setSelectedMessage, theme } = useAppStore();
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [activeFolder, setActiveFolder] = useState<Folder | null>(null);

  // Load accounts + folders on mount
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
    const folder = folders.find((f) => f.id === selectedFolderId) ?? null;
    setActiveFolder(folder);
    loadMessages(selectedFolderId);
  }, [selectedFolderId, folders, loadMessages]);

  const handleRefresh = async () => {
    if (!selectedFolderId || refreshing) return;
    setRefreshing(true);
    await loadMessages(selectedFolderId);
    setRefreshing(false);
  };

  const unread = messages.filter((m) => !m.isRead).length;

  return (
    <div className={styles.screen}>
      {/* Header */}
      <header className={styles.header}>
        <div className={styles.headerLeft}>
          <span className={styles.appName}>rsMail</span>
          {unread > 0 && <span className={styles.badge}>{unread}</span>}
        </div>
        <div className={styles.headerRight}>
          <button className={styles.headerBtn} onClick={handleRefresh} aria-label="Refresh">
            <img
              src={theme === 'dark' ? '/icons/refresh-dark.png' : '/icons/refresh-light.png'}
              alt=""
              width="20" height="20"
            />
          </button>
        </div>
      </header>

      {/* Folder chips */}
      <div className={styles.folderBar}>
        {folders.filter((f) => f.specialUse || ['inbox','sent','drafts','trash','spam'].includes(f.name.toLowerCase())).slice(0, 8).map((f) => (
          <button
            key={f.id}
            className={`${styles.chip} ${selectedFolderId === f.id ? styles.chipActive : ''}`}
            onClick={() => setSelectedFolder(f.id)}
          >
            {f.name}
            {f.unreadCount > 0 && <span className={styles.chipCount}>{f.unreadCount}</span>}
          </button>
        ))}
      </div>

      {/* Message list */}
      <div className={`${styles.list} scroll`}>
        {loading && (
          <div className={styles.empty}>Loading…</div>
        )}
        {!loading && messages.length === 0 && (
          <div className={styles.empty}>No messages</div>
        )}
        {!loading && messages.map((msg) => (
          <MailRow
            key={msg.id}
            message={msg}
            onClick={() => setSelectedMessage(msg)}
          />
        ))}
        <div className={styles.listPad} />
      </div>
    </div>
  );
}
