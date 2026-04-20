import styles from './MessageList.module.css';
import type { Message } from '../../types';

function getAvatarStyle(name: string): { bg: string; initials: string } {
  const hue = name.split('').reduce((n, c) => n + c.charCodeAt(0), 0) % 360;
  return {
    bg: `oklch(62% 0.12 ${hue})`,
    initials: name.split(' ').map((w) => w[0]).join('').slice(0, 2).toUpperCase(),
  };
}

function formatTimestamp(dateStr: string | null): string {
  if (!dateStr) return '';
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / 86_400_000);

  if (diffDays === 0) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
  if (diffDays < 7) {
    return date.toLocaleDateString([], { weekday: 'short' });
  }
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
  }
  return date.toLocaleDateString([], { year: 'numeric', month: 'short', day: 'numeric' });
}

function AttachmentIcon() {
  return (
    <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M2 5.5L5.5 2a2 2 0 012.8 2.8L3.5 9.5a1 1 0 001.5 1.5L9.5 6.5" />
    </svg>
  );
}

function FlagIcon() {
  return (
    <svg viewBox="0 0 12 12" fill="currentColor" stroke="currentColor" strokeWidth="0.5" strokeLinejoin="round">
      <path d="M2 1.5v9M2 1.5l8 2.5-8 3.5" />
    </svg>
  );
}

interface MessageRowProps {
  message: Message;
  isSelected: boolean;
  isChecked: boolean;
  onClick: () => void;
  onCheck: (e: React.MouseEvent) => void;
  onContextMenu: (e: React.MouseEvent) => void;
  threadCount?: number;
  indent?: boolean;
}

export function MessageRow({
  message,
  isSelected,
  isChecked,
  onClick,
  onCheck,
  onContextMenu,
  threadCount,
  indent,
}: MessageRowProps) {
  const senderName = message.fromName ?? message.fromEmail ?? 'Unknown';
  const avatar = getAvatarStyle(senderName);
  const timestamp = formatTimestamp(message.date);

  const rowClass = [
    indent ? styles.threadChild : styles.messageRow,
    isSelected ? styles.selected : '',
    !message.isRead ? styles.unread : '',
    isChecked ? styles.checked : '',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div
      className={rowClass}
      data-role="message-row"
      role="button"
      tabIndex={0}
      onClick={onClick}
      onContextMenu={onContextMenu}
      onKeyDown={(e) => e.key === 'Enter' && onClick()}
    >
      {/* Avatar / checkbox area */}
      <div className={styles.checkboxArea}>
        {/* Avatar — hidden on hover or when checked */}
        <div
          className={styles.avatar}
          style={{ background: avatar.bg }}
          aria-hidden="true"
        >
          {avatar.initials}
        </div>
        {/* Checkbox — shown on hover or when checked */}
        <input
          type="checkbox"
          className={styles.checkbox}
          checked={isChecked}
          aria-label={`Select message from ${senderName}`}
          onClick={(e) => {
            e.stopPropagation();
            onCheck(e);
          }}
          onChange={() => {
            // handled by onClick
          }}
        />
      </div>

      <div className={styles.content}>
        <div className={styles.topLine}>
          <span className={styles.senderName}>{senderName}</span>
          <span className={styles.timestamp}>{timestamp}</span>
        </div>
        <span className={styles.subject}>{message.subject ?? '(no subject)'}</span>
        {message.preview && (
          <span className={styles.preview}>{message.preview}</span>
        )}
      </div>

      <div className={styles.indicators}>
        {!message.isRead && <div className={styles.unreadDot} />}
        <div className={styles.iconRow}>
          {threadCount && threadCount > 1 && (
            <span className={styles.threadCount}>{threadCount}</span>
          )}
          {message.hasAttachments && <AttachmentIcon />}
          {message.isFlagged && <FlagIcon />}
        </div>
      </div>
    </div>
  );
}
