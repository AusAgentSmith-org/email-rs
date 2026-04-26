import styles from './MailRow.module.css';
import type { Message } from '../types';

function avatar(name: string): { bg: string; initials: string } {
  const hue = name.split('').reduce((n, c) => n + c.charCodeAt(0), 0) % 360;
  return {
    bg: `oklch(62% 0.12 ${hue})`,
    initials: name.split(' ').map((w) => w[0]).join('').slice(0, 2).toUpperCase(),
  };
}

function relativeDate(dateStr: string | null): string {
  if (!dateStr) return '';
  const d = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  const days = Math.floor(diff / 86400000);
  if (days === 0) return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  if (days === 1) return 'Yesterday';
  if (days < 7) return d.toLocaleDateString([], { weekday: 'short' });
  return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
}

interface Props {
  message: Message;
  onClick: () => void;
}

export function MailRow({ message: msg, onClick }: Props) {
  const sender = msg.fromName ?? msg.fromEmail ?? 'Unknown';
  const av = avatar(sender);

  return (
    <button className={`${styles.row} ${!msg.isRead ? styles.unread : ''}`} onClick={onClick}>
      {/* Unread dot */}
      <span className={styles.dot} aria-hidden />

      {/* Avatar */}
      <div className={styles.avatar} style={{ background: av.bg }}>
        {av.initials}
      </div>

      {/* Content */}
      <div className={styles.content}>
        <div className={styles.top}>
          <span className={styles.sender}>{sender}</span>
          <span className={styles.date}>{relativeDate(msg.date)}</span>
        </div>
        <div className={styles.subject}>{msg.subject ?? '(no subject)'}</div>
        {msg.preview
          ? <div className={styles.preview}>{msg.preview}</div>
          : <div className={`${styles.preview} ${styles.previewEmpty}`}>No preview available</div>
        }
      </div>

      {/* Indicators */}
      <div className={styles.indicators}>
        {msg.isFlagged && <span className={styles.flag}>⚑</span>}
        {msg.hasAttachments && (
          <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.4" width="12" height="12">
            <path d="M10 6L6 10a3 3 0 01-4.24-4.24L7.17 1.35a1.75 1.75 0 012.47 2.47L4.23 9.23a.5.5 0 01-.71-.71L8.46 3.58" strokeLinecap="round" />
          </svg>
        )}
        <svg viewBox="0 0 8 14" fill="none" stroke="currentColor" strokeWidth="1.5" width="6" height="10" className={styles.chevron}>
          <path d="M1 1l6 6-6 6" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </div>
    </button>
  );
}
