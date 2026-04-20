import React, { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import styles from './Sidebar.module.css';
import { useAppStore } from '../../store';
import { useApi, useSyncEvents } from '../../hooks/useApi';
import { AddAccountModal } from '../AccountSetup/AddAccountModal';
import { AdvancedSearchModal } from '../AdvancedSearch/AdvancedSearchModal';
import { useContextMenu } from '../ContextMenu/ContextMenu';
import type { Account, Folder, Suggestion } from '../../types';

// ── SVG icons ────────────────────────────────────────────────────────────────

function InboxIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <rect x="1" y="3" width="14" height="10" rx="2" />
      <path d="M1 9h4l1.5 2h3L11 9h4" />
    </svg>
  );
}

function SentIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <path d="M14 2L2 7l5 2 2 5 5-12z" />
    </svg>
  );
}

function DraftIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <path d="M11 2H3a1 1 0 00-1 1v10a1 1 0 001 1h10a1 1 0 001-1V5l-3-3z" />
      <path d="M11 2v3h3M5 9h6M5 12h4" />
    </svg>
  );
}

function ArchiveIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <rect x="1" y="2" width="14" height="3" rx="1" />
      <path d="M2 5v8a1 1 0 001 1h10a1 1 0 001-1V5" />
      <path d="M6 9h4" />
    </svg>
  );
}

function TrashIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <path d="M2 4h12M6 4V3a1 1 0 011-1h2a1 1 0 011 1v1M5 4v9a1 1 0 001 1h4a1 1 0 001-1V4" />
    </svg>
  );
}

function SpamIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <circle cx="8" cy="8" r="6" />
      <path d="M8 5v3M8 10.5v.5" strokeLinecap="round" />
    </svg>
  );
}

function FolderIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <path d="M1 4a1 1 0 011-1h4l2 2h6a1 1 0 011 1v6a1 1 0 01-1 1H2a1 1 0 01-1-1V4z" />
    </svg>
  );
}

function ChevronIcon({ collapsed }: { collapsed: boolean }) {
  return (
    <svg
      viewBox="0 0 10 10"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      width="10"
      height="10"
      style={{ transform: collapsed ? 'rotate(-90deg)' : 'rotate(0deg)', transition: 'transform 0.15s', flexShrink: 0 }}
    >
      <path d="M2 3.5l3 3 3-3" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function SearchIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <circle cx="7" cy="7" r="4.5" />
      <path d="M10.5 10.5L14 14" strokeLinecap="round" />
    </svg>
  );
}

function TuneIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <path d="M2 4h12M4 8h8M6 12h4" strokeLinecap="round" />
    </svg>
  );
}

// Smart folder icons
function AllInboxIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <rect x="1" y="5" width="14" height="8" rx="1.5" />
      <path d="M1 11h3.5l1 1.5h3L9.5 11H13" />
      <path d="M4 5V4a1 1 0 011-1h6a1 1 0 011 1v1" />
    </svg>
  );
}

function UnreadIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <circle cx="11" cy="5" r="2.5" fill="var(--accent)" stroke="none" />
      <rect x="1" y="3" width="14" height="10" rx="2" />
      <path d="M1 9h4l1.5 2h3L11 9h4" />
    </svg>
  );
}

function FlaggedIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <path d="M3 2v12" strokeLinecap="round" />
      <path d="M3 2l9 3-9 4" strokeLinejoin="round" />
    </svg>
  );
}

function SnoozedIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
      <circle cx="8" cy="8.5" r="5.5" />
      <path d="M8 5.5v3l2 1.5" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M6 2h4M7 2l2-1.5" strokeLinecap="round" />
    </svg>
  );
}

// ── Helpers ──────────────────────────────────────────────────────────────────

function folderDepth(folder: Folder): number {
  const normalized = folder.fullPath.replace(/^\[Gmail\]\//, '');
  return normalized.split('/').length - 1;
}

function getFolderIcon(specialUse: Folder['specialUse']) {
  switch (specialUse) {
    case 'inbox':   return InboxIcon;
    case 'sent':    return SentIcon;
    case 'drafts':  return DraftIcon;
    case 'archive': return ArchiveIcon;
    case 'trash':   return TrashIcon;
    case 'spam':    return SpamIcon;
    default:        return FolderIcon;
  }
}

function getAccountColor(account: Account): string {
  if (account.color) return account.color;
  const hue = account.email.split('').reduce((n, c) => n + c.charCodeAt(0), 0) % 360;
  return `oklch(58% 0.16 ${hue})`;
}

function getAvatarStyle(name: string): { bg: string; initials: string } {
  const hue = name.split('').reduce((n, c) => n + c.charCodeAt(0), 0) % 360;
  return {
    bg: `oklch(62% 0.12 ${hue})`,
    initials: name.split(' ').map((w) => w[0]).join('').slice(0, 2).toUpperCase(),
  };
}

// ── Smart folders ─────────────────────────────────────────────────────────────

interface SmartFolder {
  id: string;
  label: string;
  Icon: (props: { className?: string }) => React.ReactElement;
}

const SMART_FOLDERS: SmartFolder[] = [
  { id: 'smart:all',     label: 'All Inboxes', Icon: AllInboxIcon },
  { id: 'smart:unread',  label: 'Unread',       Icon: UnreadIcon  },
  { id: 'smart:flagged', label: 'Flagged',      Icon: FlaggedIcon },
  { id: 'smart:snoozed', label: 'Snoozed',      Icon: SnoozedIcon },
];

// ── Per-account folders hook ──────────────────────────────────────────────────

interface AccountFolders {
  account: Account;
  folders: Folder[] | undefined;
}

// We fetch folders for each account using stable hook slots (hooks can't be in loops).
// MAX_ACCOUNTS caps how many accounts we support in the sidebar.
const MAX_ACCOUNTS = 8;

function useAllAccountFolders(accounts: Account[] | undefined): {
  accountFolders: AccountFolders[];
  allFolderData: (Folder[] | undefined)[];
  refetchAll: () => void;
} {
  const a = accounts ?? [];

  const r0 = useApi<Folder[]>(a[0] ? `/api/v1/accounts/${a[0].id}/folders` : '', { immediate: !!a[0] });
  const r1 = useApi<Folder[]>(a[1] ? `/api/v1/accounts/${a[1].id}/folders` : '', { immediate: !!a[1] });
  const r2 = useApi<Folder[]>(a[2] ? `/api/v1/accounts/${a[2].id}/folders` : '', { immediate: !!a[2] });
  const r3 = useApi<Folder[]>(a[3] ? `/api/v1/accounts/${a[3].id}/folders` : '', { immediate: !!a[3] });
  const r4 = useApi<Folder[]>(a[4] ? `/api/v1/accounts/${a[4].id}/folders` : '', { immediate: !!a[4] });
  const r5 = useApi<Folder[]>(a[5] ? `/api/v1/accounts/${a[5].id}/folders` : '', { immediate: !!a[5] });
  const r6 = useApi<Folder[]>(a[6] ? `/api/v1/accounts/${a[6].id}/folders` : '', { immediate: !!a[6] });
  const r7 = useApi<Folder[]>(a[7] ? `/api/v1/accounts/${a[7].id}/folders` : '', { immediate: !!a[7] });

  const dataSlots = [r0.data, r1.data, r2.data, r3.data, r4.data, r5.data, r6.data, r7.data];
  const refetchSlots = [r0.refetch, r1.refetch, r2.refetch, r3.refetch, r4.refetch, r5.refetch, r6.refetch, r7.refetch];

  const accountFolders: AccountFolders[] = a.slice(0, MAX_ACCOUNTS).map((account, i) => ({
    account,
    folders: dataSlots[i],
  }));

  const refetchAll = useCallback(() => {
    refetchSlots.slice(0, a.length).forEach((fn) => fn());
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [a.length, ...refetchSlots]);

  return { accountFolders, allFolderData: dataSlots, refetchAll };
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function formatSuggestDate(dateStr: string | null): string {
  if (!dateStr) return '';
  const d = new Date(dateStr);
  const now = new Date();
  const sameYear = d.getFullYear() === now.getFullYear();
  return d.toLocaleDateString(undefined, sameYear
    ? { month: 'short', day: 'numeric' }
    : { month: 'short', day: 'numeric', year: 'numeric' });
}

// ── Component ────────────────────────────────────────────────────────────────

interface SidebarProps {
  onAccountAdded: () => void;
}

export function Sidebar({ onAccountAdded }: SidebarProps) {
  const [showAddModal, setShowAddModal] = useState(false);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [activeSuggestion, setActiveSuggestion] = useState(-1);
  const [dropdownPos, setDropdownPos] = useState({ top: 0, left: 0, width: 0 });
  const searchInputRef = useRef<HTMLInputElement>(null);

  const { selectedFolderId, setSelectedFolder, setFolders, openCompose, searchQuery, setSearchQuery, conditionGroup, navigateToMessage } = useAppStore();
  const navRef = useRef<HTMLElement>(null);
  const { contextMenu, openContextMenu } = useContextMenu();

  const toggle = (key: string) => setCollapsed((c) => ({ ...c, [key]: !c[key] }));

  // Fetch autocomplete suggestions with 150ms debounce
  useEffect(() => {
    const q = searchQuery.trim();
    if (q.length < 2) {
      setSuggestions([]);
      setShowSuggestions(false);
      setActiveSuggestion(-1);
      return;
    }
    const t = setTimeout(async () => {
      try {
        const res = await fetch(`/api/v1/search/suggest?q=${encodeURIComponent(q)}`);
        if (res.ok) {
          const data = (await res.json()) as Suggestion[];
          setSuggestions(data);
          setShowSuggestions(data.length > 0);
          setActiveSuggestion(-1);
        }
      } catch { /* ignore */ }
    }, 150);
    return () => clearTimeout(t);
  }, [searchQuery]);

  // Recalculate dropdown position whenever it opens
  useLayoutEffect(() => {
    if (showSuggestions && searchInputRef.current) {
      const r = searchInputRef.current.getBoundingClientRect();
      setDropdownPos({ top: r.bottom + 4, left: r.left, width: Math.max(r.width, 340) });
    }
  }, [showSuggestions, suggestions.length]);

  const closeSuggestions = () => {
    setShowSuggestions(false);
    setActiveSuggestion(-1);
  };

  const handleSuggestionClick = (s: Suggestion) => {
    closeSuggestions();
    setSuggestions([]);
    navigateToMessage(s.folderId, s.id);
  };

  const handleSearchKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!showSuggestions || suggestions.length === 0) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActiveSuggestion((i) => Math.min(i + 1, suggestions.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActiveSuggestion((i) => Math.max(i - 1, -1));
    } else if (e.key === 'Enter' && activeSuggestion >= 0) {
      e.preventDefault();
      handleSuggestionClick(suggestions[activeSuggestion]);
    } else if (e.key === 'Escape') {
      closeSuggestions();
    }
  };

  const { data: accounts } = useApi<Account[]>('/api/v1/accounts');
  const { accountFolders, allFolderData, refetchAll } = useAllAccountFolders(accounts);

  const handleSync = useCallback(() => {
    refetchAll();
  }, [refetchAll]);

  const syncProgress = useSyncEvents(handleSync);

  // Merge all fetched folders into the store so MessageList can look up names.
  // Depend on the individual data-slot references so this only fires when a
  // fetch resolves — not on every render.
  useEffect(() => {
    const all: Folder[] = allFolderData.flatMap((d) => d ?? []);
    if (all.length > 0) {
      setFolders(all);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [...allFolderData, setFolders]);

  const specialOrder: Folder['specialUse'][] = ['inbox', 'sent', 'drafts', 'archive', 'trash', 'spam'];

  return (
    <aside className={styles.sidebar}>
      <button
        className={styles.compose}
        type="button"
        onClick={() => {
          const accountId = accounts?.[0]?.id;
          if (accountId) openCompose({ accountId, to: '', subject: '', mode: 'compose' });
        }}
      >
        Compose
      </button>

      {syncProgress && (
        <div className={styles.syncStrip}>
          <div className={styles.syncRow}>
            {syncProgress.syncing ? (
              <span className={styles.syncSpinner} />
            ) : (
              <svg className={styles.syncDone} viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M2 5l2.5 2.5L8 3" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            )}
            <span className={styles.syncText}>
              {syncProgress.syncing
                ? syncProgress.total > 0
                  ? `Syncing ${syncProgress.done}/${syncProgress.total} folders…`
                  : 'Syncing…'
                : 'Sync complete'}
            </span>
          </div>
          {syncProgress.syncing && syncProgress.total > 0 && (
            <div className={styles.syncBar}>
              <div
                className={styles.syncBarFill}
                style={{ width: `${Math.round((syncProgress.done / syncProgress.total) * 100)}%` }}
              />
            </div>
          )}
        </div>
      )}

      <div className={styles.searchWrap}>
        <SearchIcon className={styles.searchIcon} />
        <input
          ref={searchInputRef}
          className={styles.searchInput}
          type="text"
          placeholder="Search mail…"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={handleSearchKeyDown}
          onFocus={() => { if (suggestions.length > 0) setShowSuggestions(true); }}
          onBlur={() => setTimeout(closeSuggestions, 150)}
          autoComplete="off"
        />
        <button
          className={`${styles.filterBtn}${conditionGroup ? ` ${styles.filterBtnActive}` : ''}`}
          type="button"
          title="Advanced search"
          onClick={() => setAdvancedOpen(true)}
        >
          <TuneIcon />
        </button>
      </div>

      {showSuggestions && suggestions.length > 0 && (
        <div
          className={styles.suggestDropdown}
          style={{ position: 'fixed', top: dropdownPos.top, left: dropdownPos.left, width: dropdownPos.width }}
          onMouseDown={(e) => e.preventDefault()}
        >
          {suggestions.map((s, i) => (
            <div
              key={s.id}
              className={`${styles.suggestRow}${i === activeSuggestion ? ` ${styles.suggestRowActive}` : ''}`}
              onClick={() => handleSuggestionClick(s)}
            >
              <span className={styles.suggestFrom}>{s.fromName || s.fromEmail || 'Unknown'}</span>
              <span className={styles.suggestDate}>{formatSuggestDate(s.date)}</span>
              <span className={styles.suggestSubject}>{s.subject || '(no subject)'}</span>
            </div>
          ))}
          <div className={styles.suggestFooter}>↵ Enter to search all results</div>
        </div>
      )}

      {advancedOpen && <AdvancedSearchModal onClose={() => setAdvancedOpen(false)} />}

      {/* Smart / virtual folders — pinned, never scrolls */}
      <div className={styles.smartSection}>
        {SMART_FOLDERS.map(({ id, label, Icon }) => {
          const isActive = id === selectedFolderId;
          return (
            <div
              key={id}
              className={`${styles.folderItem}${isActive ? ` ${styles.active}` : ''}`}
              role="button"
              tabIndex={0}
              onClick={() => { setSelectedFolder(id); if (navRef.current) navRef.current.scrollTop = 0; }}
              onKeyDown={(e) => { if (e.key === 'Enter') { setSelectedFolder(id); if (navRef.current) navRef.current.scrollTop = 0; } }}
              onContextMenu={(e) => openContextMenu(e, [
                { label: 'Open folder', action: () => { setSelectedFolder(id); if (navRef.current) navRef.current.scrollTop = 0; } },
              ])}
            >
              <Icon className={styles.folderIcon} />
              <span className={styles.folderName}>{label}</span>
            </div>
          );
        })}
      </div>

      {/* Divider between smart folders and IMAP folders */}
      {accountFolders.length > 0 && <div className={styles.divider} />}

      {/* Scrollable per-account IMAP folders */}
      <nav className={styles.nav} ref={navRef}>
        {accountFolders.map(({ account, folders }) => {
          if (!folders) return null;

          const accountColor = getAccountColor(account);
          const accountKey = `account:${account.id}`;
          const foldersKey = `folders:${account.id}`;
          const accountCollapsed = !!collapsed[accountKey];
          const foldersCollapsed = !!collapsed[foldersKey];

          const special = specialOrder
            .map((use) => folders.find((f) => f.specialUse === use))
            .filter((f): f is Folder => !!f);
          const other = folders.filter((f) => !f.specialUse);
          const accountUnread = folders.reduce((n, f) => n + (f.unreadCount ?? 0), 0);

          return (
            <div key={account.id} className={styles.accountSection}>
              <button
                className={styles.accountSectionHeader}
                type="button"
                onClick={() => toggle(accountKey)}
              >
                <ChevronIcon collapsed={accountCollapsed} />
                <span className={styles.accountSectionName}>{account.name}</span>
                {accountCollapsed && accountUnread > 0 && (
                  <span className={styles.badge}>{accountUnread}</span>
                )}
              </button>

              {!accountCollapsed && (
                <>
                  {special.map((folder) => {
                    const Icon = getFolderIcon(folder.specialUse);
                    const isActive = folder.id === selectedFolderId;
                    return (
                      <div
                        key={folder.id}
                        className={`${styles.folderItem}${isActive ? ` ${styles.active}` : ''}`}
                        role="button"
                        tabIndex={0}
                        onClick={() => setSelectedFolder(folder.id)}
                        onKeyDown={(e) => e.key === 'Enter' && setSelectedFolder(folder.id)}
                        onContextMenu={(e) => openContextMenu(e, [
                          { label: 'Open folder', action: () => setSelectedFolder(folder.id) },
                          { separator: true },
                          {
                            label: 'Mark all as read',
                            action: () => {
                              fetch(`/api/v1/folders/${folder.id}/mark-read`, { method: 'POST' }).catch(() => undefined);
                            },
                          },
                        ])}
                      >
                        <span className={styles.accountColorDot} style={{ background: accountColor }} />
                        <Icon className={styles.folderIcon} />
                        <span className={styles.folderName}>{folder.name}</span>
                        {folder.unreadCount > 0 && (
                          <span className={styles.badge}>{folder.unreadCount}</span>
                        )}
                      </div>
                    );
                  })}

                  {other.length > 0 && (
                    <>
                      <button
                        className={styles.folderGroupHeader}
                        type="button"
                        onClick={() => toggle(foldersKey)}
                      >
                        <ChevronIcon collapsed={foldersCollapsed} />
                        <span>Folders</span>
                        <span className={styles.folderGroupCount}>{other.length}</span>
                      </button>
                      {!foldersCollapsed && other.map((folder) => {
                        const isActive = folder.id === selectedFolderId;
                        const depth = folderDepth(folder);
                        return (
                          <div
                            key={folder.id}
                            className={`${styles.folderItem}${isActive ? ` ${styles.active}` : ''}`}
                            style={{ paddingLeft: `${18 + depth * 12}px` }}
                            role="button"
                            tabIndex={0}
                            onClick={() => setSelectedFolder(folder.id)}
                            onKeyDown={(e) => e.key === 'Enter' && setSelectedFolder(folder.id)}
                            onContextMenu={(e) => openContextMenu(e, [
                              { label: 'Open folder', action: () => setSelectedFolder(folder.id) },
                              { separator: true },
                              {
                                label: 'Mark all as read',
                                action: () => {
                                  fetch(`/api/v1/folders/${folder.id}/mark-read`, { method: 'POST' }).catch(() => undefined);
                                },
                              },
                            ])}
                          >
                            <span className={styles.accountColorDot} style={{ background: accountColor }} />
                            <FolderIcon className={styles.folderIcon} />
                            <span className={styles.folderName}>{folder.name}</span>
                            {folder.unreadCount > 0 && (
                              <span className={styles.badge}>{folder.unreadCount}</span>
                            )}
                          </div>
                        );
                      })}
                    </>
                  )}
                </>
              )}
            </div>
          );
        })}
      </nav>

      <div className={styles.accounts}>
        <button
          className={styles.addAccountBtn}
          type="button"
          onClick={() => setShowAddModal(true)}
        >
          ＋ Add account
        </button>
        {accounts?.map((account) => {
          const avatar = getAvatarStyle(account.name);
          return (
            <div key={account.id} className={styles.accountItem}>
              <div
                className={styles.accountAvatar}
                style={{ background: avatar.bg }}
              >
                {avatar.initials}
              </div>
              <span className={styles.accountEmail}>{account.email}</span>
            </div>
          );
        })}
      </div>

      {showAddModal && (
        <AddAccountModal
          onClose={() => setShowAddModal(false)}
          onAccountAdded={() => {
            setShowAddModal(false);
            onAccountAdded();
          }}
        />
      )}

      {contextMenu}
    </aside>
  );
}
