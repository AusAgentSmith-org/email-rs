import styles from './FolderDrawer.module.css';
import type { Folder } from '../types';

const SPECIAL_ORDER = ['inbox', 'sent', 'drafts', 'trash', 'spam', 'junk', 'archive'];

function sortFolders(folders: Folder[]): { pinned: Folder[]; rest: Folder[] } {
  const pinned = SPECIAL_ORDER
    .map((s) => folders.find((f) => f.specialUse?.toLowerCase() === s || f.name.toLowerCase() === s))
    .filter(Boolean) as Folder[];
  const pinnedIds = new Set(pinned.map((f) => f.id));
  const rest = folders
    .filter((f) => !pinnedIds.has(f.id))
    .sort((a, b) => a.fullPath.localeCompare(b.fullPath));
  return { pinned, rest };
}

interface Props {
  folders: Folder[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onClose: () => void;
}

export function FolderDrawer({ folders, selectedId, onSelect, onClose }: Props) {
  const { pinned, rest } = sortFolders(folders);

  const renderFolder = (f: Folder) => (
    <button
      key={f.id}
      className={`${styles.item} ${selectedId === f.id ? styles.active : ''}`}
      onClick={() => { onSelect(f.id); onClose(); }}
    >
      <span className={styles.name}>{f.fullPath}</span>
      <div className={styles.meta}>
        {f.unreadCount > 0 && <span className={styles.unread}>{f.unreadCount}</span>}
        {f.isExcluded && <span className={styles.excluded}>excluded</span>}
      </div>
    </button>
  );

  return (
    <>
      <div className={styles.backdrop} onClick={onClose} />
      <div className={styles.drawer}>
        <div className={styles.handle} />
        <div className={styles.heading}>All Folders</div>

        <div className={`${styles.list} scroll`}>
          {pinned.length > 0 && (
            <>
              <div className={styles.section}>Mailboxes</div>
              {pinned.map(renderFolder)}
              <div className={styles.divider} />
              <div className={styles.section}>Folders</div>
            </>
          )}
          {rest.map(renderFolder)}
          <div className={styles.pad} />
        </div>
      </div>
    </>
  );
}
